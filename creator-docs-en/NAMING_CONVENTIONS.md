# OCLive naming conventions (English summary)

[中文](../creator-docs/NAMING_CONVENTIONS.md)

## Quick rules

1. **Minimal tool kernel**: six-slot capability contracts and necessary public legality/causal/error boundaries, per [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities). It is a responsibility boundary, not yet a separately compiled crate or frozen new API. Concrete scheduling and domain application belong to the distro Host.
2. **Complete reference runtime**: the current embeddable `oclive_kernel_host::OcliveKernel` facade plus SQLite, Event Ring, concrete implementations, facilities, and transport dependencies. Do not call its whole physical size the minimal core.
3. **Six host slots**: `memory` / `emotion` / legacy `event` / `prompt` / `llm` / `agent` (current v2/v3/v4 blueprints: `slot_registry`; legacy v1: `plugin_backends`). These are stable capability categories, not a requirement to invoke slots 1→6 mechanically. Ordinary extensions do not become slot 7.
4. **Facility modules**: reference-runtime extensions **not** in the six slots (e.g. complex emotion, expert routing).
5. **Side-channel capability enhancement modules**: dedicated resolver + turn-chain anchor, host-input path, **or** standalone API; registry [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) (`user_identity`, `reply_post_process`, `theater_director`, `voice.asr`).
6. **Kernel crates** are **not** renamed in v0.2.x — see [kernel/crates/README.md](../kernel/crates/README.md).
7. **Blueprint file** `pipeline.ocblueprint` is the current reference-host blueprint's **frozen filename**; it is neither a step-scheduling DSL (`steps[]` is deprecated on the hot path) nor the kernel-minimal role contract.
8. **`dual_core`** = feature/config gate; **`dual_pipeline`** = Rust orchestrator + blueprint `pipeline.{stable,experimental}` JSON section.
9. **Canonical imports**:
   - DTOs / errors → `oclive_kernel_types`
   - Traits → `oclive_kernel_contracts`
   - Orchestration → `oclive_kernel_host::domain::…`
10. **Reference-Host orchestration path**: `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`. Its current scheduling policy is not a mandatory pipeline for every Host.

## Host is not a process or a crate

**Distro Host** denotes runtime composition, scheduling, and domain application (for example ChatPro Host). **Host process** denotes OS deployment; **`oclive_kernel_host`** is a physical crate containing reference-runtime responsibilities. **Adapter** denotes protocol conversion or execution of already authorized operations, not independent domain decision authority. Naming does not require a physical code split.

Minimal role definitions express persona + visual references; richer product fields remain distro-owned ([ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md)). Slot registries, single-writer SQLite deployment, and build modes below/above describe the reference runtime, not prerequisites for the minimal Kernel or minimal role.

## Reserved (not in v0.2)

- **Post-process chain**: [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)
