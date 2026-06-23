# RFC: oclive Studio merge (launcher + pack editor)

| Field | Value |
|-------|--------|
| Status | **Shipped** — **[oclive-studio](https://github.com/linkaiheng2233-cyber/oclive-studio)**; **oclive-launcher** and **oclive-pack-editor** are **deprecated** |
| Config SSOT | **`studio-config.json`** (`rolesDir` → `OCLIVE_ROLES_DIR`, LLM, runtime paths) |
| Role pack SSOT | **v2** `pipeline.ocblueprint` — [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |
| User guide | [`handoff/studio/USER_GUIDE.md`](../../handoff/studio/USER_GUIDE.md) · [CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md) |

## Goals

- One app: **Launch mode** (diagnostics, start `oclivenewnew`) and **Create mode** (edit v2 blueprint, validate, trial chat, export).
- Runtime coupling is the on-disk **roles root** only.

## Non-goals

- Full in-process `process_message` inside Studio; trial chat uses **`--api`** against the main-repo kernel.

---

[中文](../../creator-docs/rfc/RFC_STUDIO_MERGE.md)
