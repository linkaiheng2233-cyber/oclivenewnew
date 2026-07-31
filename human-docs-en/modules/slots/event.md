# Slot pack · `event` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/event.md`](../../../human-docs/modules/slots/event.md)
> Definition SSOT: [MODULE_MAP §6](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `plugin_backends` key `event` · trait `EventEstimator` · hook `co_present` `EventEstimate` stage → `PersonalityEngine::evolve_by_event`.

**Two paths**: Rule table `EventDetector` vs LLM `estimate_event_impact`. LLM switch is **`HostProfile.event_impact_llm`** — not a slot key. See [DISTRO_CAPABILITY_PROFILE](../../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md).

**Do**: Rule / LLM builtin paths · `remote` / `directory` backends · distro `distro.oclive.toml` for defaults.

**Don't**: Register Turn Thinking as a seventh slot · force LLM event path on Fast turns (HostProfile constraint) · edit `slot_registry` in role-pack tasks (G1).

**Read next**: [MODULE_MAP §12 `event_impact_llm`](../../../handoff/MODULE_MAP_AND_HANDOFF.md) · [turn-thinking](../orchestration/turn-thinking.md) · [RFC_TURN_THINKING](../../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md).
