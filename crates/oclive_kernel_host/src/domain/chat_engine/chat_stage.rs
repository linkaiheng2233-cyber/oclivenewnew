//! Orchestration stage labels shared by `process_message` and co-present paths.

/// Kernel chat pipeline stage (tracing / OOCP / error prefixes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatStage {
    EnsureRoleRuntime,
    EnsureRoleLoaded,
    EnsureInteractionModeSeeded,
    ApplyUserLlmEnv,
    StartupHealth,
    AgentProcess,
    AgentMinimalResponse,
    SetUserPresenceScene,
    GetCurrentScene,
    GetInteractionMode,
    GetRemoteLifeEnabled,
    RemoteStub,
    RemoteLife,
    EventImpactFactor,
    MutablePersonality,
    CurrentPersonality,
    UserEmotionAnalyze,
    LoadRecentContext,
    ComplexEmotionResolveTurn,
    EventEstimate,
    LoadMemories,
    MemoryRank,
    ResolveUserRelationKey,
    RelationStateForIdentity,
    RelationStateGlobal,
    EnsureIdentityStatsRow,
    FavorabilityForIdentity,
    IdlePersonalityDecay,
    VirtualTimeMs,
    BuildPrompt,
    BotReplyEmotionAnalyze,
    GetCurrentEmotion,
    PortraitEmotionLlm,
    ApplyChatTurnAtomic,
    SetMutablePersonality,
    SetCoreDeltaPersonalityJsonNonProfile,
}

impl ChatStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EnsureRoleRuntime => "ensure_role_runtime",
            Self::EnsureRoleLoaded => "ensure_role_loaded",
            Self::EnsureInteractionModeSeeded => "ensure_interaction_mode_seeded",
            Self::ApplyUserLlmEnv => "apply_user_llm_env",
            Self::StartupHealth => "startup_health",
            Self::AgentProcess => "agent_process",
            Self::AgentMinimalResponse => "agent_minimal_response",
            Self::SetUserPresenceScene => "set_user_presence_scene",
            Self::GetCurrentScene => "get_current_scene",
            Self::GetInteractionMode => "get_interaction_mode",
            Self::GetRemoteLifeEnabled => "get_remote_life_enabled",
            Self::RemoteStub => "remote_stub",
            Self::RemoteLife => "remote_life",
            Self::EventImpactFactor => "event_impact_factor",
            Self::MutablePersonality => "mutable_personality",
            Self::CurrentPersonality => "current_personality",
            Self::UserEmotionAnalyze => "user_emotion_analyze",
            Self::LoadRecentContext => "load_recent_context",
            Self::ComplexEmotionResolveTurn => "complex_emotion_resolve_turn",
            Self::EventEstimate => "event_estimate",
            Self::LoadMemories => "load_memories",
            Self::MemoryRank => "memory_rank",
            Self::ResolveUserRelationKey => "resolve_user_relation_key",
            Self::RelationStateForIdentity => "relation_state_for_identity",
            Self::RelationStateGlobal => "relation_state_global",
            Self::EnsureIdentityStatsRow => "ensure_identity_stats_row",
            Self::FavorabilityForIdentity => "favorability_for_identity",
            Self::IdlePersonalityDecay => "idle_personality_decay",
            Self::VirtualTimeMs => "virtual_time_ms",
            Self::BuildPrompt => "build_prompt",
            Self::BotReplyEmotionAnalyze => "bot_reply_emotion_analyze",
            Self::GetCurrentEmotion => "get_current_emotion",
            Self::PortraitEmotionLlm => "portrait_emotion_llm",
            Self::ApplyChatTurnAtomic => "apply_chat_turn_atomic",
            Self::SetMutablePersonality => "set_mutable_personality",
            Self::SetCoreDeltaPersonalityJsonNonProfile => {
                "set_core_delta_personality_json_non_profile"
            }
        }
    }
}
