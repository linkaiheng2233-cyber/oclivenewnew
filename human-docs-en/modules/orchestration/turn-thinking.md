# Orchestration pack · Turn Thinking (EN summary)

> Full checklist (ZH): [`human-docs/modules/orchestration/turn-thinking.md`](../../../human-docs/modules/orchestration/turn-thinking.md)
> RFC SSOT: [RFC_TURN_THINKING_PERSISTENCE](../../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) · [MODULE_MAP §12](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: **Not** a six-slot · **not** a `plugin_backends` key · code `turn_thinking.rs` · `co_present` / `TurnThinkingRouter` · distro `[turn_thinking]` · role pack `config.json` → `turn_thinking` (RFC §8–12).

**Discipline**: Chat turns **still write** UI log every round; Fast **does not compress** user original utterance.

**Do**: `fast_persistence` · `strong_only` HostProfile fields · package OR/AND · latch · `ephemeral_archive` · migration `035_turn_thinking_runtime.sql`.

**Don't**: Add `turn_thinking` as a sixth-slot key · compress user text on Fast · player-side Fast/Deep toggle (product discipline).

**Read next**: [role-pack-config](../packs/role-pack-config.md) · [slots/memory](../slots/memory.md) · [slots/event](../slots/event.md) · [model-tier](model-tier.md).
