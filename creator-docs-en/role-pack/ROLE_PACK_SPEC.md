# Role pack format (ROLE_PACK_SPEC)

This document describes the **on-disk role pack shape aligned with the oclive main host load path**, so **multiple distributions** (desktop Tauri, headless `kernel_server`, future launcher) can share one pack. Authoritative detail remains source code and existing docs:

- Creator-facing fields: [README_MANIFEST.md](../../roles/README_MANIFEST.md)
- Six host slots and orchestration: [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md), [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)
- Kernel-centric module diagram: [KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)

**Standard JSON has no `//` comments**; use **`_`-prefixed keys** (ignored at load) or out-of-pack prose.

---

## 1. Directory layout (recommended)

The pack root is usually **`roles/{roleId}/`** (`{roleId}` matches `manifest.json` → **`id`**, which simplifies import and validation).

```text
roles/{role_id}/
├── manifest.json           # Facade: display info, seven-dim default_personality, scenes, relations, …
├── settings.json           # Optional; engine: plugin_backends, evolution, schema_version, …
├── core_personality.txt    # Optional; core personality prose (profile mode, etc.)
├── ui.json                 # Optional; front-end layout
├── author.json             # Optional; author metadata
├── scenes/
│   └── {scene_id}/
│       ├── scene.json
│       ├── description.txt # optional
│       └── …
├── knowledge/              # Optional; worldview Markdown (see WORLDVIEW_KNOWLEDGE.md)
├── memories/               # Optional; preset memory assets (if the product uses them)
├── assets/                 # Optional; sprites, avatars, static assets
└── pipeline.ocblueprint    # Optional; runtime blueprint (orthogonal to monolith.toml compile-time welding; see RFC)
```

**Note**: There is **no** top-level `personality.json` convention in the repo; **seven-dimensional personality** lives in **`manifest.json` → `default_personality`** (7 `f32`s, see below). `prompts/*.md` may be creator-managed material; it is **not** a required host path; the main dialogue prompt is decided by the engine and `plugin_backends.prompt`.

---

## 2. `manifest.json` (`DiskRoleManifest`)

| Field | Type | Required | Notes |
|-------|------|----------|--------|
| `id` | string | yes | Stable role id; should match folder name |
| `name` | string | yes | Display name |
| `version` | string | yes | Semantic version (string) |
| `author` | string | yes | Author |
| `description` | string | yes | Short blurb |
| `default_personality` | number[] | no | Seven dims `f32`, order: stubbornness, clinginess, sensitivity, assertiveness, forgiveness, talkativeness, warmth; **if non-empty must be exactly 7**, each **0.0–1.0** (`oclive pack validate` checks) |
| `scenes` | string[] | no | Scene ids; may merge with `scenes/` subdirs (see `merge_role_pack_scene_ids`) |
| `user_relations` | object | yes | Keys are relation ids; values include `initial_favorability` (0–100), `favor_multiplier` (positive), … |
| `default_relation` | string | no | Must exist in `user_relations`; may be empty to fall back at load |
| `evolution` | object | no | See README_MANIFEST; `personality_source`: `vector` \| `profile` |
| `memory_config` | object | no | `topic_weights` keys must be declared scenes |
| `identity_binding` | string | no | `global` \| `per_scene` |
| `life_trajectory` / `life_schedule` / `knowledge` / … | optional blocks | no | See README_MANIFEST |
| `min_runtime_version` | string | no | semver; compared to host version passed at validation |
| `dev_only` | bool | no | Debug pack marker |

---

## 3. `settings.json` (`DiskRoleSettings`)

| Field | Type | Required | Notes |
|-------|------|----------|--------|
| `schema_version` | u32 | yes | Host currently supports **1** (see `CURRENT_SETTINGS_SCHEMA_VERSION`) |
| `plugin_backends` | object | no | **Six host slots** + `directory_plugins` + `local_memory_provider_id`; matches `PluginBackends` (see SETTINGS_REFERENCE). Scaffolds may write **`complex_emotion`** extension keys; **the host ignores them on deserialize** |
| `interaction_mode` | string | no | `immersive` \| `pure_chat` |
| `evolution` / `memory_config` / `ollama_model` / `remote_presence` / `autonomous_scene` / `knowledge` / `reply_quality_anchor` | optional | no | Merged with manifest then validated; see README_MANIFEST |

---

## 4. Alignment with kernel concepts

| Concept | On disk |
|---------|---------|
| `PluginBackends` (memory…agent) | `settings.json` → `plugin_backends` |
| Seven-dim personality (vector mode) | `manifest.json` → `default_personality` |
| Interaction mode | `settings.json` → `interaction_mode` |
| Scenes | `manifest.scenes` + `scenes/{id}/` |
| Monolith welding | **Only** scaffold project `monolith.toml` / `process_message_monolith.rs`, **not** shipped with the role pack |

---

## 5. Automated validation

```bash
cargo run -p oclive-cli -- pack validate ./roles/my-role --host-version 0.2.0
```

- Default `--host-version` is **this CLI’s `CARGO_PKG_VERSION`**; when it differs from the desktop host you target, pass a **matching semver** explicitly before checking `min_runtime_version`.
- On success prints: `✓ 角色包验证通过`; on failure lists errors line by line.

**JSON Schema** (IDE hints / external validators): `crates/oclive-cli/schemas/role_pack_manifest.schema.json`, `role_pack_settings.schema.json`.

---

## 6. Scaffold command summary

| Command | Role |
|---------|------|
| `pack validate <dir>` | Directory-level validation |
| `pack create -o <out> --id <id> [--flat]` | Minimal valid pack (`--flat` means `<out>` is the role root) |
| `pack publish <dir> [-o file.oclivepack]` | ZIP pack; root folder name is `manifest.id` |
| `init … --skip-role-pack` | Kernel project without creating `roles/` |

See [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md).
