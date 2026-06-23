# Migrating from v2 to v3 blueprint

**Audience:** Authors on `schema_version: 2` who need **`runtime_config`** or optional **dual-core**. Manual upgrade takes **about 10 minutes**.

**Normative spec:** [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · validation: `oclive_validation::blueprint_v3` · dual-core: [DEVELOPER_GUIDE.md](../dual-core/DEVELOPER_GUIDE.md)

[中文](../../creator-docs/role-pack/V2_TO_V3_MIGRATION.md)

---

## 1. Core differences

| Area | v2 | v3 |
|------|----|----|
| `schema_version` | `2` | `3` |
| Engine / system config | Often in `meta.*` (host may still read) | Top-level **`runtime_config`** (SSOT target) |
| Dual-core | No formal fields | `runtime_config.dual_core` + optional `pipeline.*` |
| Slot zone | No `zone` | Optional `slot_registry.*.zone` |
| Default runtime | `process_message` → `co_present` | **Same when dual-core is off** (zero diff) |

Batch CLI migration (Q18) is **deferred**; copy v2 pack → edit JSON → `pack validate`.

---

## 2. Manual steps (~10 min)

1. **Back up** `distros/chat-pro/roles/<id>/`.
2. Set **`"schema_version": 3`**.
3. Add **`runtime_config`** (move system fields from `meta`):

```json
"runtime_config": {
  "interaction_mode": "immersive",
  "dual_core": { "enabled": false }
}
```

Minimal v3 without dual-core: `"runtime_config": { "dual_core": { "enabled": false } }` or omit `dual_core`.

4. **Optional dual-core:** `"dual_core": { "enabled": true }` plus non-empty `pipeline.experimental` (see [DEVELOPER_GUIDE.md](../dual-core/DEVELOPER_GUIDE.md)).
5. **Optional** `zone` on `slot_registry` entries.
6. **Validate:** `cargo run -p oclive-cli -- pack validate distros/chat-pro/roles/<id>` and `oclive doctor` (v3 checks: `blueprint_v3_file_format`, `slot_registry_v3_llm`, `slot_position_v3_unique`).
7. **Smoke chat** via desktop or `--api`.

---

## 3. Field mapping (v2 → v3)

| v2 | v3 | Notes |
|----|-----|-------|
| `meta.interaction_mode` | `runtime_config.interaction_mode` | Prefer migrate |
| `meta.memory_config` | `runtime_config.memory_config` | Prefer migrate |
| `meta.reply_quality_anchor` | `runtime_config.reply_quality_anchor` | Prefer migrate |
| `slot_registry` | same + optional `zone` | |
| — | `pipeline.stable` / `experimental` | dual-core only |
| — | `runtime_config.dual_core.enabled` | default `false` |

---

## 4. Dual-core is optional

With **`dual_core.enabled: false`** (or omitted), v3 behaves like v2 on the stable path. Creator-facing packs must **not** enable dual-core alone (see [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md) §5.1).

Scaffold: `cargo run -p oclive-cli -- init --dual-core -o ./my-kernel`

---

## 5. FAQ

**Must I upgrade to v3?** No — v2 remains the default shipping baseline.

**No `migrate-v2-v3` CLI?** Q18 deferred; use manual steps or `init --dual-core` template.

**Creator profile vs full pack?** Use `--profile creator` only on pure creator packs; do not validate `distros/chat-pro/roles/mumu` with it.

---

## 6. Related

- v1 → v2: [V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md)
- Learning path: [CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)
