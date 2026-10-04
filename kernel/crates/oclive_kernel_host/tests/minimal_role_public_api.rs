#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use oclive_kernel_contracts::{
    AgentBase, BaseCallFuture, EmotionBase, EventBase, LlmBase, LlmClient, LlmGenerateOpts,
    LlmGenerateOutcome, MemoryBase, PromptBase,
};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::{
    MinimalRoleMessageError, OcliveKernel, OcliveKernelConfig, PreparedMinimalRole,
};
use oclive_kernel_types::models::dto::{
    MinimalRoleMessageRequest, MinimalRoleProductExtensionStatus,
};
use oclive_kernel_types::{
    AppError, BaseCallError, BaseCallErrorKind, MinimalRoleDefinition, PromptBaseRequest,
};

struct RecordingPrompt {
    calls: Cell<usize>,
    materials: RefCell<Vec<String>>,
    requirements: RefCell<String>,
    outcome: std::result::Result<String, BaseCallError>,
}

impl RecordingPrompt {
    fn new(outcome: std::result::Result<String, BaseCallError>) -> Self {
        Self {
            calls: Cell::new(0),
            materials: RefCell::default(),
            requirements: RefCell::default(),
            outcome,
        }
    }
}

impl PromptBase for RecordingPrompt {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.set(self.calls.get() + 1);
        Box::pin(async move {
            // Force a suspension before reading the consumer's borrowed fragments.
            tokio::task::yield_now().await;
            self.materials.replace(
                request
                    .materials
                    .iter()
                    .map(|text| (*text).into())
                    .collect(),
            );
            self.requirements.replace(request.requirements.into());
            self.outcome.clone()
        })
    }
}

struct RecordingLlm {
    calls: Mutex<Vec<(String, String)>>,
    reply: &'static str,
    fail: bool,
}

#[async_trait]
impl LlmClient for RecordingLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> oclive_kernel_types::Result<String> {
        panic!("minimal Host must use the existing checked generate_with_opts entry")
    }

    async fn generate_tag(
        &self,
        _model: &str,
        _prompt: &str,
    ) -> oclive_kernel_types::Result<String> {
        panic!("minimal text path must not invoke a rich classification extension")
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> oclive_kernel_types::Result<LlmGenerateOutcome> {
        assert!(opts.is_none());
        self.calls
            .lock()
            .unwrap()
            .push((model.into(), prompt.into()));
        if self.fail {
            return Err(AppError::OllamaError("synthetic model failure".into()));
        }
        Ok(LlmGenerateOutcome {
            reply: self.reply.into(),
            prompt_eval_ms: None,
        })
    }
}

fn definition() -> MinimalRoleDefinition {
    MinimalRoleDefinition {
        persona_prompt: "A guide prepared by a distro converter.".into(),
        visual_assets: vec!["memory:portrait".into()],
    }
}

fn role() -> PreparedMinimalRole {
    PreparedMinimalRole::new(
        "converter-owned-id",
        definition(),
        vec![b"portrait".to_vec()],
    )
    .expect("prepare minimal content")
}

fn request() -> MinimalRoleMessageRequest {
    MinimalRoleMessageRequest {
        user_message: "Hello, guide.".into(),
        requirements: String::new(),
    }
}

async fn kernel(
    temp: &tempfile::TempDir,
    reply: &'static str,
    fail: bool,
) -> (OcliveKernel, Arc<RecordingLlm>) {
    let roles = temp.path().join("fixture-role-root");
    std::fs::create_dir(&roles).unwrap();
    let llm = Arc::new(RecordingLlm {
        calls: Mutex::default(),
        reply,
        fail,
    });
    let kernel = OcliveKernel::builder(OcliveKernelConfig::new(temp.path(), roles))
        .with_host_profile(HostProfile::default())
        .with_llm_client(llm.clone())
        .build()
        .await
        .expect("production Host builder with in-memory model");
    (kernel, llm)
}

async fn close_fixture(kernel: OcliveKernel, temp: tempfile::TempDir) {
    let db_path = kernel.config().database_path().to_owned();
    kernel.shutdown().await;
    // On Windows the SQLite worker may still release its native file handle
    // after the pool has acknowledged close. Observe release, with a hard bound;
    // never turn a permanent cleanup failure into a passing test.
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                match std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .share_mode(0)
                    .open(&db_path)
                {
                    Ok(file) => {
                        drop(file);
                        break;
                    }
                    Err(error) if error.raw_os_error() == Some(32) => {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                    Err(error) => panic!("fixture DB release probe failed: {error}"),
                }
            }
        })
        .await
        .expect("fixture DB handle released within cleanup deadline");
    }
    #[cfg(not(windows))]
    let _ = db_path;
    temp.close()
        .expect("fixture directory removed after shutdown");
}

#[tokio::test]
async fn prepared_minimal_content_runs_without_rich_role_or_synthetic_extensions() {
    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "Welcome, traveller.", false).await;
    let role = role();
    let response = kernel
        .process_minimal_message(&role, &request())
        .await
        .unwrap();
    assert_eq!(response.role_id, "converter-owned-id");
    assert_eq!(response.reply, "Welcome, traveller.");
    assert_eq!(
        response.product_extensions,
        MinimalRoleProductExtensionStatus::Unavailable
    );
    assert_eq!(
        role.assets().collect::<Vec<_>>(),
        vec![("memory:portrait", b"portrait".as_slice())]
    );
    {
        let calls = llm.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "【角色设定】\nA guide prepared by a distro converter.\n\n【输入材料】\nUser: Hello, guide.");
    }
    let value = serde_json::to_value(response).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 3);
    assert_eq!(value["product_extensions"], "unavailable");
    assert!(!temp
        .path()
        .join("fixture-role-root/converter-owned-id")
        .exists());
    assert!(kernel.list_roles().await.unwrap().is_empty());

    let db_path = kernel.config().database_path().to_owned();
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .read_only(true),
        )
        .await
        .unwrap();
    for query in [
        "SELECT COUNT(*) FROM role_runtime",
        "SELECT COUNT(*) FROM chat_messages",
        "SELECT COUNT(*) FROM chat_request_receipts",
    ] {
        let count: i64 = sqlx::query_scalar(query).fetch_one(&db).await.unwrap();
        assert_eq!(count, 0);
    }
    db.close().await;
    close_fixture(kernel, temp).await;
}

#[test]
fn invalid_preparation_never_returns_a_role_handle() {
    assert!(matches!(
        PreparedMinimalRole::new(" ", definition(), vec![vec![1]]),
        Err(AppError::InvalidParameter(_))
    ));
    let mut blank_persona = definition();
    blank_persona.persona_prompt.clear();
    assert!(PreparedMinimalRole::new("id", blank_persona, vec![vec![1]]).is_err());
    let mut blank_reference = definition();
    blank_reference.visual_assets[0].clear();
    assert!(PreparedMinimalRole::new("id", blank_reference, vec![vec![1]]).is_err());
    assert!(PreparedMinimalRole::new("id", definition(), vec![]).is_err());
    assert!(PreparedMinimalRole::new("id", definition(), vec![vec![]]).is_err());
    assert!(PreparedMinimalRole::new("id", definition(), vec![vec![1], vec![2]]).is_err());
}

#[tokio::test]
async fn extra_requirements_and_blank_message_are_rejected_before_model_calls() {
    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "must not be called", false).await;
    let role = role();
    let mut req = request();
    req.requirements = "update the relationship score".into();
    assert!(matches!(kernel.process_minimal_message(&role, &req).await,
        Err(MinimalRoleMessageError::Prompt(error)) if error.kind == BaseCallErrorKind::Unsupported));
    req.requirements.clear();
    req.user_message = " \n ".into();
    assert!(matches!(
        kernel.process_minimal_message(&role, &req).await,
        Err(MinimalRoleMessageError::Host(AppError::EmptyMessage))
    ));
    assert!(llm.calls.lock().unwrap().is_empty());
    // A future transport cannot silently ignore an explicit rich request field.
    assert!(serde_json::from_str::<MinimalRoleMessageRequest>(
        r#"{"user_message":"hi","adult":{}}"#
    )
    .is_err());
    close_fixture(kernel, temp).await;
}

#[tokio::test]
async fn model_failure_is_preserved_and_completed_empty_text_is_not_fabricated() {
    for fail in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let (kernel, llm) = kernel(&temp, "", fail).await;
        let result = kernel.process_minimal_message(&role(), &request()).await;
        if fail {
            assert!(
                matches!(result, Err(MinimalRoleMessageError::Host(AppError::OllamaError(message))) if message == "synthetic model failure")
            );
        } else {
            assert_eq!(result.unwrap().reply, "");
        }
        assert_eq!(llm.calls.lock().unwrap().len(), 1);
        close_fixture(kernel, temp).await;
    }
}

#[test]
fn optional_local_adapter_keeps_snapshots_without_rereading_or_defining_a_format() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("chosen.json"),
        r#"{"persona_prompt":"A local guide.","visual_assets":["portrait.bin"]}"#,
    )
    .unwrap();
    std::fs::write(temp.path().join("portrait.bin"), b"original").unwrap();
    let snapshot = oclive_validation::minimal_role_local_file::load_minimal_role_local_file(
        temp.path(),
        "chosen.json",
        1024,
        1024,
        1024,
    )
    .unwrap();
    std::fs::write(temp.path().join("portrait.bin"), b"changed").unwrap();
    let role = PreparedMinimalRole::from_local_snapshot("local-id", &snapshot).unwrap();
    assert_eq!(
        role.assets().next().unwrap(),
        ("portrait.bin", b"original".as_slice())
    );
    assert_eq!(role.definition().persona_prompt, "A local guide.");
}

#[tokio::test]
async fn host_can_select_prompt_with_extra_requirements_and_local_async_state() {
    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "Selected capability reply.", false).await;
    let role = role();
    let request = MinimalRoleMessageRequest {
        user_message: "  Hello, café.\r\n".into(),
        requirements: "  Preserve the subject.\r\n".into(),
    };
    let prompt = RecordingPrompt::new(Ok("  Prepared by the selected Prompt.\r\n".into()));
    let response = kernel
        .process_minimal_message_with_prompt(&role, &request, &prompt)
        .await
        .unwrap();

    assert_eq!(response.role_id, "converter-owned-id");
    assert_eq!(response.reply, "Selected capability reply.");
    assert_eq!(
        response.product_extensions,
        MinimalRoleProductExtensionStatus::Unavailable
    );
    assert_eq!(prompt.calls.get(), 1);
    assert_eq!(
        *prompt.materials.borrow(),
        [
            "【角色设定】\n",
            "A guide prepared by a distro converter.",
            "\n\n【输入材料】\n",
            "User: ",
            "  Hello, café.\r\n",
        ]
    );
    assert_eq!(*prompt.requirements.borrow(), request.requirements);
    assert!(kernel.list_roles().await.unwrap().is_empty());
    {
        let calls = llm.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "  Prepared by the selected Prompt.\r\n");
    }
    let default_request = MinimalRoleMessageRequest {
        user_message: "Default remains literal.".into(),
        requirements: String::new(),
    };
    kernel
        .process_minimal_message(&role, &default_request)
        .await
        .unwrap();
    assert_eq!(prompt.calls.get(), 1);
    {
        let calls = llm.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            calls[1].1,
            "【角色设定】\nA guide prepared by a distro converter.\n\n【输入材料】\nUser: Default remains literal."
        );
    }
    close_fixture(kernel, temp).await;
}

#[tokio::test]
async fn selected_prompt_errors_stop_before_model_and_preserve_complete_error() {
    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "Must not generate.", false).await;
    let role = role();
    for kind in [
        BaseCallErrorKind::Failed,
        BaseCallErrorKind::Unavailable,
        BaseCallErrorKind::Unsupported,
        BaseCallErrorKind::Cancelled,
        BaseCallErrorKind::TimedOut,
    ] {
        let expected = BaseCallError {
            kind,
            detail: Some("selected provider detail\r\n".into()),
        };
        let prompt = RecordingPrompt::new(Err(expected.clone()));
        let result = kernel
            .process_minimal_message_with_prompt(&role, &request(), &prompt)
            .await;
        match result {
            Err(MinimalRoleMessageError::Prompt(actual)) => assert_eq!(actual, expected),
            other => panic!("expected the provider's complete error, got {other:?}"),
        }
        assert_eq!(prompt.calls.get(), 1);
        assert!(llm.calls.lock().unwrap().is_empty());
    }
    close_fixture(kernel, temp).await;
}

#[tokio::test]
async fn empty_user_input_does_not_invoke_selected_prompt_or_model() {
    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "Must not generate.", false).await;
    let prompt = RecordingPrompt::new(Ok("Must not assemble.".into()));
    let request = MinimalRoleMessageRequest {
        user_message: " \r\n".into(),
        requirements: "Do not discard this requirement.".into(),
    };
    assert!(matches!(
        kernel
            .process_minimal_message_with_prompt(&role(), &request, &prompt)
            .await,
        Err(MinimalRoleMessageError::Host(AppError::EmptyMessage))
    ));
    assert_eq!(prompt.calls.get(), 0);
    assert!(prompt.materials.borrow().is_empty());
    assert!(llm.calls.lock().unwrap().is_empty());
    close_fixture(kernel, temp).await;
}

#[tokio::test]
async fn selected_prompt_empty_output_and_model_failure_are_not_rewritten() {
    for fail in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let (kernel, llm) = kernel(&temp, "", fail).await;
        let prompt = RecordingPrompt::new(Ok(String::new()));
        let result = kernel
            .process_minimal_message_with_prompt(&role(), &request(), &prompt)
            .await;
        if fail {
            assert!(matches!(
                result,
                Err(MinimalRoleMessageError::Host(AppError::OllamaError(message)))
                    if message == "synthetic model failure"
            ));
        } else {
            assert_eq!(result.unwrap().reply, "");
        }
        assert_eq!(prompt.calls.get(), 1);
        {
            let calls = llm.calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].1, "");
        }
        close_fixture(kernel, temp).await;
    }
}

#[tokio::test]
async fn composed_host_model_can_serve_the_shared_six_base_consumer() {
    use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
    use oclive_kernel_runtime::domain::base_emotion::KeywordEmotionBase;
    use oclive_kernel_runtime::domain::base_event::LlmEventAnalyzer;
    use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
    use oclive_kernel_runtime::domain::minimal_role_consumer::{
        MinimalRoleBaseBindings, MinimalRoleBaseConsumer,
    };
    use oclive_kernel_runtime::domain::prompt_assembler::BuiltinPromptAssembler;
    use oclive_kernel_types::{
        AgentBaseRequest, EmotionBaseRequest, EventBaseRequest, LlmBaseRequest, MemoryBaseRequest,
    };

    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(
        &temp,
        "ANALYSIS\nA quoted speaker expressed happiness.",
        false,
    )
    .await;
    let role = role();
    let model = kernel.text_generation_base();
    let memory = KeywordMemoryBase;
    let emotion = KeywordEmotionBase;
    let event = LlmEventAnalyzer::new(&model);
    let prompt = BuiltinPromptAssembler;
    let agent = ScalarCountAgent;
    let consumer = MinimalRoleBaseConsumer::new(
        role.definition(),
        MinimalRoleBaseBindings {
            memory: &memory,
            emotion: &emotion,
            event: &event,
            prompt: &prompt,
            llm: &model,
            agent: &agent,
        },
    )
    .unwrap();
    assert!(
        llm.calls.lock().unwrap().is_empty(),
        "binding must not generate"
    );
    let candidates = ["She likes coffee.", "He prefers tea."];
    let selected = consumer
        .retrieve(MemoryBaseRequest {
            materials: &candidates,
            query: "coffee",
        })
        .await
        .unwrap();
    assert_eq!(selected, ["She likes coffee."]);
    let material = "她说：我很开心，想聊咖啡。";
    let feeling = EmotionBase::analyze(
        &consumer,
        EmotionBaseRequest {
            material,
            context: None,
        },
    )
    .await
    .unwrap()
    .expect("literal happiness clue");
    let analysis = EventBase::analyze(
        &consumer,
        EventBaseRequest {
            material,
            context: Some(&feeling),
        },
    )
    .await
    .unwrap()
    .unwrap();
    let task = consumer
        .execute(AgentBaseRequest {
            task: "请统计材料中 Unicode 标量值的个数",
            context: Some("aé😀"),
        })
        .await
        .unwrap();
    assert_eq!(task, "本次完成了计数：3 个 Unicode 标量值。");
    let materials = [
        selected[0].as_str(),
        feeling.as_str(),
        analysis.as_str(),
        task.as_str(),
    ];
    let input = consumer
        .assemble(PromptBaseRequest {
            materials: &materials,
            requirements: "",
        })
        .await
        .unwrap();
    let reply = consumer
        .generate(LlmBaseRequest { input: &input })
        .await
        .unwrap();
    assert_eq!(reply, "ANALYSIS\nA quoted speaker expressed happiness.");
    {
        let calls = llm.calls.lock().unwrap();
        assert_eq!(
            calls.len(),
            2,
            "explicit Event analysis plus final generation"
        );
        assert!(calls[0].1.contains(material));
        assert!(!calls[0].1.contains(&role.definition().persona_prompt));
        assert_eq!(calls[1].1, input);
        assert_eq!(
            calls[1]
                .1
                .matches(&role.definition().persona_prompt)
                .count(),
            1
        );
        for material in materials {
            assert!(calls[1].1.contains(material));
        }
        assert!(!calls[0].0.is_empty());
        assert_eq!(calls[0].0, calls[1].0);
    }
    // No role activation or default rich context was needed by this Rust caller.
    assert!(kernel.list_roles().await.unwrap().is_empty());
    drop(model);
    close_fixture(kernel, temp).await;
}

#[tokio::test]
async fn composed_host_base_keeps_raw_input_and_normal_empty_output() {
    use oclive_kernel_types::LlmBaseRequest;

    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "", false).await;
    let model = kernel.text_generation_base();
    assert!(llm.calls.lock().unwrap().is_empty());
    for input in ["", "  café 😀\r\n"] {
        assert_eq!(model.generate(LlmBaseRequest { input }).await.unwrap(), "");
    }
    {
        let calls = llm.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].1, "");
        assert_eq!(calls[1].1, "  café 😀\r\n");
    }
    drop(model);
    close_fixture(kernel, temp).await;
}

#[tokio::test]
async fn composed_host_base_projects_failure_without_changing_legacy_error() {
    use oclive_kernel_types::LlmBaseRequest;

    let temp = tempfile::tempdir().unwrap();
    let (kernel, llm) = kernel(&temp, "must not become a fallback", true).await;
    let model = kernel.text_generation_base();
    let error = model
        .generate(LlmBaseRequest {
            input: "prepared input",
        })
        .await
        .unwrap_err();
    assert_eq!(error.kind, BaseCallErrorKind::Failed);
    assert_eq!(
        error.detail.as_deref(),
        Some("Ollama error: synthetic model failure")
    );
    assert_eq!(llm.calls.lock().unwrap().len(), 1);
    drop(model);
    assert!(matches!(
        kernel.process_minimal_message(&role(), &request()).await,
        Err(MinimalRoleMessageError::Host(AppError::OllamaError(message)))
            if message == "synthetic model failure"
    ));
    assert_eq!(llm.calls.lock().unwrap().len(), 2);
    close_fixture(kernel, temp).await;
}
