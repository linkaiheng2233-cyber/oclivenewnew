# Creator workflow (English summary)

The **authoritative** walkthrough (Chinese, kept current with imports and `OCLIVE_ROLES_DIR`) is:

**[../../creator-docs/getting-started/CREATOR_WORKFLOW.md](../../creator-docs/getting-started/CREATOR_WORKFLOW.md)**

## Short English checklist (v2)

1. Put each role under `distros/chat-pro/roles/<role_id>/` with **`pipeline.ocblueprint`** (`schema_version: 2`, `meta`, `slot_registry`) plus assets (`core_personality.txt`, scenes, etc.). See [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md).
2. Use **oclive-studio** create mode or `oclive pack create` / copy `distros/chat-pro/roles/mumu/`; export zip or folder; the host imports `.ocpak` / `.zip` / directory.
3. Set **`OCLIVE_ROLES_DIR`** to the roles root; validate with `oclive pack validate` (v2 default) before sharing.
4. **Legacy v1 (deprecated):** `manifest.json` + `settings.json` only for migration — [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md).

This file exists so the English **getting-started** tree can link a first-class path from [DOCUMENTATION_INDEX.md](./DOCUMENTATION_INDEX.md).
