//! A separate, bounded Host example for the portable minimal-role content.
//!
//! The Host chooses a technical ID, owns local or in-memory content, selects a
//! Prompt or six-Base implementations and in-memory generators, and decides when to call them.
//! It does not register a role in the reference `AppState` or define a mandatory
//! six-slot turn order. The immediate poller is valid only for these in-memory capabilities.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{
    AgentBase, BaseCallFuture, EmotionBase, EventBase, LlmBase, MemoryBase, PromptBase,
};
use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
use oclive_kernel_runtime::domain::base_emotion::KeywordEmotionBase;
use oclive_kernel_runtime::domain::base_event::LlmEventAnalyzer;
use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
use oclive_kernel_runtime::domain::minimal_role_consumer::{
    MinimalRoleBaseBindings, MinimalRoleBaseConsumer, MinimalRoleTextConsumer, MinimalRoleTextError,
};
use oclive_kernel_runtime::domain::minimal_role_prompt::MinimalRolePrompt;
use oclive_kernel_runtime::domain::prompt_assembler::BuiltinPromptAssembler;
use oclive_kernel_runtime::domain::query_memory::QueryMemoryRetrieval;
use oclive_kernel_runtime::domain::user_emotion_analyzer::BuiltinUserEmotionAnalyzer;
use oclive_kernel_types::{
    AgentBaseRequest, BaseCallError, BaseCallErrorKind, EmotionBaseRequest, EventBaseRequest,
    LlmBaseRequest, MemoryBaseRequest, MinimalRoleDefinition, PromptBaseRequest,
};
use oclive_validation::minimal_role_local_file::{
    load_minimal_role_local_file, LocalMinimalRoleSnapshot,
};
use oclive_validation::validate_minimal_role_definition;

const MAX_DEFINITION_BYTES: usize = 64 * 1024;
const MAX_ASSET_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOTAL_ASSET_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
enum HostError {
    InvalidTechnicalId,
    UnknownRole,
    InvalidLocalFiles,
    InvalidRoleDefinition,
    InvalidRoleAssets,
    Capability(BaseCallErrorKind),
    PendingCapability,
    UnexpectedOutput,
}

// The selected Prompt and Echo LLM are immediate. A production Host needs its own
// async executor, deadlines and resource policy; pending is a visible failure here.
fn poll_immediate<T>(mut future: BaseCallFuture<'_, T>) -> Result<T, HostError> {
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(Ok(value)) => Ok(value),
        Poll::Ready(Err(error)) => Err(HostError::Capability(error.kind)),
        Poll::Pending => Err(HostError::PendingCapability),
    }
}

#[derive(Default)]
struct EchoLlm {
    calls: Cell<usize>,
}

impl LlmBase for EchoLlm {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            Ok(format!("echo: {}", request.input))
        })
    }
}

// These are this example's synthetic input and task agreements, not Kernel defaults.
const DEMO_COUNT_TASK: &str = "请统计材料中 Unicode 标量值的个数";
const DEMO_EVENT_REPORT: &str = "本次输入表达了聊天意愿；不宣称事件已经发生。";
const DEMO_EVENT_REPLY: &str = "ANALYSIS\n本次输入表达了聊天意愿；不宣称事件已经发生。";

// A declared stand-in for Event's generator. It exercises the real analyzer's
// preparation and response projection, not model understanding or real inference.
struct ScriptedEventModel {
    reply: String,
    calls: Cell<usize>,
    input: RefCell<String>,
}

impl ScriptedEventModel {
    fn new(reply: &str) -> Self {
        Self {
            reply: reply.into(),
            calls: Cell::new(0),
            input: RefCell::default(),
        }
    }
}

impl LlmBase for ScriptedEventModel {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            *self.input.borrow_mut() = request.input.into();
            Ok(self.reply.clone())
        })
    }
}

// Input and result belong to this finite example operation, not a public role,
// turn, state or six-slot wire format. The caller owns the memory scope and task.
struct HostSixInput<'a> {
    user_text: &'a str,
    memory: MemoryBaseRequest<'a>,
    delegated: AgentBaseRequest<'a>,
    requirements: &'a str,
}

struct HostSixResult {
    memory: Vec<String>,
    emotion: Option<String>,
    event: Option<String>,
    agent: String,
    reply: String,
}

fn native_bindings<'a>(
    memory: &'a dyn MemoryBase,
    emotion: &'a dyn EmotionBase,
    event: &'a dyn EventBase,
    llm: &'a dyn LlmBase,
) -> MinimalRoleBaseBindings<'a> {
    MinimalRoleBaseBindings {
        memory,
        emotion,
        event,
        prompt: &BuiltinPromptAssembler,
        llm,
        agent: &ScalarCountAgent,
    }
}

// This requirement and JSON assembly agreement belong only to this example
// implementation. Quoting preserves decoded fragments, not output bytes or a
// downstream model's prompt-injection resistance.
const QUOTED_MATERIAL_REQUIREMENT: &str = "quote each material as a JSON string";

#[derive(Default)]
struct QuotedMaterialPrompt {
    calls: Cell<usize>,
}

impl PromptBase for QuotedMaterialPrompt {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            if request.requirements != QUOTED_MATERIAL_REQUIREMENT {
                return Err(BaseCallError {
                    kind: BaseCallErrorKind::Unsupported,
                    detail: Some(
                        "this example supports only the material quoting agreement".into(),
                    ),
                });
            }
            serde_json::to_string(request.materials).map_err(|error| BaseCallError {
                kind: BaseCallErrorKind::Failed,
                detail: Some(error.to_string()),
            })
        })
    }
}

// This adapter seam belongs to the example Host. It is not a new Kernel port or
// a requirement that other distros use the local file format.
trait HostContent {
    fn definition(&self) -> &MinimalRoleDefinition;
    fn asset_count(&self) -> usize;
    fn assets(&self) -> Box<dyn ExactSizeIterator<Item = (&str, &[u8])> + '_>;
}

impl HostContent for LocalMinimalRoleSnapshot {
    fn definition(&self) -> &MinimalRoleDefinition {
        LocalMinimalRoleSnapshot::definition(self)
    }

    fn asset_count(&self) -> usize {
        LocalMinimalRoleSnapshot::assets(self).len()
    }

    fn assets(&self) -> Box<dyn ExactSizeIterator<Item = (&str, &[u8])> + '_> {
        Box::new(LocalMinimalRoleSnapshot::assets(self))
    }
}

struct MemoryContent {
    definition: MinimalRoleDefinition,
    asset_bytes: Vec<Vec<u8>>,
}

impl HostContent for MemoryContent {
    fn definition(&self) -> &MinimalRoleDefinition {
        &self.definition
    }

    fn asset_count(&self) -> usize {
        self.asset_bytes.len()
    }

    fn assets(&self) -> Box<dyn ExactSizeIterator<Item = (&str, &[u8])> + '_> {
        Box::new(
            self.definition
                .visual_assets
                .iter()
                .zip(&self.asset_bytes)
                .map(|(reference, bytes)| (reference.as_str(), bytes.as_slice())),
        )
    }
}

struct MinimalHost<L, C> {
    technical_id: String,
    content: C,
    llm: L,
}

impl<L: LlmBase> MinimalHost<L, LocalMinimalRoleSnapshot> {
    fn load(
        asset_root: &Path,
        definition_reference: &str,
        technical_id: &str,
        llm: L,
    ) -> Result<Self, HostError> {
        if technical_id.trim().is_empty() {
            return Err(HostError::InvalidTechnicalId);
        }
        let snapshot = load_minimal_role_local_file(
            asset_root,
            definition_reference,
            MAX_DEFINITION_BYTES,
            MAX_ASSET_BYTES,
            MAX_TOTAL_ASSET_BYTES,
        )
        .map_err(|_| HostError::InvalidLocalFiles)?;
        Self::new(technical_id, snapshot, llm)
    }
}

impl<L: LlmBase, C: HostContent> MinimalHost<L, C> {
    fn new(technical_id: &str, content: C, llm: L) -> Result<Self, HostError> {
        if technical_id.trim().is_empty() {
            return Err(HostError::InvalidTechnicalId);
        }
        validate_minimal_role_definition(content.definition())
            .map_err(|_| HostError::InvalidRoleDefinition)?;
        let mut assets = content.assets();
        if content.asset_count() != content.definition().visual_assets.len()
            || assets.len() != content.definition().visual_assets.len()
            || content
                .definition()
                .visual_assets
                .iter()
                .zip(&mut assets)
                .any(|(expected, (actual, bytes))| expected != actual || bytes.is_empty())
        {
            return Err(HostError::InvalidRoleAssets);
        }
        drop(assets);
        Ok(Self {
            technical_id: technical_id.to_owned(),
            content,
            llm,
        })
    }

    fn reply(
        &self,
        requested_role_id: &str,
        user_text: &str,
        requirements: &str,
    ) -> Result<String, HostError> {
        self.reply_using_prompt(requested_role_id, user_text, requirements, None)
    }

    fn reply_with_prompt(
        &self,
        requested_role_id: &str,
        user_text: &str,
        requirements: &str,
        prompt: &dyn PromptBase,
    ) -> Result<String, HostError> {
        self.reply_using_prompt(requested_role_id, user_text, requirements, Some(prompt))
    }

    fn reply_using_prompt(
        &self,
        requested_role_id: &str,
        user_text: &str,
        requirements: &str,
        selected_prompt: Option<&dyn PromptBase>,
    ) -> Result<String, HostError> {
        if requested_role_id != self.technical_id {
            return Err(HostError::UnknownRole);
        }
        let materials = ["User: ", user_text];
        let request = PromptBaseRequest {
            materials: &materials,
            requirements,
        };
        if let Some(prompt) = selected_prompt {
            let consumer =
                MinimalRoleTextConsumer::new(self.content.definition(), prompt, &self.llm)
                    .map_err(|_| HostError::InvalidRoleDefinition)?;
            // The shared operation retains the full phase/error. This example's
            // existing narrow HostError deliberately reports only its reason kind.
            return poll_immediate(Box::pin(async {
                consumer
                    .generate(request)
                    .await
                    .map_err(|error| match error {
                        MinimalRoleTextError::Prompt(reason)
                        | MinimalRoleTextError::Llm(reason) => reason,
                    })
            }));
        }
        let prompt = MinimalRolePrompt::new(self.content.definition())
            .map_err(|_| HostError::InvalidRoleDefinition)?;
        let prepared = poll_immediate(prompt.assemble(request))?;
        poll_immediate(self.llm.generate(LlmBaseRequest { input: &prepared }))
    }

    fn visual_assets(&self) -> Box<dyn ExactSizeIterator<Item = (&str, &[u8])> + '_> {
        self.content.assets()
    }

    // One explicitly chosen Host graph. It is not the common consumer's policy:
    // other Hosts can use other dependencies, tasks and legitimately bound resources.
    fn reply_with_bases(
        &self,
        requested_role_id: &str,
        input: HostSixInput<'_>,
        bindings: MinimalRoleBaseBindings<'_>,
    ) -> Result<HostSixResult, HostError> {
        if requested_role_id != self.technical_id {
            return Err(HostError::UnknownRole);
        }
        let consumer = MinimalRoleBaseConsumer::new(self.content.definition(), bindings)
            .map_err(|_| HostError::InvalidRoleDefinition)?;
        let memory = poll_immediate(MemoryBase::retrieve(&consumer, input.memory))?;
        let emotion = poll_immediate(EmotionBase::analyze(
            &consumer,
            EmotionBaseRequest {
                material: input.user_text,
                context: None,
            },
        ))?;
        let event = poll_immediate(EventBase::analyze(
            &consumer,
            EventBaseRequest {
                material: input.user_text,
                context: emotion.as_deref(),
            },
        ))?;
        let agent = poll_immediate(AgentBase::execute(&consumer, input.delegated))?;
        let mut materials = vec!["User: ", input.user_text, "\nMemory:\n"];
        materials.extend(memory.iter().map(String::as_str));
        if let Some(report) = &emotion {
            materials.extend(["\nEmotion analysis:\n", report.as_str()]);
        }
        if let Some(report) = &event {
            materials.extend(["\nEvent analysis:\n", report.as_str()]);
        }
        materials.extend(["\nDelegated task report:\n", agent.as_str()]);
        let prepared = poll_immediate(PromptBase::assemble(
            &consumer,
            PromptBaseRequest {
                materials: &materials,
                requirements: input.requirements,
            },
        ))?;
        let reply = poll_immediate(LlmBase::generate(
            &consumer,
            LlmBaseRequest { input: &prepared },
        ))?;
        Ok(HostSixResult {
            memory,
            emotion,
            event,
            agent,
            reply,
        })
    }
}

fn run_demo() -> Result<(), HostError> {
    let directory = tempfile::tempdir().map_err(|_| HostError::InvalidLocalFiles)?;
    std::fs::write(
        directory.path().join("content.json"),
        r#"{"persona_prompt":"A curious guide.","visual_assets":["portrait.bin"]}"#,
    )
    .map_err(|_| HostError::InvalidLocalFiles)?;
    std::fs::write(directory.path().join("portrait.bin"), b"synthetic-portrait")
        .map_err(|_| HostError::InvalidLocalFiles)?;

    let host = MinimalHost::load(
        directory.path(),
        "content.json",
        "host-owned-guide-id",
        EchoLlm::default(),
    )?;
    let answer = host.reply("host-owned-guide-id", "Hello.", "")?;
    if answer != "echo: 【角色设定】\nA curious guide.\n\n【输入材料】\nUser: Hello."
        || host.visual_assets().len() != 1
        || host.llm.calls.get() != 1
    {
        return Err(HostError::UnexpectedOutput);
    }
    let memory_host = MinimalHost::new(
        "memory-host-id",
        MemoryContent {
            definition: MinimalRoleDefinition {
                persona_prompt: "A guide from another distro.".into(),
                visual_assets: vec!["memory:portrait".into()],
            },
            asset_bytes: vec![b"synthetic-memory-portrait".to_vec()],
        },
        EchoLlm::default(),
    )?;
    if memory_host.reply("memory-host-id", "Hello.", "")?
        != "echo: 【角色设定】\nA guide from another distro.\n\n【输入材料】\nUser: Hello."
        || memory_host.visual_assets().len() != 1
        || memory_host.llm.calls.get() != 1
    {
        return Err(HostError::UnexpectedOutput);
    }
    let prompt = QuotedMaterialPrompt::default();
    let quoted_reply = memory_host.reply_with_prompt(
        "memory-host-id",
        "Hello again.",
        QUOTED_MATERIAL_REQUIREMENT,
        &prompt,
    )?;
    let quoted_input = quoted_reply
        .strip_prefix("echo: ")
        .ok_or(HostError::UnexpectedOutput)?;
    let fragments: Vec<String> =
        serde_json::from_str(quoted_input).map_err(|_| HostError::UnexpectedOutput)?;
    if fragments
        != [
            "【角色设定】\n",
            "A guide from another distro.",
            "\n\n【输入材料】\n",
            "User: ",
            "Hello again.",
        ]
        || prompt.calls.get() != 1
        || memory_host.llm.calls.get() != 2
    {
        return Err(HostError::UnexpectedOutput);
    }
    Ok(())
}

fn main() {
    if let Err(error) = run_demo().and_then(|()| run_six_base_demo()) {
        eprintln!("minimal role Host example failed: {error:?}");
        std::process::exit(1);
    }
    println!("minimal role Host example: local, in-memory, selected Prompt and native six-Base calls passed");
}

fn run_six_base_demo() -> Result<(), HostError> {
    let definition = MinimalRoleDefinition {
        persona_prompt: "A curious guide.".into(),
        visual_assets: vec!["portrait.bin".into()],
    };
    // Explicitly converted file and memory representations carry the same definition.
    let directory = tempfile::tempdir().map_err(|_| HostError::InvalidLocalFiles)?;
    std::fs::write(
        directory.path().join("content.json"),
        serde_json::to_vec(&definition).map_err(|_| HostError::InvalidRoleDefinition)?,
    )
    .map_err(|_| HostError::InvalidLocalFiles)?;
    std::fs::write(directory.path().join("portrait.bin"), b"synthetic-portrait")
        .map_err(|_| HostError::InvalidLocalFiles)?;
    let local_host = MinimalHost::load(
        directory.path(),
        "content.json",
        "local-six",
        EchoLlm::default(),
    )?;
    let memory_host = MinimalHost::new(
        "memory-six",
        MemoryContent {
            definition,
            asset_bytes: vec![b"synthetic-portrait".to_vec()],
        },
        EchoLlm::default(),
    )?;
    let candidates = ["她昨天说喜欢咖啡。", "另一个人喜欢茶。"];
    let user_text = "她说：我很开心，想聊咖啡。";
    let request = || HostSixInput {
        user_text,
        memory: MemoryBaseRequest {
            materials: &candidates,
            query: "咖啡",
        },
        delegated: AgentBaseRequest {
            task: DEMO_COUNT_TASK,
            context: Some("aé😀"),
        },
        requirements: "",
    };
    for alternate in [false, true] {
        let event_model = ScriptedEventModel::new(DEMO_EVENT_REPLY);
        let event = LlmEventAnalyzer::new(&event_model);
        let outcome = if alternate {
            memory_host.reply_with_bases(
                "memory-six",
                request(),
                native_bindings(
                    &QueryMemoryRetrieval,
                    &BuiltinUserEmotionAnalyzer,
                    &event,
                    &memory_host.llm,
                ),
            )?
        } else {
            local_host.reply_with_bases(
                "local-six",
                request(),
                native_bindings(
                    &KeywordMemoryBase,
                    &KeywordEmotionBase,
                    &event,
                    &local_host.llm,
                ),
            )?
        };
        if outcome.memory != [candidates[0]]
            || !outcome
                .emotion
                .as_ref()
                .is_some_and(|report| report.contains("开心"))
            || outcome.event.as_deref() != Some(DEMO_EVENT_REPORT)
            || outcome.agent != "本次完成了计数：3 个 Unicode 标量值。"
            || !outcome.reply.contains(&outcome.agent)
            || outcome.reply.matches("A curious guide.").count() != 1
            || event_model.calls.get() != 1
        {
            return Err(HostError::UnexpectedOutput);
        }
    }
    if local_host.llm.calls.get() != 1 || memory_host.llm.calls.get() != 1 {
        return Err(HostError::UnexpectedOutput);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("content.json"),
            r#"{"persona_prompt":"A curious guide.","visual_assets":["portrait.bin"]}"#,
        )
        .unwrap();
        std::fs::write(directory.path().join("portrait.bin"), b"synthetic-portrait").unwrap();
        directory
    }

    #[test]
    fn host_owns_identity_snapshot_and_one_capability_call() {
        let directory = files();
        let host = MinimalHost::load(
            directory.path(),
            "content.json",
            "host-owned-guide-id",
            EchoLlm::default(),
        )
        .unwrap();
        assert_eq!(host.visual_assets().next().unwrap().0, "portrait.bin");
        assert_eq!(
            host.visual_assets().next().unwrap().1,
            b"synthetic-portrait"
        );
        assert_eq!(
            host.reply("host-owned-guide-id", "Hello.", "").unwrap(),
            "echo: 【角色设定】\nA curious guide.\n\n【输入材料】\nUser: Hello."
        );
        assert_eq!(host.llm.calls.get(), 1);
    }

    #[test]
    fn memory_adapter_uses_the_same_host_path_without_a_local_role_file() {
        let content = MemoryContent {
            definition: MinimalRoleDefinition {
                persona_prompt: "A guide from another distro.".into(),
                visual_assets: vec!["memory:portrait".into()],
            },
            asset_bytes: vec![b"synthetic-memory-portrait".to_vec()],
        };
        let host = MinimalHost::new("memory-host-id", content, EchoLlm::default()).unwrap();
        assert_eq!(host.visual_assets().next().unwrap().0, "memory:portrait");
        assert_eq!(
            host.reply("memory-host-id", "Hello.", "").unwrap(),
            "echo: 【角色设定】\nA guide from another distro.\n\n【输入材料】\nUser: Hello."
        );
        assert_eq!(host.llm.calls.get(), 1);
    }

    #[test]
    fn memory_adapter_rejects_unpaired_or_empty_assets_before_host_construction() {
        let definition = MinimalRoleDefinition {
            persona_prompt: "A guide.".into(),
            visual_assets: vec!["memory:portrait".into()],
        };
        for asset_bytes in [vec![], vec![Vec::new()], vec![vec![1], vec![2]]] {
            assert!(matches!(
                MinimalHost::new(
                    "memory-host-id",
                    MemoryContent {
                        definition: definition.clone(),
                        asset_bytes,
                    },
                    EchoLlm::default(),
                ),
                Err(HostError::InvalidRoleAssets)
            ));
        }
    }

    #[test]
    fn wrong_identity_or_unsupported_requirement_never_reaches_llm() {
        let directory = files();
        let host = MinimalHost::load(
            directory.path(),
            "content.json",
            "host-owned-guide-id",
            EchoLlm::default(),
        )
        .unwrap();
        assert_eq!(
            host.reply("other-role", "Hello.", ""),
            Err(HostError::UnknownRole)
        );
        assert_eq!(
            host.reply("host-owned-guide-id", "Hello.", "extra policy"),
            Err(HostError::Capability(BaseCallErrorKind::Unsupported))
        );
        assert_eq!(host.llm.calls.get(), 0);
    }

    #[test]
    fn invalid_identity_or_missing_asset_does_not_construct_host() {
        let directory = files();
        assert!(matches!(
            MinimalHost::load(directory.path(), "content.json", " ", EchoLlm::default()),
            Err(HostError::InvalidTechnicalId)
        ));
        std::fs::remove_file(directory.path().join("portrait.bin")).unwrap();
        assert!(matches!(
            MinimalHost::load(
                directory.path(),
                "content.json",
                "host-owned-guide-id",
                EchoLlm::default()
            ),
            Err(HostError::InvalidLocalFiles)
        ));
    }

    #[test]
    fn host_selects_material_quoting_without_changing_the_default_agreement() {
        let directory = files();
        let host = MinimalHost::load(
            directory.path(),
            "content.json",
            "host-owned-guide-id",
            EchoLlm::default(),
        )
        .unwrap();
        let prompt = QuotedMaterialPrompt::default();
        let user_text = "  Hello, café.\r\n";
        let reply = host
            .reply_with_prompt(
                "host-owned-guide-id",
                user_text,
                QUOTED_MATERIAL_REQUIREMENT,
                &prompt,
            )
            .unwrap();
        let fragments: Vec<String> =
            serde_json::from_str(reply.strip_prefix("echo: ").unwrap()).unwrap();
        assert_eq!(
            fragments,
            [
                "【角色设定】\n",
                "A curious guide.",
                "\n\n【输入材料】\n",
                "User: ",
                user_text
            ]
        );
        assert_eq!(prompt.calls.get(), 1);
        assert_eq!(host.llm.calls.get(), 1);

        assert_eq!(
            host.reply(
                "host-owned-guide-id",
                user_text,
                QUOTED_MATERIAL_REQUIREMENT
            ),
            Err(HostError::Capability(BaseCallErrorKind::Unsupported))
        );
        assert_eq!(
            host.reply_with_prompt("other-id", user_text, QUOTED_MATERIAL_REQUIREMENT, &prompt),
            Err(HostError::UnknownRole)
        );
        assert_eq!(prompt.calls.get(), 1);
        assert_eq!(
            host.reply_with_prompt(
                "host-owned-guide-id",
                user_text,
                "unhandled requirement",
                &prompt
            ),
            Err(HostError::Capability(BaseCallErrorKind::Unsupported))
        );
        assert_eq!(prompt.calls.get(), 2);
        assert_eq!(host.llm.calls.get(), 1);
    }

    #[test]
    fn the_same_local_and_memory_content_runs_all_six_native_bases() {
        let directory = files();
        let local_host = MinimalHost::load(
            directory.path(),
            "content.json",
            "local-six",
            EchoLlm::default(),
        )
        .unwrap();
        let memory_host = MinimalHost::new(
            "memory-six",
            MemoryContent {
                definition: local_host.content.definition().clone(),
                asset_bytes: local_host
                    .visual_assets()
                    .map(|(_, bytes)| bytes.to_vec())
                    .collect(),
            },
            EchoLlm::default(),
        )
        .unwrap();
        assert_eq!(
            local_host.content.definition(),
            memory_host.content.definition()
        );
        assert_eq!(
            local_host.visual_assets().collect::<Vec<_>>(),
            memory_host.visual_assets().collect::<Vec<_>>()
        );
        let candidates = ["她昨天说喜欢咖啡。", "另一个人喜欢茶。"];
        let user = "她说：我很开心，想聊咖啡。";
        for alternate in [false, true] {
            let event_model = ScriptedEventModel::new(DEMO_EVENT_REPLY);
            let event = LlmEventAnalyzer::new(&event_model);
            let memory: &dyn MemoryBase = if alternate {
                &QueryMemoryRetrieval
            } else {
                &KeywordMemoryBase
            };
            let emotion: &dyn EmotionBase = if alternate {
                &BuiltinUserEmotionAnalyzer
            } else {
                &KeywordEmotionBase
            };
            let input = HostSixInput {
                user_text: user,
                memory: MemoryBaseRequest {
                    materials: &candidates,
                    query: "咖啡",
                },
                delegated: AgentBaseRequest {
                    task: DEMO_COUNT_TASK,
                    context: Some("aé😀"),
                },
                requirements: "",
            };
            let outcome = if alternate {
                memory_host.reply_with_bases(
                    "memory-six",
                    input,
                    native_bindings(memory, emotion, &event, &memory_host.llm),
                )
            } else {
                local_host.reply_with_bases(
                    "local-six",
                    input,
                    native_bindings(memory, emotion, &event, &local_host.llm),
                )
            }
            .unwrap();
            assert_eq!(outcome.memory, vec![candidates[0]]);
            assert!(outcome.emotion.as_ref().unwrap().contains("开心"));
            assert_eq!(outcome.event.as_deref(), Some(DEMO_EVENT_REPORT));
            assert_eq!(outcome.agent, "本次完成了计数：3 个 Unicode 标量值。");
            for material in [
                user,
                candidates[0],
                outcome.emotion.as_ref().unwrap(),
                DEMO_EVENT_REPORT,
                outcome.agent.as_str(),
            ] {
                assert!(outcome.reply.contains(material));
            }
            assert_eq!(outcome.reply.matches("A curious guide.").count(), 1);
            assert_eq!(event_model.calls.get(), 1);
            let event_input = event_model.input.borrow();
            assert!(event_input.contains(user));
            if alternate {
                assert_eq!(memory_host.llm.calls.get(), 1);
            } else {
                assert_eq!(local_host.llm.calls.get(), 1);
            }
        }
    }

    #[test]
    fn six_base_operation_rejects_the_wrong_technical_identity_before_generation() {
        let directory = files();
        let host =
            MinimalHost::load(directory.path(), "content.json", "six", EchoLlm::default()).unwrap();
        let event_model = ScriptedEventModel::new(DEMO_EVENT_REPLY);
        let event = LlmEventAnalyzer::new(&event_model);
        let result = host.reply_with_bases(
            "other",
            HostSixInput {
                user_text: "我很开心。",
                memory: MemoryBaseRequest {
                    materials: &[],
                    query: "",
                },
                delegated: AgentBaseRequest {
                    task: DEMO_COUNT_TASK,
                    context: Some(""),
                },
                requirements: "",
            },
            native_bindings(&KeywordMemoryBase, &KeywordEmotionBase, &event, &host.llm),
        );
        assert!(matches!(result, Err(HostError::UnknownRole)));
        assert_eq!(event_model.calls.get(), 0);
        assert_eq!(host.llm.calls.get(), 0);
    }

    #[test]
    fn real_event_agent_and_prompt_refusals_never_reach_the_reply_model() {
        for (event_reply, task, requirements, reason) in [
            (
                "invalid event protocol",
                DEMO_COUNT_TASK,
                "",
                BaseCallErrorKind::Failed,
            ),
            (
                DEMO_EVENT_REPLY,
                "perform an unsupported task",
                "",
                BaseCallErrorKind::Unsupported,
            ),
            (
                DEMO_EVENT_REPLY,
                DEMO_COUNT_TASK,
                "extra unsupported requirement",
                BaseCallErrorKind::Unsupported,
            ),
        ] {
            let directory = files();
            let host =
                MinimalHost::load(directory.path(), "content.json", "six", EchoLlm::default())
                    .unwrap();
            let event_model = ScriptedEventModel::new(event_reply);
            let event = LlmEventAnalyzer::new(&event_model);
            let result = host.reply_with_bases(
                "six",
                HostSixInput {
                    user_text: "我很开心。",
                    memory: MemoryBaseRequest {
                        materials: &[],
                        query: "",
                    },
                    delegated: AgentBaseRequest {
                        task,
                        context: Some("aé😀"),
                    },
                    requirements,
                },
                native_bindings(&KeywordMemoryBase, &KeywordEmotionBase, &event, &host.llm),
            );
            assert!(matches!(result, Err(HostError::Capability(kind)) if kind == reason));
            // Event generation already happened. Fail-fast is not zero work or rollback.
            assert_eq!(event_model.calls.get(), 1);
            assert_eq!(host.llm.calls.get(), 0);
        }
    }
}
