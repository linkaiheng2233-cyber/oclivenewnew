# Role pack versioning and compatibility

[中文](../../creator-docs/role-pack/PACK_VERSIONING.md)

This page explains **manifest / settings** version fields, unknown-key policy, and how they relate to [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md), [`distros/chat-pro/roles/README_MANIFEST.md`](../../distros/chat-pro/roles/README_MANIFEST.md), and **[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md)** (on-disk shape and CLI validation). Implementation is authoritative in source. **Full index**: [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md).

## `settings.json`: `schema_version`

- **Field**: `schema_version` (`u32`), see [`DiskRoleSettings`](../../kernel/crates/oclive_kernel_types/src/models/role_settings_disk.rs).
- **Default**: `1` (`default_schema_version`).
- **Purpose**: Reserved for future breaking structural changes; runtime currently parses the **latest contract**. If incompatible changes ship, raise the version and branch in the loader.

## `plugin_backends` and PLUGIN_V1

- **`plugin_backends`**: Optional; when omitted, memory / emotion / event / prompt / **Agent** default to **builtin**, **`llm` to `ollama`** (see [`PluginBackends`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs)).
- Backend enums and semantics: **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**.
- **Unknown enum values**: Deserialization may fail — fix spelling or extend `serde` with aliases if added later.

## `min_runtime_version` (manifest, optional)

- **Field**: Top-level **`min_runtime_version`** in `manifest.json` (`Option<String>`), see [`DiskRoleManifest`](../../kernel/crates/oclive_validation/src/manifest.rs).
- **Form**: **semver** (e.g. `"0.2.0"`), parsed by the **semver** crate; compared to **oclivenewnew app version** (`distros/desktop-tauri/Cargo.toml` `version`, compile-time `env!("CARGO_PKG_VERSION")`).
- **Semantics**: If **host version is below** `min_runtime_version`, `load_role` **refuses** with a readable error (prompt to upgrade oclive). Omitted field → no check.
- **Pack editor**: `HOST_RUNTIME_VERSION` in `oclive-pack-editor` should stay in sync with Cargo version; WASM from `npm run wasm:build` matches [`validate_min_runtime_version`](../../kernel/crates/oclive_validation/src/validate.rs).

## Unknown JSON keys (top-level tightening)

- **`manifest.json` / `settings.json` root**: Before deserialize, **top-level key whitelist** ([`json_keys`](../../kernel/crates/oclive_validation/src/json_keys.rs)): keys not on the list **error**; keys starting with **`_`** are creator notes and **allowed** (see [`README_MANIFEST`](../../distros/chat-pro/roles/README_MANIFEST.md)).
- **Nested objects**: Extra keys still mostly follow **serde struct** ignore rules (historical behavior); further tightening is a separate contract change.

## Validation chain

- Merged disk manifest goes through **`validate_disk_manifest`** ([`role_manifest_validate`](../../kernel/crates/oclive_kernel_host/src/domain/role_manifest_validate.rs)) and **`validate_min_runtime_version`** before runtime `Role`.
- After load: **`validate_role_interaction_mode`**; if `plugin_backends` declares **`remote`** without matching `OCLIVE_REMOTE_*` env, runtime **`log::warn`** (does not block load; runtime still falls back per PLUGIN_V1). See [`log_plugin_backends_remote_missing_env`](../../kernel/crates/oclive_kernel_host/src/domain/role_manifest_validate.rs).
- On contract changes, sync: **Rust validation**, `README_MANIFEST`, **this file**, and PLUGIN_V1 when needed.

## Month-1 milestone (contract boundary) — aligned with source

Roadmap “month 1” **swappable subsystems** in this repo:

- **Contract docs**: [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) (includes `send_message` orchestration), this file, and [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) month-1 entries (`plugin_backends` naming).
- **Code boundary**: [`PluginHost`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) + [`PluginBackends`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs); **not** separate top-level `memory_backend` / `affect_backend` manifest fields.
- **`min_runtime_version`**: Enabled; compared to **`distros/desktop-tauri/Cargo.toml` `version`**.

## `knowledge` (worldview, optional)

- **Semantics**: Markdown under role pack **`knowledge/`** for prompt retrieval and event keyword hints; **not** a `plugin_backends` subsystem — see [WORLDVIEW_KNOWLEDGE.md](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md) (ZH).
- **Fields**: `enabled` (`bool`), `glob` (`string`, must start with `knowledge/`; default `knowledge/**/*.md`).
- **Validation**: `glob` must not be empty (`validate_knowledge_manifest_disk`).

## `evolution.personality_source` (summary)

- **Field**: `vector` (default) or `profile`, see `EvolutionConfigDisk`.
- **Summary**: `profile` = **core personality archive** (`core_personality.txt`) + **runtime mutable archive** (DB, model-maintained); **seven dimensions** are mostly a view. Details: [personality-archive-notes.md](../../docs/personality-archive-notes.md) and [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md).

## Related

- [EDITOR_VALIDATION_ROADMAP.md](../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md) — editor vs runtime validation split (ZH)
- [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)
- [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)
- [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)
- [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md)
