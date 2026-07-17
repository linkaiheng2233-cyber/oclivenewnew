# Role pack · `config.json` & validation (EN summary)

> Full checklist (ZH): [`human-docs/modules/packs/role-pack-config.md`](../../../human-docs/modules/packs/role-pack-config.md)
> Field SSOT: [ROLE_PACK_SPEC](../../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [MODULE_MAP §12](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `distros/chat-pro/roles/{id}/config.json` · `RoleStorage::load_role` · validation `kernel/crates/oclive_validation` · runtime `turn_thinking.rs` · engines (**not** API layer).

**Do**: Documented `config.json` fields · `oclive_validation` schema · `turn_thinking` OR/AND · latch · ephemeral (RFC §8–12) · migration `035_turn_thinking_runtime.sql` when persistence changes.

**Don't**: Edit blueprint `slot_registry` in role tasks (G1) · parse in Tauri `api/*.rs` · register Turn Thinking as seventh slot.

**Read next**: [turn-thinking](../orchestration/turn-thinking.md) · [07 common tasks §3 (ZH)](../../../human-docs/07_COMMON_TASKS.md) · [RFC_TURN_THINKING §8–12](../../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) · [role-pack-content](role-pack-content.md).
