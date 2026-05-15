# Role pack author learning path

Time-boxed steps. **Normative layout** remains [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) and [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md). CLI from repo root: **`cargo run -p oclive-cli -- pack …`**.

---

## Beginner (~30 min)

| Step | Goal | Read / do |
|------|------|-------------|
| 1 | Know the on-disk shape | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) **§1** |
| 2 | Generate a minimal pack | `cargo run -p oclive-cli -- pack create -o <parent> --id my_first_role` (creates `roles/<id>/`; see [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)) |
| 3 | Open in the editor | Use **oclive-pack-editor** for `manifest.json` / `settings.json` ([CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)) |
| 4 | Edit façade fields | `name`, `description`, scenes — [README_MANIFEST](../../roles/README_MANIFEST.md) |

**Done when:** `cargo run -p oclive-cli -- pack validate <role-root>` passes.

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
| **`pipeline.ocblueprint` (optional)** | Optional file per ROLE_PACK_SPEC; **desktop hot path** is **`process_message` → `co_present`** ([AGENTS.md](../../AGENTS.md)). Blueprint is orthogonal to **Monolith** compile-time welding — see [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| **Validate** | `cargo run -p oclive-cli -- pack validate <role-root>` (`--profile robot-soul` when needed) |
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
