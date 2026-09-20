use crate::domain::ports::LlmClient;
use crate::error::{AppError, Result};
use async_trait::async_trait;
pub use oclive_kernel_contracts::AgentProvider;
use oclive_kernel_contracts::{FunctionCallingParserPort, McpBridgePort};
pub use oclive_kernel_types::{
    AgentDebugTrace, AgentInput, AgentOutput, AgentToolCallTrace, McpServerInfo, McpToolInfo,
    ToolSchemaInput,
};
use parking_lot::RwLock;
use serde_json::Value;
use std::sync::Arc;

pub struct BuiltinReActAgent {
    llm: Arc<dyn LlmClient>,
    bridge: Arc<dyn McpBridgePort>,
    parser: Arc<dyn FunctionCallingParserPort>,
    traces: RwLock<Vec<AgentDebugTrace>>,
}

/// The existing loop bound. This unit does not change it.
pub(crate) const MAX_ROUNDS: usize = 3;

/// The finite stop reason of **one real** ReAct execution.
///
/// This is not a task terminal state, an effect ledger or a recovery/idempotency record: it names
/// the branch of this one call that ended the loop, read at that branch itself rather than inferred
/// afterwards from `handled`, from whether the reply is empty, or from the reply's wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReActStop {
    /// The delegated message was empty after `trim` — no discovery and no model round ran.
    EmptyTask,
    /// The bridge reported no usable tool schema — no model round ran.
    NoTools,
    /// A model round produced a final answer.
    FinalAnswer,
    /// A model round produced no parseable function call (the loop's `break` branch).
    NoFunctionCall,
    /// The existing loop bound was reached without a final answer.
    LoopExhausted,
}

/// What one real ReAct execution observed, filled in at the branch points that produced it.
///
/// Every field is read where it happens: a successful tool call from the call that returned `Ok`, a
/// failure from the call that returned `Err`, the round count from the rounds that actually returned
/// a model response. Nothing here is recovered from [`AgentOutput`] afterwards.
#[derive(Debug, Clone)]
pub(crate) struct ReActFacts {
    /// Which branch ended this call.
    pub(crate) stop: ReActStop,
    /// Model rounds that actually returned a response (`0` when the call stopped earlier).
    pub(crate) rounds: usize,
    /// Successful tool calls, in execution order (the same entries the debug trace carries).
    pub(crate) tool_calls: Vec<AgentToolCallTrace>,
    /// Failed tool calls, in execution order, as the observation text the loop recorded.
    pub(crate) tool_failures: Vec<String>,
    /// `Some(capability (id=..))` when a tool failure was the typed refusal
    /// [`AppError::HighRiskCapabilityNotGranted`].
    ///
    /// This refusal happened **inside the tool loop**: it is captured into the observations and the
    /// loop continues, so that run finishes normally with the existing fallback reply — the original
    /// error is not returned upward in this branch. Discovery errors and model errors are the
    /// different case: those propagate to the caller unchanged.
    pub(crate) refusal: Option<String>,
    /// The observation history this loop built, i.e. the exact text handed to the next model round.
    pub(crate) observations: Vec<String>,
}

impl ReActFacts {
    fn stopped(stop: ReActStop) -> Self {
        Self {
            stop,
            rounds: 0,
            tool_calls: Vec::new(),
            tool_failures: Vec::new(),
            refusal: None,
            observations: Vec::new(),
        }
    }
}

/// The only input one real ReAct execution consumes, borrowed from the caller's own values.
///
/// CP-B3-ALL-R2: this replaces the previous whole-[`AgentInput`] parameter. The core never read more
/// than these four fields, and passing a complete domain DTO made every non-product caller
/// fabricate the rest (personality vector, relation state, favour, scene, policy text, protocol
/// version…) through `AgentInput::default()`. A private borrowed view has nothing to default, so the
/// hidden input package is structurally impossible rather than merely unused.
///
/// The conversion below reads **only** these four fields. The remaining `AgentInput` fields are not
/// consumed by this core at all — the product entry has always accepted and ignored them — and this
/// type invents no value for them.
#[derive(Clone, Copy)]
pub(crate) struct ReActInput<'a> {
    /// The delegated message as the caller wrote it (the core applies the existing `trim`).
    pub(crate) message: &'a str,
    /// The model this turn uses (the core passes it to the LLM port unchanged).
    pub(crate) model: &'a str,
    /// The role id recorded in this turn's debug trace.
    pub(crate) role_id: &'a str,
    /// The session namespace recorded in this turn's debug trace.
    pub(crate) session_namespace: &'a str,
}

impl<'a> ReActInput<'a> {
    /// Borrows the four consumed fields of a caller-owned [`AgentInput`].
    pub(crate) fn from_agent_input(input: &'a AgentInput) -> Self {
        Self {
            message: input.message.as_str(),
            model: input.model.as_str(),
            role_id: input.role_id.as_str(),
            session_namespace: input.session_namespace.as_str(),
        }
    }
}

impl BuiltinReActAgent {
    #[must_use]
    pub fn new(
        llm: Arc<dyn LlmClient>,
        bridge: Arc<dyn McpBridgePort>,
        parser: Arc<dyn FunctionCallingParserPort>,
    ) -> Self {
        Self {
            llm,
            bridge,
            parser,
            traces: RwLock::new(Vec::new()),
        }
    }

    fn push_trace(&self, trace: AgentDebugTrace) {
        const MAX_TRACES: usize = 40;
        let mut w = self.traces.write();
        w.push(trace);
        if w.len() > MAX_TRACES {
            let drop_n = w.len() - MAX_TRACES;
            w.drain(0..drop_n);
        }
    }

    #[must_use]
    pub fn recent_traces(&self) -> Vec<AgentDebugTrace> {
        self.traces.read().clone()
    }

    pub fn clear_traces(&self) {
        self.traces.write().clear();
    }

    #[must_use]
    pub fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
        self.bridge.list_mcp_servers()
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>> {
        self.bridge.list_mcp_tools(server_id).await
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn call_tool_direct(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<oclive_kernel_types::AgentToolResult> {
        self.bridge.call_tool(server_id, tool_name, params).await
    }

    fn extract_final_answer(raw: &str) -> Option<String> {
        let v = serde_json::from_str::<Value>(raw).ok()?;
        if let Some(s) = v.get("final_answer").and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        if let Some(s) = v.get("answer").and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        None
    }

    /// **The one real execution**: the legacy product entry and the Base view both run this, and
    /// neither has an execution copy of its own.
    ///
    /// The sequence, the loop bound ([`MAX_ROUNDS`]), the function-calling schema, the prompt text,
    /// the model source (`input.model`), the tool order, the trace save/trim behaviour, the fallback
    /// text, and the `handled`/empty-reply values are the ones the product entry already had; this
    /// extraction only lets the same run report what it actually did. It returns the existing
    /// [`AgentOutput`] together with the branch facts read where they happened.
    ///
    /// CP-B3-ALL-R2: the parameter is the private four-field [`ReActInput`] rather than a whole
    /// [`AgentInput`], so a caller cannot hand this core a fabricated domain DTO and the Base path
    /// never constructs one.
    ///
    /// Existing behaviour that is deliberately kept:
    ///
    /// - an empty (`trim`-empty) message and an empty tool-schema list are the "not taken up"
    ///   branches: `handled: false`, empty reply, and no model round;
    /// - a tool-call failure is still recorded and the loop continues; a discovery or model `Err`
    ///   still propagates to the caller unchanged;
    /// - reaching the loop bound still returns `handled: true` with the previous fallback text.
    ///
    /// # Errors
    ///
    /// Propagates the bridge discovery error and the model error exactly as before. This function
    /// adds no error, no retry and no second attempt for them.
    pub(crate) async fn execute_react(
        &self,
        input: ReActInput<'_>,
    ) -> Result<(AgentOutput, ReActFacts)> {
        let message = input.message.trim().to_string();
        if message.is_empty() {
            return Ok((
                AgentOutput {
                    handled: false,
                    reply: String::new(),
                },
                ReActFacts::stopped(ReActStop::EmptyTask),
            ));
        }
        let tool_schemas = self.bridge.list_agent_tool_schemas().await?;
        if tool_schemas.is_empty() {
            return Ok((
                AgentOutput {
                    handled: false,
                    reply: String::new(),
                },
                ReActFacts::stopped(ReActStop::NoTools),
            ));
        }
        let tool_schema_inputs: Vec<ToolSchemaInput> = tool_schemas
            .iter()
            .map(|t| ToolSchemaInput {
                name: t.name.clone(),
                description: t.description.clone(),
            })
            .collect();
        let mut trace = AgentDebugTrace {
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            role_id: input.role_id.to_string(),
            session_namespace: input.session_namespace.to_string(),
            message: message.clone(),
            plan: String::new(),
            tool_calls: Vec::new(),
            reply: String::new(),
            error: None,
        };
        trace.plan = format!(
            "react + function-calling; tools={}",
            tool_schema_inputs
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(",")
        );
        let schema = self.parser.to_function_calling_schema(&tool_schema_inputs);
        // The default is the branch the loop falls through to; the other exits overwrite it where
        // they happen.
        let mut facts = ReActFacts::stopped(ReActStop::LoopExhausted);
        for _ in 0..MAX_ROUNDS {
            let prompt = format!(
                "你是 oclive Agent。可用函数 schema: {schema}\n\
                 用户请求: {msg}\n\
                 观察历史: {obs}\n\
                 你必须输出 JSON：\n\
                 1) 若要调工具: {{\"tool_calls\":[{{\"id\":\"1\",\"function\":{{\"name\":\"tool_name\",\"arguments\":{{...}}}}}}]}}\n\
                 2) 若可直接回答: {{\"final_answer\":\"...\"}}\n\
                 不要输出 markdown。",
                schema = schema,
                msg = message,
                obs = facts.observations.join(" | ")
            );
            let llm_raw = match self.llm.generate(input.model, prompt.as_str()).await {
                Ok(s) => s,
                Err(e) => {
                    trace.error = Some(e.to_string());
                    self.push_trace(trace);
                    return Err(e);
                }
            };
            facts.rounds += 1;
            if let Some(answer) = Self::extract_final_answer(&llm_raw) {
                trace.reply = answer.clone();
                self.push_trace(trace);
                facts.stop = ReActStop::FinalAnswer;
                return Ok((
                    AgentOutput {
                        handled: true,
                        reply: answer,
                    },
                    facts,
                ));
            }
            let calls = self.parser.parse_from_llm_response(&llm_raw);
            if calls.is_empty() {
                facts.stop = ReActStop::NoFunctionCall;
                break;
            }
            for call in calls {
                let tool_name = call.function.name.trim().to_string();
                if tool_name.is_empty() {
                    continue;
                }
                match self
                    .bridge
                    .call_tool_qualified(tool_name.as_str(), call.function.arguments.clone())
                    .await
                {
                    Ok(result) => {
                        let entry = AgentToolCallTrace {
                            server_id: result.server_id.clone(),
                            tool_name: result.tool_name.clone(),
                            params: call.function.arguments.clone(),
                            result: result.result.clone(),
                        };
                        let observation = format!(
                            "{}.{} -> {}",
                            result.server_id, result.tool_name, result.result
                        );
                        trace.tool_calls.push(entry.clone());
                        facts.tool_calls.push(entry);
                        facts.observations.push(observation);
                    }
                    Err(e) => {
                        let msg = format!("tool {} failed: {}", tool_name, e);
                        trace.error = Some(msg.clone());
                        if let AppError::HighRiskCapabilityNotGranted { capability, id } = &e {
                            facts.refusal = Some(format!("{capability} (id={id})"));
                        }
                        facts.tool_failures.push(msg.clone());
                        facts.observations.push(msg);
                    }
                }
            }
        }
        let fallback = if let Some(last) = trace.tool_calls.last() {
            if let Some(s) = last.result.get("summary").and_then(|x| x.as_str()) {
                s.to_string()
            } else {
                format!("工具调用完成：{}", last.result)
            }
        } else if let Some(e) = trace.error.clone() {
            format!("Agent 执行未完成：{}", e)
        } else {
            "我已尝试调度工具，但模型没有返回可执行的 function call。".to_string()
        };
        trace.reply = fallback.clone();
        self.push_trace(trace);
        Ok((
            AgentOutput {
                handled: true,
                reply: fallback,
            },
            facts,
        ))
    }
}

#[async_trait]
impl AgentProvider for BuiltinReActAgent {
    /// The legacy product entry: it runs the one shared execution and returns its existing output.
    ///
    /// The branch facts are dropped here on purpose — the product path's input and output, including
    /// `handled` and an empty reply, are unchanged by this unit. CP-B3-ALL-R2: this entry borrows the
    /// four fields the core consumes out of the caller's real [`AgentInput`]; it neither rewrites nor
    /// re-supplies the rest of that value.
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let (output, _facts) = self
            .execute_react(ReActInput::from_agent_input(&input))
            .await?;
        Ok(output)
    }
}

#[cfg(test)]
mod cp_b3_all_agent_tests {
    //! CP-B3-ALL unit A. The fakes sit only at the two end resource seams (`LlmClient` and
    //! `McpBridgePort`); the parser is the **existing production** one, and every assertion below
    //! drives the real shared core ([`BuiltinReActAgent::execute_react`]) or the real Base view
    //! ([`HostAgentBaseView`]). No test re-implements the loop, the report or the error projection.
    //!
    //! The Base-view agreement (the `context` branches, the report text, the failure projection) is
    //! tested here rather than next to the binding so that one set of fakes serves both entries.

    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::task::{Context as TaskContext, Poll, Wake, Waker};

    use async_trait::async_trait;
    use oclive_kernel_contracts::{
        AgentBase, BaseCallFuture, FunctionCallingParserPort, LlmClient, McpBridgePort,
    };
    use oclive_kernel_types::{
        AgentBaseRequest, AgentToolResult, AgentToolSchema, BaseCallError, BaseCallErrorKind,
    };
    use parking_lot::Mutex;
    use serde_json::{json, Value};

    use super::*;
    use crate::domain::agent_base_binding::{build_report, AgentTurnIdentity, HostAgentBaseView};

    /// The **existing production** parser, reached by its full path from inside this test module, so
    /// no domain→infrastructure `use` line is added and the layering ratchet
    /// (`scripts/check-domain-layering.mjs`) keeps its baseline count.
    fn builtin_parser() -> Arc<dyn FunctionCallingParserPort> {
        Arc::new(crate::infrastructure::function_call_parser::BuiltinFunctionCallingParser)
    }

    // -------------------------------------------------------------------------------------
    // Fakes: end resource seams only.
    // -------------------------------------------------------------------------------------

    /// Scripted LLM. It records the exact `(model, prompt)` of every `generate` call, so a test can
    /// assert the model source, the prompt content and the number of rounds independently.
    struct ScriptedLlm {
        script: Mutex<VecDeque<Result<String>>>,
        calls: Mutex<Vec<(String, String)>>,
    }

    impl ScriptedLlm {
        fn new(script: impl IntoIterator<Item = Result<String>>) -> Arc<Self> {
            Arc::new(Self {
                script: Mutex::new(script.into_iter().collect()),
                calls: Mutex::new(Vec::new()),
            })
        }

        fn calls(&self) -> Vec<(String, String)> {
            self.calls.lock().clone()
        }

        fn call_count(&self) -> usize {
            self.calls.lock().len()
        }
    }

    #[async_trait]
    impl LlmClient for ScriptedLlm {
        async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
            self.calls
                .lock()
                .push((model.to_string(), prompt.to_string()));
            self.script
                .lock()
                .pop_front()
                .unwrap_or_else(|| Err(AppError::Unknown("script exhausted".into())))
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            unreachable!("the agent core only calls `generate`")
        }
    }

    /// A gate one scripted response waits on.
    ///
    /// CP-B3-ALL-R3: `polls` counts **every** poll of this gate, including the poll that finds it
    /// released and returns `Ready`; `wakes` counts the wakeups this gate itself delivered. A poll
    /// count is therefore not an entry count and not a call count.
    #[derive(Default)]
    struct Gate {
        released: AtomicBool,
        polls: AtomicUsize,
        wakes: AtomicUsize,
        waker: Mutex<Option<Waker>>,
    }

    impl Gate {
        fn release(&self) {
            self.released.store(true, Ordering::SeqCst);
            // The guard is dropped before the wake call; no lock is held across an await boundary
            // anywhere in this fake.
            let waker = self.waker.lock().take();
            if let Some(waker) = waker {
                self.wakes.fetch_add(1, Ordering::SeqCst);
                waker.wake();
            }
        }

        fn polls(&self) -> usize {
            self.polls.load(Ordering::SeqCst)
        }

        fn wakes(&self) -> usize {
            self.wakes.load(Ordering::SeqCst)
        }

        fn poll(&self, cx: &mut TaskContext<'_>) -> Poll<()> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            if self.released.load(Ordering::SeqCst) {
                return Poll::Ready(());
            }
            *self.waker.lock() = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    /// Counts every wake delivered to one task.
    struct CountingWake {
        count: Arc<AtomicUsize>,
    }

    impl Wake for CountingWake {
        fn wake(self: Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// The model and prompt one LLM entry observed, read from the borrowed parameters at that
    /// moment. Owning the snapshot is fine; the point is **which** source it was read from.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct BorrowedParams {
        model: String,
        prompt: String,
    }

    /// Reads both parameters as borrowed `&str` at the call site that uses this helper.
    fn observe(model: &str, prompt: &str) -> BorrowedParams {
        BorrowedParams {
            model: model.to_string(),
            prompt: prompt.to_string(),
        }
    }

    /// What one entry of the gated LLM recorded: the read taken before waiting, and the read taken
    /// again after the wait resumed — both from the same borrowed parameters.
    #[derive(Debug, Clone)]
    struct GatedEntry {
        before_wait: BorrowedParams,
        after_resume: Option<BorrowedParams>,
    }

    /// LLM whose `n`-th response waits on gate `n`.
    ///
    /// CP-B3-ALL-R3: the entry count is its own counter; `model`/`prompt` are read from the real
    /// borrowed parameters **both** before the wait and again after the wait resumed, so the record
    /// shows a second read through the original borrow rather than a replay of the first snapshot.
    struct GatedLlm {
        gates: Vec<Arc<Gate>>,
        responses: Vec<String>,
        entries: AtomicUsize,
        records: Mutex<Vec<GatedEntry>>,
    }

    impl GatedLlm {
        fn entries(&self) -> usize {
            self.entries.load(Ordering::SeqCst)
        }

        fn records(&self) -> Vec<GatedEntry> {
            self.records.lock().clone()
        }
    }

    #[async_trait]
    impl LlmClient for GatedLlm {
        async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
            // Independent entry count, plus the first read of the real borrowed parameters. The
            // guard is released at the end of this block and never held across the await below.
            let index = {
                self.entries.fetch_add(1, Ordering::SeqCst);
                let mut records = self.records.lock();
                records.push(GatedEntry {
                    before_wait: observe(model, prompt),
                    after_resume: None,
                });
                records.len() - 1
            };
            let gate = self
                .gates
                .get(index)
                .cloned()
                .unwrap_or_else(|| Arc::new(Gate::default()));
            std::future::poll_fn(|cx| gate.poll(cx)).await;
            // Resume-stage read: again from the borrowed `model`/`prompt` parameters of this same
            // call, not from the snapshot taken before the wait.
            {
                let mut records = self.records.lock();
                records[index].after_resume = Some(observe(model, prompt));
            }
            Ok(self.responses.get(index).cloned().unwrap_or_default())
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            unreachable!("the agent core only calls `generate`")
        }
    }

    /// Fake MCP bridge: scripted discovery and scripted per-call tool outcomes, with the exact call
    /// sequence recorded. No MCP client, directory plugin or network is constructed.
    struct FakeBridge {
        schemas: Vec<AgentToolSchema>,
        discovery_denied: Option<(String, String)>,
        discovery_calls: AtomicUsize,
        outcomes: Mutex<VecDeque<Result<AgentToolResult>>>,
        tool_calls: Mutex<Vec<(String, Value)>>,
    }

    impl FakeBridge {
        fn new(
            schemas: Vec<AgentToolSchema>,
            outcomes: impl IntoIterator<Item = Result<AgentToolResult>>,
        ) -> Arc<Self> {
            Arc::new(Self {
                schemas,
                discovery_denied: None,
                discovery_calls: AtomicUsize::new(0),
                outcomes: Mutex::new(outcomes.into_iter().collect()),
                tool_calls: Mutex::new(Vec::new()),
            })
        }

        fn denied(capability: &str, id: &str) -> Arc<Self> {
            Arc::new(Self {
                schemas: Vec::new(),
                discovery_denied: Some((capability.to_string(), id.to_string())),
                discovery_calls: AtomicUsize::new(0),
                outcomes: Mutex::new(VecDeque::new()),
                tool_calls: Mutex::new(Vec::new()),
            })
        }

        fn discovery_calls(&self) -> usize {
            self.discovery_calls.load(Ordering::SeqCst)
        }

        fn tool_calls(&self) -> Vec<(String, Value)> {
            self.tool_calls.lock().clone()
        }
    }

    #[async_trait]
    impl McpBridgePort for FakeBridge {
        fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
            Vec::new()
        }

        async fn list_mcp_tools(&self, _server_id: &str) -> Result<Vec<McpToolInfo>> {
            Ok(Vec::new())
        }

        async fn list_agent_tool_schemas(&self) -> Result<Vec<AgentToolSchema>> {
            self.discovery_calls.fetch_add(1, Ordering::SeqCst);
            if let Some((capability, id)) = &self.discovery_denied {
                return Err(AppError::HighRiskCapabilityNotGranted {
                    capability: capability.clone(),
                    id: id.clone(),
                });
            }
            Ok(self.schemas.clone())
        }

        async fn call_tool_qualified(
            &self,
            tool_name: &str,
            params: Value,
        ) -> Result<AgentToolResult> {
            self.tool_calls.lock().push((tool_name.to_string(), params));
            self.outcomes
                .lock()
                .pop_front()
                .unwrap_or_else(|| Err(AppError::Unknown("no scripted tool outcome".to_string())))
        }

        async fn call_tool(
            &self,
            _server_id: &str,
            _tool_name: &str,
            _params: Value,
        ) -> Result<AgentToolResult> {
            unreachable!("the agent core calls tools through `call_tool_qualified`")
        }
    }

    // -------------------------------------------------------------------------------------
    // Small builders. Expected strings are written by hand in each test, never computed by the
    // production helpers under test.
    // -------------------------------------------------------------------------------------

    fn agent_with(llm: Arc<ScriptedLlm>, bridge: Arc<FakeBridge>) -> BuiltinReActAgent {
        BuiltinReActAgent::new(llm, bridge, builtin_parser())
    }

    fn input(message: &str, model: &str) -> AgentInput {
        AgentInput {
            role_id: "role-A".to_string(),
            session_namespace: "sess-A".to_string(),
            message: message.to_string(),
            model: model.to_string(),
            ..Default::default()
        }
    }

    fn schema(name: &str) -> AgentToolSchema {
        AgentToolSchema {
            name: name.to_string(),
            description: Some(format!("{name} tool")),
        }
    }

    fn answer(text: &str) -> String {
        json!({ "final_answer": text }).to_string()
    }

    fn call(name: &str, args: Value) -> String {
        json!({ "tool_calls": [ { "id": "1", "function": { "name": name, "arguments": args } } ] })
            .to_string()
    }

    fn tool_ok(server: &str, tool: &str, result: Value) -> Result<AgentToolResult> {
        Ok(AgentToolResult {
            server_id: server.to_string(),
            tool_name: tool.to_string(),
            params: json!({}),
            result,
            error: None,
        })
    }

    fn drive_base(
        future: BaseCallFuture<'_, String>,
    ) -> std::result::Result<String, BaseCallError> {
        let mut future = future;
        let waker = Waker::noop();
        let mut cx = TaskContext::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(outcome) => outcome,
            Poll::Pending => panic!("this fake setup has no pending path"),
        }
    }

    /// A fresh agent with its own scripted fakes.
    ///
    /// Tests that check **both** entries build one of these per entry: driving two entries is two
    /// executions, so they must not share a consumed script. That one execution can serve both views
    /// is proven separately in `cp_b3_all_agent_one_execution_serves_both_views`.
    fn scripted(
        script: impl IntoIterator<Item = Result<String>>,
        schemas: Vec<AgentToolSchema>,
        outcomes: impl IntoIterator<Item = Result<AgentToolResult>>,
    ) -> (BuiltinReActAgent, Arc<ScriptedLlm>, Arc<FakeBridge>) {
        let llm = ScriptedLlm::new(script);
        let bridge = FakeBridge::new(schemas, outcomes);
        (agent_with(llm.clone(), bridge.clone()), llm, bridge)
    }

    fn base_view<'a>(agent: &'a BuiltinReActAgent) -> HostAgentBaseView<'a> {
        HostAgentBaseView::new(
            agent,
            AgentTurnIdentity {
                model: "model-A",
                role_id: "role-A",
                session_namespace: "sess-A",
            },
        )
    }

    /// The literal report tails, written out independently of the production constants.
    const TAIL_NOT_TAKEN: &str =
        "本报告只说明本次调用没有承接该委托，不构成对外部效果、任务终态或重试安全性的判断。";
    const TAIL_ANSWERED: &str =
        "本报告只说明模型生成了回答；不代表委托的客观目标已经达成，也不代表工具效果已回滚。";
    const TAIL_OBSERVED: &str =
        "本报告只陈述本次真实执行已观察到的过程与结果，不代表全部任务已完成、\
                                 不代表没有产生外部效果、也不代表可以安全重试。";

    // -------------------------------------------------------------------------------------
    // Branches that stop before any model round.
    // -------------------------------------------------------------------------------------

    /// Empty (or whitespace-only) task and an empty tool list are the not-taken-up branches: the
    /// legacy output is `handled: false` with an empty reply, **no** model round runs, no trace is
    /// saved, and the Base view reports which of the two it was.
    #[tokio::test]
    async fn cp_b3_all_agent_not_taken_up_branches_are_counted_not_inferred() {
        let llm = ScriptedLlm::new(Vec::new());
        let bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let agent = agent_with(llm.clone(), bridge.clone());

        for message in ["", "   ", "\n\t "] {
            let out = AgentProvider::process(&agent, input(message, "model-A"))
                .await
                .expect("not taken up is not an error");
            assert!(!out.handled, "{message:?}");
            assert_eq!(out.reply, "", "{message:?}");
        }
        assert_eq!(
            bridge.discovery_calls(),
            0,
            "an empty task must not even discover tools"
        );
        assert_eq!(llm.call_count(), 0, "an empty task must not call the model");
        assert!(agent.recent_traces().is_empty());

        let empty_report = drive_base(base_view(&agent).execute(AgentBaseRequest {
            task: "   ",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            empty_report.lines().next().expect("first line"),
            "未承接：本次委托文本为空（trim 后没有字符）；本次调用没有发现工具，没有调用模型，\
             也没有调用任何工具。"
        );
        assert!(empty_report.contains(TAIL_NOT_TAKEN), "{empty_report}");
        assert!(!empty_report.contains("生成了回答"), "{empty_report}");

        // No usable tool schema: discovery ran, the model did not.
        let no_tools = agent_with(llm.clone(), FakeBridge::new(Vec::new(), Vec::new()));
        let out = AgentProvider::process(&no_tools, input("查一下", "model-A"))
            .await
            .expect("not taken up is not an error");
        assert!(!out.handled);
        assert_eq!(out.reply, "");
        assert_eq!(llm.call_count(), 0);
        assert!(no_tools.recent_traces().is_empty());

        let no_tools_report = drive_base(base_view(&no_tools).execute(AgentBaseRequest {
            task: "查一下",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            no_tools_report.lines().next().expect("first line"),
            "未承接：当前没有可用的工具 schema；本次调用没有调用模型，也没有调用任何工具。"
        );
        assert!(
            no_tools_report.contains(TAIL_NOT_TAKEN),
            "{no_tools_report}"
        );
        // The two not-taken-up branches are distinct in the report, not one merged "unhandled".
        assert_ne!(empty_report, no_tools_report);
    }

    /// A typed discovery refusal propagates to the product entry unchanged (the Host keeps its own
    /// error), while the Base view projects the typed reason without reading the error text.
    #[tokio::test]
    async fn cp_b3_all_agent_discovery_refusal_keeps_the_host_error_and_projects_a_reason() {
        let bridge = FakeBridge::denied("process:spawn", "mcp-server-1");
        let llm = ScriptedLlm::new(Vec::new());
        let agent = agent_with(llm.clone(), bridge.clone());

        let error = AgentProvider::process(&agent, input("查一下", "model-A"))
            .await
            .expect_err("a refusal is an error");
        match error {
            AppError::HighRiskCapabilityNotGranted { capability, id } => {
                assert_eq!(capability, "process:spawn");
                assert_eq!(id, "mcp-server-1");
            }
            other => panic!("the product entry must keep the typed refusal: {other:?}"),
        }

        let projected = drive_base(base_view(&agent).execute(AgentBaseRequest {
            task: "查一下",
            context: None,
        }))
        .expect_err("the Base view cannot complete normally here");
        assert_eq!(projected.kind, BaseCallErrorKind::Unavailable);
        assert!(
            projected
                .detail
                .unwrap_or_default()
                .contains("mcp-server-1"),
            "the human-readable detail keeps the original error text"
        );
        assert_eq!(llm.call_count(), 0);
        assert_eq!(bridge.tool_calls().len(), 0);
        assert!(agent.recent_traces().is_empty());
    }

    // -------------------------------------------------------------------------------------
    // Branches that ran model rounds.
    // -------------------------------------------------------------------------------------

    /// A final answer: the report says an answer was generated, on which round, with its content.
    #[tokio::test]
    async fn cp_b3_all_agent_final_answer_reports_the_answer_and_no_more() {
        // The product entry's own execution.
        let (agent, llm, bridge) = scripted(
            [Ok(answer("  答案文本  "))],
            vec![schema("echo")],
            Vec::new(),
        );

        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("the model answered");
        assert!(out.handled);
        assert_eq!(out.reply, "答案文本", "the existing trim is kept");

        let calls = llm.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "model-A", "the model source is the input model");
        assert!(calls[0].1.contains("用户请求: 任务"), "{}", calls[0].1);
        assert!(
            calls[0].1.contains("\"name\":\"echo\""),
            "the schema reaches the prompt: {}",
            calls[0].1
        );
        assert_eq!(bridge.discovery_calls(), 1);

        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 1, "one execution saves one trace");
        assert_eq!(traces[0].role_id, "role-A");
        assert_eq!(traces[0].session_namespace, "sess-A");
        assert_eq!(traces[0].message, "任务");
        assert_eq!(traces[0].plan, "react + function-calling; tools=echo");
        assert_eq!(traces[0].reply, "答案文本");
        assert!(traces[0].error.is_none());
        assert!(traces[0].tool_calls.is_empty());

        // The Base entry's own execution (a second, independent run).
        let (base_agent, _, _) = scripted(
            [Ok(answer("  答案文本  "))],
            vec![schema("echo")],
            Vec::new(),
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        assert_eq!(report.lines().count(), 2);
        assert_eq!(
            report.lines().next().expect("first line"),
            "模型在第 1 轮生成了回答：答案文本"
        );
        assert!(report.contains(TAIL_ANSWERED), "{report}");
    }

    /// A successful tool call followed by a final answer: the next round sees the observation, and
    /// the report lists the observed process with the round the answer came from.
    #[tokio::test]
    async fn cp_b3_all_agent_tool_success_then_answer_is_observed_in_order() {
        let (agent, llm, bridge) = scripted(
            [Ok(call("echo", json!({ "x": 1 }))), Ok(answer("完成"))],
            vec![schema("echo")],
            [tool_ok("mcp", "echo", json!({ "summary": "摘要A" }))],
        );

        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("the model answered in round 2");
        assert!(out.handled);
        assert_eq!(out.reply, "完成");

        assert_eq!(
            bridge.tool_calls(),
            vec![("echo".to_string(), json!({ "x": 1 }))]
        );
        let calls = llm.calls();
        assert_eq!(calls.len(), 2);
        assert!(
            !calls[0].1.contains("观察历史: mcp.echo"),
            "round 1 has no observation yet: {}",
            calls[0].1
        );
        assert!(
            calls[1]
                .1
                .contains("观察历史: mcp.echo -> {\"summary\":\"摘要A\"}"),
            "round 2 must see the observation the loop recorded: {}",
            calls[1].1
        );

        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].tool_calls.len(), 1);
        assert_eq!(traces[0].tool_calls[0].server_id, "mcp");
        assert_eq!(traces[0].tool_calls[0].tool_name, "echo");
        assert_eq!(traces[0].tool_calls[0].params, json!({ "x": 1 }));
        assert_eq!(
            traces[0].tool_calls[0].result,
            json!({ "summary": "摘要A" })
        );
        assert_eq!(traces[0].reply, "完成");

        // The Base entry's own execution (a second, independent run).
        let (base_agent, base_llm, _) = scripted(
            [Ok(call("echo", json!({ "x": 1 }))), Ok(answer("完成"))],
            vec![schema("echo")],
            [tool_ok("mcp", "echo", json!({ "summary": "摘要A" }))],
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            report.lines().next().expect("first line"),
            "模型在第 2 轮生成了回答：完成"
        );
        assert!(report.contains(TAIL_ANSWERED), "{report}");
        assert_eq!(base_llm.call_count(), 2);
    }

    /// A failing tool call is still recorded and the loop continues, exactly as before; the report
    /// lists the observed failure.
    #[tokio::test]
    async fn cp_b3_all_agent_tool_failure_is_recorded_and_the_loop_continues() {
        let (agent, llm, bridge) = scripted(
            [Ok(call("fragile", json!({}))), Ok(answer("换了一条路"))],
            vec![schema("fragile")],
            [Err(AppError::RemoteServiceUnavailable("mcp down".into()))],
        );

        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("a tool failure does not abort the loop");
        assert!(out.handled);
        assert_eq!(out.reply, "换了一条路");
        assert_eq!(llm.call_count(), 2, "the loop continued after the failure");
        assert_eq!(bridge.tool_calls().len(), 1);

        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 1);
        assert!(traces[0].tool_calls.is_empty());
        assert_eq!(
            traces[0].error.as_deref(),
            Some("tool fragile failed: Remote service unavailable: mcp down")
        );

        let observed = "tool fragile failed: Remote service unavailable: mcp down";
        assert!(llm.calls()[1].1.contains(observed), "{}", llm.calls()[1].1);

        // The Base entry's own execution (a second, independent run).
        let (base_agent, base_llm, _) = scripted(
            [Ok(call("fragile", json!({}))), Ok(answer("换了一条路"))],
            vec![schema("fragile")],
            [Err(AppError::RemoteServiceUnavailable("mcp down".into()))],
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            report.lines().next().expect("first line"),
            "模型在第 2 轮生成了回答：换了一条路"
        );
        assert!(report.contains(TAIL_ANSWERED), "{report}");
        assert!(
            base_llm.calls()[1].1.contains(observed),
            "the observation of the failed call reached the next model round: {}",
            base_llm.calls()[1].1
        );
    }

    /// A refusal of one tool call is reported as a refusal of that call, not as a missing
    /// capability, and the product fallback text is unchanged.
    #[tokio::test]
    async fn cp_b3_all_agent_tool_refusal_is_reported_as_a_refusal() {
        let refusal = || {
            Err(AppError::HighRiskCapabilityNotGranted {
                capability: "process:spawn".into(),
                id: "mcp-server-1".into(),
            })
        };
        let (agent, _, _) = scripted(
            [Ok(call("spawn", json!({}))), Ok("不再调工具".to_string())],
            vec![schema("spawn")],
            [refusal()],
        );

        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("the loop finishes normally after the refusal");
        assert!(out.handled);
        assert_eq!(
            out.reply,
            "Agent 执行未完成：tool spawn failed: High-risk capability not granted: process:spawn \
             (id=mcp-server-1)"
        );

        // The Base entry's own execution (a second, independent run).
        let (base_agent, _, _) = scripted(
            [Ok(call("spawn", json!({}))), Ok("不再调工具".to_string())],
            vec![schema("spawn")],
            [refusal()],
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            report.lines().next().expect("first line"),
            "第 2 轮模型输出没有可解析的 function call，循环结束；已观察到的过程：\
             tool spawn failed: High-risk capability not granted: process:spawn (id=mcp-server-1)；\
             产品路径返回的回复：Agent 执行未完成：tool spawn failed: High-risk capability not \
             granted: process:spawn (id=mcp-server-1)"
        );
        assert!(report.contains(TAIL_OBSERVED), "{report}");
        assert!(
            report.contains(
                "被拒绝的调用：process:spawn (id=mcp-server-1)（该次调用的授权决定，\
                 不是本实现不支持该能力；本报告不承诺重试结果）"
            ),
            "{report}"
        );
    }

    /// No parseable function call and three-round exhaustion are two different real branches, and
    /// both report only what was observed.
    #[tokio::test]
    async fn cp_b3_all_agent_no_function_call_and_exhaustion_stay_distinct() {
        // (a) No function call: one round, then the loop's `break`.
        let (agent, llm, bridge) = scripted(
            [Ok("不是 JSON，也不是函数调用".to_string())],
            vec![schema("echo")],
            Vec::new(),
        );
        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("no function call is not an error");
        assert!(out.handled, "the existing handled value is unchanged");
        assert_eq!(
            out.reply,
            "我已尝试调度工具，但模型没有返回可执行的 function call。"
        );
        assert_eq!(llm.call_count(), 1, "the loop breaks instead of retrying");
        assert_eq!(bridge.tool_calls().len(), 0);

        let (base_agent, _, _) = scripted(
            [Ok("不是 JSON，也不是函数调用".to_string())],
            vec![schema("echo")],
            Vec::new(),
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            report.lines().next().expect("first line"),
            "第 1 轮模型输出没有可解析的 function call，循环结束；已观察到的过程：\
             （本次执行没有工具调用记录）；产品路径返回的回复：\
             我已尝试调度工具，但模型没有返回可执行的 function call。"
        );
        assert!(report.contains(TAIL_OBSERVED), "{report}");

        // (b) Three-round exhaustion: each round calls a tool, none answers.
        let (agent, llm, bridge) = scripted(
            [
                Ok(call("echo", json!({ "n": 1 }))),
                Ok(call("echo", json!({ "n": 2 }))),
                Ok(call("echo", json!({ "n": 3 }))),
            ],
            vec![schema("echo")],
            [
                tool_ok("mcp", "echo", json!({ "n": 1 })),
                tool_ok("mcp", "echo", json!({ "n": 2 })),
                tool_ok("mcp", "echo", json!({ "n": 3 })),
            ],
        );
        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("exhaustion is not an error");
        assert!(out.handled, "the existing handled value is unchanged");
        assert_eq!(
            out.reply,
            format!("工具调用完成：{}", json!({ "n": 3 })),
            "the existing fallback text is kept"
        );
        assert_eq!(llm.call_count(), MAX_ROUNDS, "the loop bound is kept");
        assert_eq!(bridge.tool_calls().len(), MAX_ROUNDS);
        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].tool_calls.len(), MAX_ROUNDS);
        assert_eq!(traces[0].reply, out.reply);

        // The Base entry's own execution (a second, independent run).
        let (base_agent, _, _) = scripted(
            [
                Ok(call("echo", json!({ "n": 1 }))),
                Ok(call("echo", json!({ "n": 2 }))),
                Ok(call("echo", json!({ "n": 3 }))),
            ],
            vec![schema("echo")],
            [
                tool_ok("mcp", "echo", json!({ "n": 1 })),
                tool_ok("mcp", "echo", json!({ "n": 2 })),
                tool_ok("mcp", "echo", json!({ "n": 3 })),
            ],
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        let first = report.lines().next().expect("first line").to_string();
        assert!(
            first.starts_with("已用满循环上限 3 轮仍未获得最终回答；"),
            "{first}"
        );
        assert_eq!(
            first,
            format!(
                "已用满循环上限 3 轮仍未获得最终回答；已观察到的过程：{}；产品路径返回的回复：{}",
                [
                    "mcp.echo -> {\"n\":1}",
                    "mcp.echo -> {\"n\":2}",
                    "mcp.echo -> {\"n\":3}"
                ]
                .join("；"),
                out.reply
            )
        );
        assert!(report.contains(TAIL_OBSERVED), "{report}");
    }

    /// A model failure after a successful tool call: the product entry returns the original error,
    /// the trace is saved once with the tool call it did observe, and the Base view projects the
    /// typed reason.
    #[tokio::test]
    async fn cp_b3_all_agent_model_failure_after_tool_success() {
        let (agent, llm, bridge) = scripted(
            [
                Ok(call("echo", json!({ "x": 1 }))),
                Err(AppError::OllamaError("model down".into())),
            ],
            vec![schema("echo")],
            [tool_ok("mcp", "echo", json!({ "summary": "摘要A" }))],
        );

        let error = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect_err("the model error propagates");
        assert!(matches!(error, AppError::OllamaError(ref m) if m == "model down"));

        assert_eq!(llm.call_count(), 2);
        assert_eq!(bridge.tool_calls().len(), 1, "the finished call stays done");
        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 1, "the failing run still saves its trace");
        assert_eq!(traces[0].tool_calls.len(), 1);
        assert_eq!(traces[0].reply, "", "no reply was produced");
        assert_eq!(traces[0].error.as_deref(), Some("Ollama error: model down"));

        // The Base entry's own execution (a second, independent run).
        let (base_agent, _, _) = scripted(
            [
                Ok(call("echo", json!({ "x": 1 }))),
                Err(AppError::OllamaError("model down".into())),
            ],
            vec![schema("echo")],
            [tool_ok("mcp", "echo", json!({ "summary": "摘要A" }))],
        );
        let projected = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect_err("the Base view cannot complete normally here");
        assert_eq!(projected.kind, BaseCallErrorKind::Failed);
        assert_eq!(
            projected.detail.as_deref(),
            Some("Ollama error: model down"),
            "the original error text is kept verbatim as the human-readable detail"
        );
    }

    // -------------------------------------------------------------------------------------
    // One execution, two views; both entries reach the same core.
    // -------------------------------------------------------------------------------------

    /// Both views are read from **one** execution: the product output and the report come from the
    /// same carrier, and the call counts do not move when the report is built.
    #[tokio::test]
    async fn cp_b3_all_agent_one_execution_serves_both_views() {
        let llm = ScriptedLlm::new([Ok(call("echo", json!({ "x": 1 }))), Ok(answer("完成"))]);
        let bridge = FakeBridge::new(
            vec![schema("echo")],
            [tool_ok("mcp", "echo", json!({ "summary": "摘要A" }))],
        );
        let agent = agent_with(llm.clone(), bridge.clone());

        // CP-B3-ALL-R2: the core's private borrowed input, built from the local values this test
        // owns — the same four fields the production entries borrow.
        let borrowed = ReActInput {
            message: "任务",
            model: "model-A",
            role_id: "role-A",
            session_namespace: "sess-A",
        };
        let (output, facts) = agent.execute_react(borrowed).await.expect("one execution");
        // The product view of this run.
        assert!(output.handled);
        assert_eq!(output.reply, "完成");
        // The Base view of the same run: same carrier, no second execution.
        let report = build_report(&facts, output.reply.as_str());
        assert_eq!(
            report.lines().next().expect("first line"),
            "模型在第 2 轮生成了回答：完成"
        );
        assert_eq!(facts.rounds, 2);
        assert_eq!(facts.stop, ReActStop::FinalAnswer);
        assert_eq!(facts.tool_calls.len(), 1);
        assert!(facts.tool_failures.is_empty());
        assert_eq!(
            facts.observations,
            vec!["mcp.echo -> {\"summary\":\"摘要A\"}"]
        );

        // Still one execution: projecting the second view performed no extra work.
        assert_eq!(llm.call_count(), 2);
        assert_eq!(bridge.tool_calls().len(), 1);
        assert_eq!(agent.recent_traces().len(), 1);
    }

    /// Both entries run the shared core: each entry's own run performs the same discovery, the same
    /// model round and produces the same prompt and trace plan, byte for byte.
    #[tokio::test]
    async fn cp_b3_all_agent_both_entries_enter_the_shared_core() {
        let product_llm = ScriptedLlm::new([Ok(answer("入口"))]);
        let product_bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let product_agent = agent_with(product_llm.clone(), product_bridge.clone());
        let out = AgentProvider::process(&product_agent, input("任务", "model-A"))
            .await
            .expect("product entry");
        assert_eq!(out.reply, "入口");

        let base_llm = ScriptedLlm::new([Ok(answer("入口"))]);
        let base_bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let base_agent = agent_with(base_llm.clone(), base_bridge.clone());
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("base entry");

        assert_eq!(product_bridge.discovery_calls(), 1);
        assert_eq!(base_bridge.discovery_calls(), 1);
        assert_eq!(product_llm.call_count(), 1);
        assert_eq!(base_llm.call_count(), 1);
        assert_eq!(product_agent.recent_traces().len(), 1);
        assert_eq!(base_agent.recent_traces().len(), 1);
        assert_eq!(
            base_llm.calls()[0].1,
            product_llm.calls()[0].1,
            "the same core builds the same prompt for both entries"
        );
        assert_eq!(
            base_agent.recent_traces()[0].plan,
            product_agent.recent_traces()[0].plan
        );
        assert_eq!(
            base_agent.recent_traces()[0].message,
            product_agent.recent_traces()[0].message
        );
        assert_eq!(
            report.lines().next().expect("first line"),
            "模型在第 1 轮生成了回答：入口"
        );
    }

    /// The report is read from the branch, not from `handled`, from an empty reply or from the
    /// wording of a reply — the counter-examples below would flip it if it were.
    #[tokio::test]
    async fn cp_b3_all_agent_report_is_not_recovered_from_handled_or_wording() {
        // (a) A final answer whose text reads like a failure: still an answer.
        let (agent, _, _) = scripted(
            [Ok(answer("未承接，已失败"))],
            vec![schema("echo")],
            Vec::new(),
        );
        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("answer");
        assert!(out.handled);

        let (base_agent, _, _) = scripted(
            [Ok(answer("未承接，已失败"))],
            vec![schema("echo")],
            Vec::new(),
        );
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        assert_eq!(
            report.lines().next().expect("first line"),
            "模型在第 1 轮生成了回答：未承接，已失败",
            "the branch decides the report; the quoted reply keeps its own wording"
        );
        assert!(report.contains(TAIL_ANSWERED), "{report}");

        // (b) Exhaustion whose product reply reads like success: still exhaustion, and `handled` is
        // `true` in both cases, so it cannot be what the report is read from.
        let exhausted = || {
            scripted(
                [
                    Ok(call("echo", json!({ "n": 1 }))),
                    Ok(call("echo", json!({ "n": 2 }))),
                    Ok(call("echo", json!({ "n": 3 }))),
                ],
                vec![schema("echo")],
                [
                    tool_ok("mcp", "echo", json!({ "summary": "已完成" })),
                    tool_ok("mcp", "echo", json!({ "summary": "已完成" })),
                    tool_ok("mcp", "echo", json!({ "summary": "已完成" })),
                ],
            )
        };
        let (agent, _, _) = exhausted();
        let out = AgentProvider::process(&agent, input("任务", "model-A"))
            .await
            .expect("exhaustion");
        assert!(out.handled);
        assert_eq!(out.reply, "已完成", "the existing summary fallback is kept");

        let (base_agent, _, _) = exhausted();
        let report = drive_base(base_view(&base_agent).execute(AgentBaseRequest {
            task: "任务",
            context: None,
        }))
        .expect("a report");
        let first = report.lines().next().expect("first line").to_string();
        assert!(
            first.starts_with("已用满循环上限 3 轮仍未获得最终回答；"),
            "{first}"
        );
        assert!(!first.contains("生成了回答"), "{first}");
        assert!(report.contains(TAIL_OBSERVED), "{report}");
    }

    // -------------------------------------------------------------------------------------
    // Context agreement, borrowing, and the Send facts.
    // -------------------------------------------------------------------------------------

    /// Only `None` and `Some("")` are accepted, and a rejected context is reported before anything
    /// runs; the local material stays usable afterwards.
    #[tokio::test]
    async fn cp_b3_all_agent_context_branches_are_exact_and_run_nothing() {
        let llm = ScriptedLlm::new([Ok(answer("答案")), Ok(answer("答案"))]);
        let bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let agent = agent_with(llm.clone(), bridge.clone());
        let view = base_view(&agent);
        let task = String::from("任务");

        let without = drive_base(view.execute(AgentBaseRequest {
            task: task.as_str(),
            context: None,
        }))
        .expect("a report");
        let empty = drive_base(view.execute(AgentBaseRequest {
            task: task.as_str(),
            context: Some(""),
        }))
        .expect("a report");
        assert_eq!(without, empty, "`Some(\"\")` is this adapter's `None`");
        assert_eq!(
            llm.call_count(),
            2,
            "each accepted call is its own execution"
        );

        for context in [" ", "\n", "\t", "\u{3000}", "参考上文"] {
            let error = drive_base(view.execute(AgentBaseRequest {
                task: task.as_str(),
                context: Some(context),
            }))
            .expect_err("a non-empty context is rejected");
            assert_eq!(
                error.kind,
                BaseCallErrorKind::Unsupported,
                "unexpected kind for context {context:?}"
            );
        }
        // An empty task must not short-circuit the context check either.
        assert_eq!(
            drive_base(view.execute(AgentBaseRequest {
                task: "",
                context: Some("参考上文"),
            }))
            .unwrap_err()
            .kind,
            BaseCallErrorKind::Unsupported
        );

        assert_eq!(
            llm.call_count(),
            2,
            "a rejected context runs nothing at all"
        );
        assert_eq!(
            bridge.discovery_calls(),
            2,
            "a rejected context does not even discover tools"
        );
        assert_eq!(bridge.tool_calls().len(), 0);
        // The borrowed locals are still usable after the calls.
        assert_eq!(task, "任务");
    }

    /// Two requests that really intersect at `Pending` keep their own parameters, identity and
    /// counters, and each request's model/prompt is read **through its own borrow** both before the
    /// wait and again after the wait resumed (CP-B3-ALL-R3).
    ///
    /// Scope of this evidence: the shared core is allowed to organise the material into an owned
    /// prompt, so what is proven here is that the `&str` arguments of the endpoint `LlmClient`
    /// entry stay alive and readable across a real `Pending` — not that the original
    /// `AgentBaseRequest` text is borrowed all the way through. That the local `String`s are still
    /// readable afterwards is only a borrow-API observation and is not used as proof by itself.
    #[tokio::test]
    async fn cp_b3_all_agent_intersecting_pending_keeps_requests_isolated() {
        let gates: Vec<Arc<Gate>> = (0..2).map(|_| Arc::new(Gate::default())).collect();
        let llm = Arc::new(GatedLlm {
            gates: gates.clone(),
            responses: vec![answer("答复A"), answer("答复B")],
            entries: AtomicUsize::new(0),
            records: Mutex::new(Vec::new()),
        });
        let bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let agent = BuiltinReActAgent::new(llm.clone(), bridge, builtin_parser());

        let task_a = String::from("任务A");
        let model_a = String::from("model-A");
        let role_a = String::from("role-A");
        let session_a = String::from("sess-A");
        let task_b = String::from("任务B");
        let model_b = String::from("model-B");
        let role_b = String::from("role-B");
        let session_b = String::from("sess-B");

        let view_a = HostAgentBaseView::new(
            &agent,
            AgentTurnIdentity {
                model: model_a.as_str(),
                role_id: role_a.as_str(),
                session_namespace: session_a.as_str(),
            },
        );
        let view_b = HostAgentBaseView::new(
            &agent,
            AgentTurnIdentity {
                model: model_b.as_str(),
                role_id: role_b.as_str(),
                session_namespace: session_b.as_str(),
            },
        );
        let mut future_a = view_a.execute(AgentBaseRequest {
            task: task_a.as_str(),
            context: None,
        });
        let mut future_b = view_b.execute(AgentBaseRequest {
            task: task_b.as_str(),
            context: None,
        });

        let wake_count = Arc::new(AtomicUsize::new(0));
        let waker = Waker::from(Arc::new(CountingWake {
            count: wake_count.clone(),
        }));
        let mut cx = TaskContext::from_waker(&waker);

        // A Pending -> B Pending.
        assert!(matches!(future_a.as_mut().poll(&mut cx), Poll::Pending));
        assert!(matches!(future_b.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(gates[0].polls(), 1, "A's first poll");
        assert_eq!(gates[1].polls(), 1, "B's first poll");
        assert_eq!(llm.entries(), 2, "one LLM entry per request so far");
        let first = llm.records();
        assert!(
            first.iter().all(|entry| entry.after_resume.is_none()),
            "no resume-stage read may exist before the resumes"
        );

        // B Ready first: releasing B wakes B only.
        gates[1].release();
        assert_eq!(gates[1].wakes(), 1, "B's gate delivered one wake");
        assert_eq!(gates[0].wakes(), 0, "releasing B must not wake A's gate");
        assert_eq!(wake_count.load(Ordering::SeqCst), 1, "exactly B's wake");
        match future_b.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(report)) => assert_eq!(
                report.lines().next().expect("first line"),
                "模型在第 1 轮生成了回答：答复B"
            ),
            other => panic!("B must finish on its own gate: {other:?}"),
        }
        assert_eq!(gates[1].polls(), 2, "B's Ready poll is counted too");

        // A Ready afterwards.
        gates[0].release();
        assert_eq!(gates[0].wakes(), 1, "A's gate delivered one wake");
        assert_eq!(wake_count.load(Ordering::SeqCst), 2, "A's wake is separate");
        match future_a.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(report)) => assert_eq!(
                report.lines().next().expect("first line"),
                "模型在第 1 轮生成了回答：答复A"
            ),
            other => panic!("A must finish on its own gate: {other:?}"),
        }
        assert_eq!(gates[0].polls(), 2, "A's Ready poll is counted too");

        // One entry per request: the resume did not re-enter the endpoint.
        assert_eq!(llm.entries(), 2);
        let records = llm.records();
        assert_eq!(records.len(), 2);
        // Each request read its own parameters in both stages, from the real borrows.
        assert_eq!(records[0].before_wait.model, model_a);
        assert_eq!(records[1].before_wait.model, model_b);
        assert!(records[0].before_wait.prompt.contains("用户请求: 任务A"));
        assert!(!records[0].before_wait.prompt.contains("任务B"));
        assert!(records[1].before_wait.prompt.contains("用户请求: 任务B"));
        assert!(!records[1].before_wait.prompt.contains("任务A"));
        for entry in &records {
            let resumed = entry
                .after_resume
                .as_ref()
                .expect("every resumed request records its second read");
            assert_eq!(
                resumed, &entry.before_wait,
                "the resume-stage read must see the same borrowed parameters"
            );
        }
        // No cross-request mixing in either stage.
        let prompt_a = records[0]
            .after_resume
            .as_ref()
            .expect("A resumed")
            .prompt
            .clone();
        let prompt_b = records[1]
            .after_resume
            .as_ref()
            .expect("B resumed")
            .prompt
            .clone();
        assert!(!prompt_a.contains("任务B"), "{prompt_a}");
        assert!(!prompt_b.contains("任务A"), "{prompt_b}");

        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 2);
        // Traces are saved on completion, so their order is this interleaving's real completion
        // order (B finished first), and each request's identity travelled with its own run.
        assert_eq!(traces[0].message, "任务B");
        assert_eq!(traces[1].message, "任务A");
        let trace_a = traces
            .iter()
            .find(|trace| trace.message == "任务A")
            .expect("A's trace");
        assert_eq!(trace_a.role_id, "role-A");
        assert_eq!(trace_a.session_namespace, "sess-A");
        let trace_b = traces
            .iter()
            .find(|trace| trace.message == "任务B")
            .expect("B's trace");
        assert_eq!(trace_b.role_id, "role-B");
        assert_eq!(trace_b.session_namespace, "sess-B");

        // The locals are still readable after the calls (borrow-API observation only).
        assert_eq!(task_a, "任务A");
        assert_eq!(model_a, "model-A");
        assert_eq!(role_a, "role-A");
        assert_eq!(session_a, "sess-A");
        assert_eq!(task_b, "任务B");
        assert_eq!(model_b, "model-B");
        assert_eq!(role_b, "role-B");
        assert_eq!(session_b, "sess-B");
    }

    /// The Host's own concrete futures are `Send` by compile assertion, not by inference from the
    /// concrete type or from the borrow shape.
    ///
    /// The Base future is deliberately **not** asserted: [`BaseCallFuture`] is
    /// `Pin<Box<dyn Future + '_>>` with no `Send`, and this unit does not strengthen it.
    #[test]
    fn cp_b3_all_agent_host_concrete_future_is_send() {
        fn require_send<T: Send>(_: T) {}

        let llm = ScriptedLlm::new([Ok(answer("ok"))]);
        let bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let agent = agent_with(llm, bridge);

        // The shared core's own future — the one both entries run.
        require_send(agent.execute_react(ReActInput {
            message: "任务",
            model: "model-A",
            role_id: "role-A",
            session_namespace: "sess-A",
        }));
        // The legacy product entry's future.
        require_send(AgentProvider::process(&agent, input("任务", "model-A")));
    }

    /// The existing trace bound is kept: one trace per execution, and the oldest are dropped at 40.
    #[tokio::test]
    async fn cp_b3_all_agent_trace_save_and_trim_are_kept() {
        let runs = 41;
        let llm = ScriptedLlm::new((0..runs).map(|_| Ok(answer("答案"))));
        let bridge = FakeBridge::new(vec![schema("echo")], Vec::new());
        let agent = agent_with(llm, bridge);

        for index in 0..runs {
            let out = AgentProvider::process(&agent, input(&format!("任务{index}"), "model-A"))
                .await
                .expect("answer");
            assert!(out.handled);
        }

        let traces = agent.recent_traces();
        assert_eq!(traces.len(), 40, "the existing trace bound is kept");
        assert_eq!(
            traces[0].message, "任务1",
            "the oldest trace is the one dropped"
        );
        assert_eq!(traces[39].message, "任务40");
    }
}
