# Migrating role packs from v1 to v2 (A.I.Live)

**A.I.Live** role packs use **`pipeline.ocblueprint`** as the v2 SSOT (engineering codename **oclive**).

**Already on v2?** Next: **[V2_TO_V3_MIGRATION.md](V2_TO_V3_MIGRATION.md)** (`runtime_config`, optional dual-core; ~10 min manual upgrade).

**Audience**: creators still on `manifest.json` + `settings.json`. Following this guide takes **about 10 minutes** including validation.

**Normative spec**: [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · validation: `kernel/crates/oclive_validation`.

[中文](../../creator-docs/role-pack/V1_TO_V2_MIGRATION.md)

---

## 1. v2 blueprint architecture (short)

**`pipeline.ocblueprint` is the single source of truth (SSOT)** for the pack: `schema_version: 2`, `meta` (former manifest + engine settings fields), and `slot_registry` (open multi-instance slots). **Do not** keep legacy twin files alongside the blueprint. **Do not** put `steps[]`, `entry`, or `module_relations` in the file (edges are derived at runtime from `slot_registry`).

Desktop chat orchestration remains **`process_message` → `co_present`**; the old blueprint `steps[]` DSL is **not** used on the hot path (see root `AGENTS.md`).

---

## 2. Field mapping

### 2.1 `manifest.json` → `meta`

| legacy `manifest.json` | v2 `meta` | notes |
|------------------------|-----------|-------|
| `id` | `id` | match folder name |
| `name` | `name` | |
| `version` | `version` | |
| `author` | `author` | |
| `description` | `description` | |
| `default_personality` (7 floats) | `personality` | object or 7-element array |
| `user_relations` | `relations` | renamed |
| `default_relation` | `default_relation` | |
| `scenes` | `scenes` | |
| `evolution` | `evolution` | |
| `memory_config` | `memory_config` | |
| `identity_binding` | `identity_binding` | |
| `life_trajectory` / `life_schedule` / `knowledge` | same keys | |
| `dev_only` | `dev_only` | |
| `min_runtime_version` | `min_runtime_version` | |
| `ollama_model` (if on manifest) | `ollama_model` | |

> A standalone `personality.json` was never loaded by the host; personality lives in `default_personality` → `meta.personality`.

### 2.2 `settings.json` → `meta` or `slot_registry`

| legacy `settings.json` | v2 location | notes |
|------------------------|-------------|-------|
| `interaction_mode` | `meta.interaction_mode` | |
| `remote_presence` | `meta.remote_presence` | |
| `autonomous_scene` | `meta.autonomous_scene` | |
| `reply_quality_anchor` | `meta.reply_quality_anchor` | |
| `plugin_backends` | **`slot_registry`** | see below |
| `plugin_backends.directory_plugins` | per-slot `plugin` / `plugins` | |

### 2.3 `plugin_backends` → `slot_registry` (default instance keys)

| module `type` | default key | `backend` from |
|---------------|-------------|----------------|
| `memory` | `memory` | `plugin_backends.memory` |
| `emotion` | `emotion` | `plugin_backends.emotion` |
| `event` | `event` | `plugin_backends.event` |
| `prompt` | `prompt` | `plugin_backends.prompt` |
| `llm` | `llm` | `plugin_backends.llm` |
| `agent` | `agent` | `plugin_backends.agent` |
| `complex_emotion` | `complex_emotion` | defaults to `builtin` if absent |

Each entry needs `label`, `position`, and directory `plugin`/`plugins` when applicable.

---

## 3. Automated migration: `pack migrate-to-blueprint`

From the **oclivenewnew** repo root:

```bash
cargo run -p oclive-cli -- pack migrate-to-blueprint distros/chat-pro/roles/my_role
```

| flag | default | meaning |
|------|---------|---------|
| `path` | positional | role root with `manifest.json` |
| `--remove-legacy` | **true** | delete `manifest.json` and `settings.json` after write |
| `--no-remove-legacy` | — | keep legacy files (not recommended; the default blueprint profile rejects dual on-disk shapes) |

---

## 4. Post-migration validation

```bash
cargo run -p oclive-cli -- pack validate distros/chat-pro/roles/my_role
```

Default profile is **v2** (`default` / `blueprint-v2`). Use `--profile legacy` only for unmigrated packs.

**Sample pack**: `distros/chat-pro/roles/mumu/` (blueprint only).

---

## 5. FAQ

**Are legacy files kept?**  
With default `--remove-legacy`, no — content is merged into `pipeline.ocblueprint`. Use Git history or backups to recover.

**Can I roll back?**  
There is no host “downgrade to v1” button. Restore the directory from Git or a zip. v2 and legacy files must not coexist (validation error).

**Do session module overrides still work?**  
v2 uses **`slot_registry` + per-session `slot_key` overrides**. C1 `set_session_plugin_backend` maps module names to default keys and requires `slot_registry` on the pack.

**What happened to `steps[]`?**  
Removed from the runtime hot path; use plugins and `slot_registry` instead.

---

## 6. Ten-minute checklist

| min | action |
|-----|--------|
| 0–2 | backup; skim §2 |
| 2–4 | `pack migrate-to-blueprint` |
| 4–6 | `pack validate` (default blueprint profile; this migration emits v2) |
| 6–8 | editor: `meta` + `slot_registry` (≥1 `llm`) |
| 8–10 | `load_role` + test chat |

---

| date | note |
|------|------|
| 2026-05-20 | initial v2 migration guide |
