# Role pack author learning path

Time-boxed steps. **Normative layout** remains [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) and [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md). CLI from repo root: **`cargo run -p oclive-cli -- pack …`**.

---

## Migrate v1 → v2 (~10 min)

Packs still on **`manifest.json` + `settings.json`**: **[V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md)** — `pack migrate-to-blueprint` → default `pack validate` → smoke chat in the host.

---

## Beginner (~30 min)

| Step | Goal | Read / do |
|------|------|-------------|
| 1 | Know the on-disk shape | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) **§1** |
| 2 | Generate a minimal pack | `cargo run -p oclive-cli -- pack create -o <parent> --id my_first_role --format-blueprint-v2` (writes `pipeline.ocblueprint`; see [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)) |
| 3 | Open in the editor | **oclive-pack-editor** for v2 blueprint or legacy files ([CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)) |
| 4 | Edit façade fields | v2: `pipeline.ocblueprint` → `meta`; legacy: [README_MANIFEST](../../roles/README_MANIFEST.md) |

**Done when:** `pack validate <role-root>` passes (default v2); legacy packs use `--profile legacy`.

---

## Intermediate (~1–2 h)

| Topic | Read |
|-------|------|
| **Seven-dim personality** | [README_MANIFEST](../../roles/README_MANIFEST.md) · [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) |
| **System prompts / openings** | ROLE_PACK_SPEC + [WORLDVIEW_KNOWLEDGE.md](WORLDVIEW_KNOWLEDGE.md); final chat prompt is driven by **`plugin_backends.prompt`** and engine policy |
| **`plugin_backends` slots** | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |

**Done when:** You can set each slot to `builtin` / `remote` / `directory` and explain `directory_plugins` → manifest `id`.

---

## Advanced (~half day)

| Topic | Notes |
|-------|--------|
| **`reply_quality_anchor`** | See README_MANIFEST + ROLE_PACK_SPEC merged settings table; behavior follows validation + host load rules |
| **`pipeline.ocblueprint` v2 (recommended SSOT)** | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · [BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md). **Desktop orchestration** is **`process_message` → `co_present`** (no blueprint `steps[]`; [AGENTS.md](../../AGENTS.md)). Desktop graph can **`save_role_slot_registry`** |
| **Validate** | Default v2: `pack validate <role-root>`; legacy: `--profile legacy`; headless: `--profile robot-soul` (legacy shape; ROLE_PACK_SPEC §6) |
| **Editor wasm checks** | **oclive-pack-editor** `wasm:build` + “run all checks” |

**Done when:** `pack validate` is clean and you know which keys are host-validated vs author-only.

---

## Publish

| Step | Command / doc |
|------|----------------|
| **`.oclivepack`** | `cargo run -p oclive-cli -- pack publish <role-root> -o <path>` |
| **Community index JSON** | [ROLE_PACK_INDEX.md](ROLE_PACK_INDEX.md) · [../roadmap/MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| **Host compatibility** | [COMPATIBILITY.md](../COMPATIBILITY.md) · `manifest.min_runtime_version` |

---

## Next

- Versioning: [PACK_VERSIONING.md](PACK_VERSIONING.md)  
- Editor validation roadmap: [EDITOR_VALIDATION_ROADMAP.md](EDITOR_VALIDATION_ROADMAP.md)
