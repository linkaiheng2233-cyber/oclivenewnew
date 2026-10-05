#![allow(clippy::expect_used, clippy::unwrap_used)]

//! External callers use the production Agent core and parser. Only model and
//! tool resources are in memory; this test grants no process or network access.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{AgentBase, BaseCallFuture, LlmBase, LlmClient, McpBridgePort};
use oclive_kernel_host::domain::BuiltinReActAgent;
use oclive_kernel_host::error::{AppError, Result};
use oclive_kernel_host::infrastructure::function_call_parser::BuiltinFunctionCallingParser;
use oclive_kernel_runtime::domain::base_emotion::KeywordEmotionBase;
use oclive_kernel_runtime::domain::base_event::LlmEventAnalyzer;
use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
use oclive_kernel_runtime::domain::base_prompt::LiteralMaterialAssembler;
use oclive_kernel_runtime::domain::minimal_role_consumer::{
    MinimalRoleBaseBindings, MinimalRoleBaseConsumer,
};
use oclive_kernel_types::{
    AgentBaseRequest, AgentToolResult, AgentToolSchema, BaseCallErrorKind, LlmBaseRequest,
    McpServerInfo, McpToolInfo, MinimalRoleDefinition,
};
use parking_lot::Mutex;
use serde_json::{json, Value};

struct Model {
    responses: Mutex<VecDeque<Result<String>>>,
    calls: Mutex<Vec<(String, String)>>,
}

impl Model {
    fn new(responses: impl IntoIterator<Item = Result<String>>) -> Arc<Self> {
        Arc::new(Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl LlmClient for Model {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        self.calls.lock().push((model.into(), prompt.into()));
        self.responses
            .lock()
            .pop_front()
            .expect("unexpected extra model round")
    }

    async fn generate_tag(&self, _: &str, _: &str) -> Result<String> {
        panic!("Agent must use its original generate entry")
    }
}

#[derive(Clone, Copy)]
enum Discovery {
    Tools,
    Empty,
    Denied,
}

struct Tools {
    discovery: Discovery,
    discoveries: AtomicUsize,
    calls: Mutex<Vec<(String, Value)>>,
}

impl Tools {
    fn new(discovery: Discovery) -> Arc<Self> {
        Arc::new(Self {
            discovery,
            discoveries: AtomicUsize::new(0),
            calls: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl McpBridgePort for Tools {
    fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
        panic!("not the Agent discovery entry")
    }

    async fn list_mcp_tools(&self, _: &str) -> Result<Vec<McpToolInfo>> {
        panic!("not the Agent discovery entry")
    }

    async fn list_agent_tool_schemas(&self) -> Result<Vec<AgentToolSchema>> {
        self.discoveries.fetch_add(1, Ordering::SeqCst);
        match self.discovery {
            Discovery::Tools => Ok(vec![AgentToolSchema {
                name: "memory::read_note".into(),
                description: Some("Read caller-owned in-memory note".into()),
            }]),
            Discovery::Empty => Ok(Vec::new()),
            Discovery::Denied => Err(AppError::HighRiskCapabilityNotGranted {
                capability: "network:local".into(),
                id: "caller-withheld".into(),
            }),
        }
    }

    async fn call_tool_qualified(&self, name: &str, params: Value) -> Result<AgentToolResult> {
        assert!(matches!(self.discovery, Discovery::Tools));
        self.calls.lock().push((name.into(), params.clone()));
        Ok(AgentToolResult {
            server_id: "memory".into(),
            tool_name: "read_note".into(),
            params,
            result: json!({"note": "门在周五关闭；是否重开未知。"}),
            error: None,
        })
    }

    async fn call_tool(&self, _: &str, _: &str, _: Value) -> Result<AgentToolResult> {
        panic!("not the Agent qualified call entry")
    }
}

fn agent(model: &Arc<Model>, tools: &Arc<Tools>) -> BuiltinReActAgent {
    BuiltinReActAgent::new(
        model.clone(),
        tools.clone(),
        Arc::new(BuiltinFunctionCallingParser),
    )
}

struct UnusedGeneration;

impl LlmBase for UnusedGeneration {
    fn generate<'a>(&'a self, _: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        panic!("delegating an Agent task must not call other Base slots")
    }
}

#[tokio::test]
async fn explicit_task_uses_real_agent_resources_and_shared_minimal_consumer() {
    let model = Model::new([
        Ok(json!({"tool_calls": [{"id": "read-1", "function": {
            "name": "memory::read_note", "arguments": {"key": "入口"}
        }}]})
        .to_string()),
        Ok(json!({"final_answer": "记录说周五关闭；重开时间未知。"}).to_string()),
    ]);
    let tools = Tools::new(Discovery::Tools);
    let agent = agent(&model, &tools);
    let selected_model = String::from("caller-selected-model");
    let role_id = String::from("converter-owned-role");
    let session = String::from("temporary-binding-甲");
    let view = agent.task_execution_base(&selected_model, &role_id, &session);
    let definition = MinimalRoleDefinition {
        persona_prompt: "本角色没有关系或人格扩展；不要自动追加工具权限。".into(),
        visual_assets: vec!["memory:portrait".into()],
    };
    let event = LlmEventAnalyzer::new(&UnusedGeneration);
    let consumer = MinimalRoleBaseConsumer::new(
        &definition,
        MinimalRoleBaseBindings {
            memory: &KeywordMemoryBase,
            emotion: &KeywordEmotionBase,
            event: &event,
            prompt: &LiteralMaterialAssembler,
            llm: &UnusedGeneration,
            agent: &view,
        },
    )
    .unwrap();
    assert!(model.calls.lock().is_empty());
    assert_eq!(tools.discoveries.load(Ordering::SeqCst), 0);
    assert!(tools.calls.lock().is_empty());
    let task = "请读我的入口记录，报告已知关闭日期和未知事项。";
    let report = consumer
        .execute(AgentBaseRequest {
            task,
            context: None,
        })
        .await
        .unwrap();
    assert!(report.contains("记录说周五关闭；重开时间未知。"));
    assert!(report.contains("不代表委托的客观目标已经达成"));
    let calls = model.calls.lock();
    assert_eq!(calls.len(), 2);
    assert!(calls
        .iter()
        .all(|(name, prompt)| name == &selected_model && prompt.contains(task)));
    assert!(!calls[0].1.contains(&definition.persona_prompt));
    assert!(calls[1].1.contains("是否重开未知"));
    assert_eq!(tools.discoveries.load(Ordering::SeqCst), 1);
    assert_eq!(
        *tools.calls.lock(),
        [("memory::read_note".into(), json!({"key": "入口"}))]
    );
    let traces = agent.recent_traces();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].role_id, role_id);
    assert_eq!(traces[0].session_namespace, session);
    assert_eq!(traces[0].message, task);
    assert_eq!(traces[0].tool_calls.len(), 1);
}

#[tokio::test]
async fn nonempty_context_is_refused_before_any_resource_call() {
    let model = Model::new([]);
    let tools = Tools::new(Discovery::Tools);
    let agent = agent(&model, &tools);
    let view = agent.task_execution_base("selected", "actual-role", "actual-session");
    let error = view
        .execute(AgentBaseRequest {
            task: "请读取记录",
            context: Some(" "),
        })
        .await
        .unwrap_err();
    assert_eq!(error.kind, BaseCallErrorKind::Unsupported);
    assert_eq!(tools.discoveries.load(Ordering::SeqCst), 0);
    assert!(model.calls.lock().is_empty());
    assert!(tools.calls.lock().is_empty());
    assert!(agent.recent_traces().is_empty());
}

#[tokio::test]
async fn empty_task_and_no_tools_report_not_taken_without_generation() {
    let model = Model::new([]);
    let tools = Tools::new(Discovery::Empty);
    let agent = agent(&model, &tools);
    let view = agent.task_execution_base("selected", "actual-role", "actual-session");
    let empty = view
        .execute(AgentBaseRequest {
            task: " \n",
            context: None,
        })
        .await
        .unwrap();
    assert!(empty.contains("委托文本为空"));
    assert_eq!(tools.discoveries.load(Ordering::SeqCst), 0);
    let report = view
        .execute(AgentBaseRequest {
            task: "请读记录",
            context: Some(""),
        })
        .await
        .unwrap();
    assert!(report.contains("没有可用的工具 schema"));
    assert_eq!(tools.discoveries.load(Ordering::SeqCst), 1);
    assert!(model.calls.lock().is_empty());
    assert!(tools.calls.lock().is_empty());
}

#[tokio::test]
async fn withheld_discovery_grant_stays_unavailable_with_original_diagnostic() {
    let model = Model::new([]);
    let tools = Tools::new(Discovery::Denied);
    let agent = agent(&model, &tools);
    let view = agent.task_execution_base("selected", "actual-role", "actual-session");
    let error = view
        .execute(AgentBaseRequest {
            task: "请读记录",
            context: None,
        })
        .await
        .unwrap_err();
    assert_eq!(error.kind, BaseCallErrorKind::Unavailable);
    assert_eq!(
        error.detail,
        Some(
            AppError::HighRiskCapabilityNotGranted {
                capability: "network:local".into(),
                id: "caller-withheld".into(),
            }
            .to_string()
        )
    );
    assert_eq!(tools.discoveries.load(Ordering::SeqCst), 1);
    assert!(model.calls.lock().is_empty());
    assert!(tools.calls.lock().is_empty());
}

#[tokio::test]
async fn model_failure_is_failed_without_retry_or_tool_effect() {
    let diagnostic =
        AppError::Unknown("transport TimedOut Cancelled words are not typed states".into());
    let original = diagnostic.to_string();
    let model = Model::new([Err(diagnostic)]);
    let tools = Tools::new(Discovery::Tools);
    let agent = agent(&model, &tools);
    let view = agent.task_execution_base("selected", "actual-role", "actual-session");
    let error = view
        .execute(AgentBaseRequest {
            task: "请读记录",
            context: None,
        })
        .await
        .unwrap_err();
    assert_eq!(error.kind, BaseCallErrorKind::Failed);
    assert_eq!(error.detail, Some(original.clone()));
    assert_eq!(model.calls.lock().len(), 1);
    assert!(tools.calls.lock().is_empty());
    assert_eq!(agent.recent_traces()[0].error, Some(original));
}
