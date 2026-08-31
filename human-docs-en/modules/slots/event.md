# Slot pack · `event` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/event.md`](../../../human-docs/modules/slots/event.md)
> Definition SSOT: [MODULE_MAP §6](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `plugin_backends` key `event` · trait `EventEstimator` · hook `co_present/run_middle.rs` `EventEstimate` → `publish_legacy_event_impact` → `PersonalityEngine::evolve_by_event`.

This slot estimates dialogue event impact only. It does not own Event Ring envelopes, decision modules, or proactive authorization. See [EVENT_RING](../../../creator-docs-en/plugin-and-architecture/EVENT_RING.md).

**Two paths**: Rule table `EventDetector` vs LLM `estimate_event_impact`. LLM switch is **`HostProfile.event_impact_llm`** — not a slot key. See [DISTRO_CAPABILITY_PROFILE](../../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md).

**Do**: Rule / LLM builtin paths · `remote` / `directory` backends · distro `distro.oclive.toml` for defaults.

**Don't**: Call this slot the whole Event Ring · issue proactive permits · register Turn Thinking as a seventh slot · force the LLM event path on Fast turns.

**Read next**: [MODULE_MAP §12 `event_impact_llm`](../../../handoff/MODULE_MAP_AND_HANDOFF.md) · [turn-thinking](../orchestration/turn-thinking.md) · [RFC_TURN_THINKING](../../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md).
