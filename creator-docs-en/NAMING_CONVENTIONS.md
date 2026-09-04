# OCLive naming conventions (English summary)

[中文](../creator-docs/NAMING_CONVENTIONS.md)

## Quick rules

1. **Minimal tool kernel**: one turn/lifecycle orchestration and authority boundary plus six stable capability ports. It is a responsibility boundary today, not yet a separately compiled crate.
2. **Complete reference runtime**: the current embeddable `oclive_kernel_host::OcliveKernel` facade plus SQLite, Event Ring, concrete implementations, facilities, and transport dependencies. Do not call its whole physical size the minimal core.
3. **Six host slots**: `memory` / `emotion` / legacy `event` / `prompt` / `llm` / `agent` (current v2/v3/v4 blueprints: `slot_registry`; legacy v1: `plugin_backends`). These are stable capability categories, not a requirement to invoke slots 1→6 mechanically. Ordinary extensions do not become slot 7.
4. **Facility modules**: reference-runtime extensions **not** in the six slots (e.g. complex emotion, expert routing).
5. **Side-channel capability enhancement modules**: dedicated resolver + turn-chain anchor, host-input path, **or** standalone API; registry [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) (`user_identity`, `reply_post_process`, `theater_director`, `voice.asr`).
6. **Kernel crates** are **not** renamed in v0.2.x — see [kernel/crates/README.md](../kernel/crates/README.md).
7. **Blueprint file** `pipeline.ocblueprint` is a **frozen filename**; it is **not** a step-scheduling DSL (`steps[]` is deprecated on the hot path).
8. **`dual_core`** = feature/config gate; **`dual_pipeline`** = Rust orchestrator + blueprint `pipeline.{stable,experimental}` JSON section.
9. **Canonical imports**:
   - DTOs / errors → `oclive_kernel_types`
   - Traits → `oclive_kernel_contracts`
   - Orchestration → `oclive_kernel_host::domain::…`
10. **Orchestration path**: `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` — **not** `kernel/crates/oclive_kernel_host/src/domain/` (legacy).

## Reserved (not in v0.2)

- **Post-process chain**: [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)
