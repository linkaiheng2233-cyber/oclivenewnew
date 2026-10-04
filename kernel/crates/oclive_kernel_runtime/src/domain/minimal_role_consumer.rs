//! Shared minimal-role consumption through Host-selected Base capabilities.
//!
//! The Host supplies the current materials, requirements, and capability. This
//! Prompt consumer prepares the same persona/material sections as the reference
//! minimal prompt. The six-Base consumer exposes independent calls; the optional
//! text operation passes the selected Prompt's result to a bound LLM Base.
//! None chooses capabilities, resolves assets, loads a legacy
//! `Role`, owns Host state, or prescribes a six-slot turn order.

use oclive_kernel_contracts::{
    AgentBase, BaseCallFuture, EmotionBase, EventBase, LlmBase, MemoryBase, PromptBase,
};
use oclive_kernel_types::{
    AgentBaseRequest, BaseCallError, EmotionBaseRequest, EventBaseRequest, LlmBaseRequest,
    MemoryBaseRequest, MinimalRoleDefinition, PromptBaseRequest,
};
use oclive_validation::validate_minimal_role_definition;

const PERSONA_HEADING: &str = "【角色设定】\n";
const MATERIAL_HEADING: &str = "\n\n【输入材料】\n";

/// A borrowed consumer that prepares a minimal role for a selected Prompt Base.
///
/// The prepared request adds three fragments: a persona heading, the authored
/// persona, and a material heading. Every current material remains a distinct
/// fragment with its original bytes and order; requirements pass through unchanged.
/// The selected capability owns assembly and requirement handling, so the Host must
/// select an implementation whose published agreement serves the current purpose.
/// Headings are text delimiters, not a downstream prompt-injection guarantee.
///
/// Normal results (including empty text) and complete errors pass through without
/// fallback, retry, or post-processing. No stronger scheduling or cancellation
/// guarantee than the existing local [`BaseCallFuture`] contract is introduced.
pub struct MinimalRolePromptConsumer<'a> {
    definition: &'a MinimalRoleDefinition,
    prompt: &'a dyn PromptBase,
}

impl<'a> MinimalRolePromptConsumer<'a> {
    /// Validate logical role content and borrow the Host's selected capability.
    ///
    /// Asset references are validated as content only; this does not establish
    /// that the referenced assets exist or can be rendered by the Host.
    ///
    /// # Errors
    ///
    /// Returns the shared field/index diagnostics for invalid minimal-role content.
    pub fn new(
        definition: &'a MinimalRoleDefinition,
        prompt: &'a dyn PromptBase,
    ) -> Result<Self, Vec<String>> {
        validate_minimal_role_definition(definition)?;
        Ok(Self { definition, prompt })
    }
}

impl PromptBase for MinimalRolePromptConsumer<'_> {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        assemble_minimal_role_materials(self.definition, self.prompt, request)
    }
}

/// Six capabilities explicitly selected and borrowed by the caller.
///
/// This binding is for the optional six-Base consumer, not a requirement that every
/// Host construct all slots for every operation. Each implementation's published
/// agreement must serve the request the caller sends to it. No registry, capability
/// discovery, asset resolution, permission or state ownership is introduced here.
pub struct MinimalRoleBaseBindings<'a> {
    /// Selects from the caller's supplied memory material.
    pub memory: &'a dyn MemoryBase,
    /// Analyses the caller's material under the selected implementation's agreement.
    pub emotion: &'a dyn EmotionBase,
    /// Analyses the caller's described event without publishing it.
    pub event: &'a dyn EventBase,
    /// Assembles fragments prepared by the existing minimal-role Prompt consumer.
    pub prompt: &'a dyn PromptBase,
    /// Generates from the input the caller has already prepared.
    pub llm: &'a dyn LlmBase,
    /// Processes the caller's delegated task under its existing authorisation.
    pub agent: &'a dyn AgentBase,
}

/// An additive six-slot consumption surface outside the small Kernel.
///
/// The same minimal definition can be used with different Base implementations.
/// Only Prompt prepares the authored persona; other requests pass through unchanged.
/// The definition is not a hidden context bundle, a storage key, a permission grant,
/// or a requirement that each implementation understand a complete role format.
///
/// Each call invokes only its bound capability, without retry or post-processing.
/// In particular, LLM does not implicitly invoke Prompt, Event does not invoke
/// Emotion, and Agent's report does not grant effects. The caller owns dependencies,
/// scheduling, memory scope, resources, state application and invocation lifetime.
/// Normal empty results and full errors retain the existing Base semantics. Local
/// futures may borrow non-`Send` implementations; dropping one is not proof that
/// external execution or effects stopped.
pub struct MinimalRoleBaseConsumer<'a> {
    prompt: MinimalRolePromptConsumer<'a>,
    memory: &'a dyn MemoryBase,
    emotion: &'a dyn EmotionBase,
    event: &'a dyn EventBase,
    llm: &'a dyn LlmBase,
    agent: &'a dyn AgentBase,
}

impl<'a> MinimalRoleBaseConsumer<'a> {
    /// Validate the logical definition and borrow the six caller-bound capabilities.
    ///
    /// Validation performs no capability calls. Visual references are checked as
    /// content only; the caller still resolves and renders assets. The selected
    /// Prompt must accept prepared fragments rather than already adding this persona.
    ///
    /// # Errors
    /// Returns the existing field/index diagnostics for invalid minimal content.
    pub fn new(
        definition: &'a MinimalRoleDefinition,
        bindings: MinimalRoleBaseBindings<'a>,
    ) -> Result<Self, Vec<String>> {
        Ok(Self {
            prompt: MinimalRolePromptConsumer::new(definition, bindings.prompt)?,
            memory: bindings.memory,
            emotion: bindings.emotion,
            event: bindings.event,
            llm: bindings.llm,
            agent: bindings.agent,
        })
    }
}

impl MemoryBase for MinimalRoleBaseConsumer<'_> {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        self.memory.retrieve(request)
    }
}

impl EmotionBase for MinimalRoleBaseConsumer<'_> {
    fn analyze<'a>(
        &'a self,
        request: EmotionBaseRequest<'a>,
    ) -> BaseCallFuture<'a, Option<String>> {
        self.emotion.analyze(request)
    }
}

impl EventBase for MinimalRoleBaseConsumer<'_> {
    fn analyze<'a>(&'a self, request: EventBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>> {
        self.event.analyze(request)
    }
}

impl PromptBase for MinimalRoleBaseConsumer<'_> {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.prompt.assemble(request)
    }
}

impl LlmBase for MinimalRoleBaseConsumer<'_> {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.llm.generate(request)
    }
}

impl AgentBase for MinimalRoleBaseConsumer<'_> {
    fn execute<'a>(&'a self, request: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.agent.execute(request)
    }
}

/// A failure of one capability call in the optional basic text operation.
///
/// The phase identifies which bound capability did not complete normally; the
/// enclosed error preserves its complete reason and detail. This is not a Host
/// session/turn terminal state, effect ledger, retry permission or wire error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinimalRoleTextError {
    /// Prompt preparation/assembly did not complete; no LLM call was made.
    Prompt(BaseCallError),
    /// The LLM call did not complete normally; no effects are inferred.
    Llm(BaseCallError),
}

impl std::fmt::Display for MinimalRoleTextError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prompt(error) => write!(formatter, "minimal role Prompt: {error}"),
            Self::Llm(error) => write!(formatter, "minimal role LLM: {error}"),
        }
    }
}

impl std::error::Error for MinimalRoleTextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Prompt(error) | Self::Llm(error) => Some(error),
        }
    }
}

/// A reusable basic text operation outside the small Kernel's responsibilities.
///
/// The caller supplies logically valid role content and selects compatible Prompt
/// and LLM Base instances. This consumer reuses the existing role preparation,
/// makes one Prompt call and, only on normal completion, one LLM call with the
/// Prompt's verbatim output. The generated text is returned unchanged.
///
/// This explicit operation's causal dependency is not a mandatory six-stage
/// pipeline or a new slot contract. It performs no other slot calls, asset I/O,
/// identity/state management, resource binding, persistence, retry or fallback.
/// Normal empty results remain empty. Local futures may suspend and need not be
/// `Send`; the caller remains responsible for scheduling and cancellation, whose
/// effects on an external provider are not inferred here.
pub struct MinimalRoleTextConsumer<'a> {
    prompt: MinimalRolePromptConsumer<'a>,
    llm: &'a dyn LlmBase,
}

impl<'a> MinimalRoleTextConsumer<'a> {
    /// Validate minimal logical content and borrow the caller's bound capabilities.
    ///
    /// Asset references are checked as content only, not read or rendered. The
    /// Prompt must accept prepared fragments rather than already carrying this
    /// role's persona, and its agreement must serve the caller's requirements.
    ///
    /// # Errors
    /// Returns existing logical validation diagnostics without calling either Base.
    pub fn new(
        definition: &'a MinimalRoleDefinition,
        prompt: &'a dyn PromptBase,
        llm: &'a dyn LlmBase,
    ) -> Result<Self, Vec<String>> {
        Ok(Self {
            prompt: MinimalRolePromptConsumer::new(definition, prompt)?,
            llm,
        })
    }

    /// Prepare this call's role materials and generate using the two bound Bases.
    ///
    /// Material/requirement semantics are unchanged from the Prompt consumer. No
    /// Host-specific empty-user-input rule or completed-result quality assertion
    /// is introduced at this library boundary.
    ///
    /// # Errors
    /// Returns the failed phase with the provider's full error, without retry.
    pub async fn generate(
        &self,
        request: PromptBaseRequest<'_>,
    ) -> Result<String, MinimalRoleTextError> {
        let prepared = self
            .prompt
            .assemble(request)
            .await
            .map_err(MinimalRoleTextError::Prompt)?;
        self.llm
            .generate(LlmBaseRequest { input: &prepared })
            .await
            .map_err(MinimalRoleTextError::Llm)
    }
}

// The legacy prompt keeps its concrete storage (and existing auto traits) while
// sharing preparation with the consumer. Both constructors validate the role.
pub(super) fn assemble_minimal_role_materials<'a>(
    definition: &'a MinimalRoleDefinition,
    prompt: &'a dyn PromptBase,
    request: PromptBaseRequest<'a>,
) -> BaseCallFuture<'a, String> {
    Box::pin(async move {
        let mut fragments = Vec::with_capacity(request.materials.len() + 3);
        fragments.push(PERSONA_HEADING);
        fragments.push(definition.persona_prompt.as_str());
        fragments.push(MATERIAL_HEADING);
        fragments.extend_from_slice(request.materials);
        prompt
            .assemble(PromptBaseRequest {
                materials: &fragments,
                requirements: request.requirements,
            })
            .await
    })
}
