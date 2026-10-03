#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::{
    MinimalRoleMessageError, OcliveKernel, OcliveKernelConfig, PreparedMinimalRole,
};
use oclive_kernel_types::models::dto::{
    MinimalRoleMessageRequest, MinimalRoleProductExtensionStatus,
};
use oclive_kernel_types::{AppError, BaseCallErrorKind, MinimalRoleDefinition};

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
