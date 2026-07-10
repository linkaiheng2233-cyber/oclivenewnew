# Role pack · content & persona (EN summary)

> Full checklist (ZH): [`human-docs/modules/packs/role-pack-content.md`](../../human-docs/modules/packs/role-pack-content.md)  
> Boundary SSOT: [ROLE_PACK_BOUNDARY](../../handoff/ROLE_PACK_BOUNDARY.md) · [MODULE_MAP §14](../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `distros/chat-pro/roles/{role_id}/` · Tier0 truth source **`core_personality.txt`** (not `prompts/system.md`) · **not** `process_message` · **not** blueprint `slot_registry`.

**Do**: `core_personality.txt` · `scenes/` · `prompts/` (incl. `deep_capsule.txt`) · portrait catalog assets · `reply_quality_anchor` (replaces default anchor only, **not** guardrails) · oclive-pack-editor for visual editing.

**Don't**: Edit `slot_registry` / `plugin_backends` (G1) · override `KERNEL_DIALOGUE_GUARDRAILS` · change kernel migrations or DTOs in role-pack tasks.

**Read next**: [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [role-pack-config](role-pack-config.md) · [CREATOR_LEARNING_PATH](../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md) · [README_MANIFEST](../../distros/chat-pro/roles/README_MANIFEST.md).
