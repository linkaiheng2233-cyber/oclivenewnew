//! Prompt-assembly input (pure data structures).

use crate::models::{EventType, Memory, PersonalityVector, Role};

/// Generic prompt section injected before the reply-quality anchor footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptExtraSection<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

/// Input for the main-dialogue `build_prompt`, avoiding a long parameter list and call-site mismatches.
pub struct PromptInput<'a> {
    pub role: &'a Role,
    pub personality: &'a PersonalityVector,
    pub memories: &'a [Memory],
    pub user_input: &'a str,
    pub user_emotion: &'a str,
    /// Current user-identity key (consistent with manifest `user_relations` and the DB); if empty, the entire [User Identity] section is skipped.
    pub user_relation_id: &'a str,
    pub relation_hint: &'a str,
    pub relation_before: &'a str,
    pub favorability_before: f64,
    pub relation_preview: &'a str,
    pub favorability_preview: f64,
    pub event_type: &'a EventType,
    pub impact_factor: f64,
    pub scene_label: &'a str,
    /// Automatically assembled from the role pack's `description.txt` or `scene.json`; new scenes need no code changes.
    pub scene_detail: &'a str,
    pub topic_hint_line: &'a str,
    /// One line of virtual-time schedule inference; if empty it is skipped (does not change dialogue behavior when unconfigured).
    pub life_context_line: &'a str,
    /// Worldview knowledge snippet retrieved this turn; if empty the [Worldview Settings] section is skipped.
    pub worldview_snippet: &'a str,
    /// Full text of the "mutable personality profile" from the DB under persona-first mode; pass an empty string in `vector` mode.
    pub mutable_personality: &'a str,
    /// Merged "reply quality anchor" (engine default or `settings.json` override); injected before "User says".
    pub reply_quality_anchor: &'a str,
    /// The `narrative_hint` output by the builtin complex-emotion module in the previous turn; if empty the [Complex-Emotion Narrative Hint] section is skipped.
    pub previous_complex_emotion_narrative_hint: &'a str,
    /// Full User Identity Prompt Template body (host-loaded); when non-empty it replaces legacy `relation_hint` as the section body.
    pub user_identity_template: &'a str,
    /// Current User Identity Prompt Template id (audit / debug).
    pub user_identity_id: &'a str,
    /// Distro concise overlay (e.g. VS Code); injected inside the scene-constraint block when non-empty.
    pub host_prompt_overlay: &'a str,
    /// HostProfile `[state_expression]` hint for current favor tier; empty when unset.
    pub host_state_expression_hint: &'a str,
    /// Multi-turn relation transition hint from SessionCache; empty when inactive.
    pub relation_transition_hint: &'a str,
    /// Host-orchestrated extra sections rendered before the reply-quality anchor (ordered).
    pub extra_sections: &'a [PromptExtraSection<'a>],
    /// When set, replaces Tier0 `core_personality` injection (Wave D Deep capsule).
    pub persona_override: Option<&'a str>,
}
