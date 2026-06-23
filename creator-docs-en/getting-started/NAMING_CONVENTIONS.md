# OCLive naming conventions (English summary)

**Canonical SSOT (Chinese)**: [`../creator-docs/NAMING_CONVENTIONS.md`](../creator-docs/NAMING_CONVENTIONS.md)

## Quick rules

1. **Six host slots**: `memory` / `emotion` / `event` / `prompt` / `llm` / `agent` (v2: `slot_registry`; legacy: `plugin_backends`).
2. **Facility modules**: in-orchestration kernel extensions **not** in the six slots (e.g. complex emotion, expert routing).
3. **Side-channel capability enhancement modules**: dedicated resolver + turn-chain anchor **or** standalone API; registry [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) (`user_identity`, `reply_post_process`, `theater_director`).
4. **Kernel crates** are **not** renamed in v0.2.x — see [kernel/crates/README.md](../../kernel/crates/README.md).
5. **Blueprint file** `pipeline.ocblueprint` is a **frozen filename**; it is **not** a step-scheduling DSL (`steps[]` is deprecated on the hot path).
6. **`dual_core`** = feature/config gate; **`dual_pipeline`** = Rust orchestrator + blueprint `pipeline.{stable,experimental}` JSON section.
7. **Canonical imports**:
   - DTOs / errors → `oclive_kernel_types`
   - Traits → `oclive_kernel_contracts`
   - Orchestration → `oclive_kernel_host::domain::…`
8. **Orchestration path**: `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` — **not** `kernel/crates/oclive_kernel_host/src/domain/` (legacy).

## Reserved (not in v0.2)

- **Post-process chain**: [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)
