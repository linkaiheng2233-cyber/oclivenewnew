# V1 documentation cleanup audit

**Date:** 2026-05-20  
**Scope:** `creator-docs/`, `creator-docs-en/`  
**Authority:** v2 = `pipeline.ocblueprint` + `slot_registry`; v1 = `manifest.json` / `settings.json` / `plugin_backends` (**deprecated**, migration/RFC only).

## Summary

| Action | Count (approx.) |
|--------|-----------------|
| **Modified** (active doc → v2-first) | 18 files (zh + en pairs) |
| **Confirmed legacy / migration** | `V1_TO_V2_MIGRATION.md`, `ROLE_PACK_SPEC` §legacy, `SETTINGS_REFERENCE` §legacy |
| **Confirmed design history** | `RFC_*`, `CHANGELOG`, roadmap, `PLUGIN_V1` legacy Mermaid |
| **OK as non–role-pack manifest** | Directory plugin `plugins/*/manifest.json` in `DIRECTORY_PLUGINS.md` |

## Keyword sweep (post-fix)

Active docs may still **mention** v1 terms when labeled **legacy / deprecated / migration**. They must not describe v1 as the **current** authoring path.

| Keyword | Allowed in | Removed or relabeled in |
|---------|------------|-------------------------|
| `manifest.json` (role pack) | Migration, ROLE_PACK_SPEC legacy, README_MANIFEST history | Studio USER_GUIDE, CREATOR_WORKFLOW (en), PURE_KERNEL_BOUNDARY |
| `settings.json` (role backends) | SETTINGS_REFERENCE legacy §, PLUGIN_V1 legacy diagrams | Studio create flow, index “current config” rows |
| `plugin_backends` | Runtime fold semantics, PLUGIN_V1 legacy, migration | KERNEL diagram labels, learning-path acceptance |
| `set_session_plugin_backend` | — | Replaced with `set_session_slot_override` in KERNEL_AND_MODULES (zh/en) |
| `personality.json` | No active hits as current SSOT | — |
| 六槽硬编码 | PLUGIN_V1 / architecture as **fold target**, not pack SSOT | — |

## Per-file disposition (high priority)

| File | Disposition |
|------|-------------|
| `ROLE_PACK_SPEC.md` (zh/en) | **Confirmed** — v2 §2 primary; legacy files marked deprecated |
| `PLUGIN_V1.md` (zh/en) | **Modified** — v2 blueprint § first; legacy diagram labeled |
| `CREATOR_WORKFLOW.md` (zh) | **Confirmed** — already v2-first |
| `CREATOR_WORKFLOW.md` (en) | **Modified** — v2 checklist |
| `CREATOR_LEARNING_PATH.md` (zh/en) | **Modified** — acceptance uses `slot_registry` |
| `OCLIVE_CLI_GUIDE.md` | **Confirmed** — `--smart`, `--deny`, `--oocp` documented |
| `SETTINGS_REFERENCE.md` (zh/en) | **Modified** (en header) — v2 + legacy sections |
| `PROJECT_OVERVIEW.md` (zh) | **Confirmed** — v2 SSOT stated |
| `KERNEL_AND_MODULES_ARCHITECTURE.md` (zh/en) | **Modified** — slot override API, diagram labels |
| `studio/USER_GUIDE.md` (zh/en) | **Modified** — create/edit/export v2 workflow |
| `PURE_KERNEL_BOUNDARY.md` (zh/en) | **Modified** — soul pack = blueprint |
| `DOCUMENTATION_INDEX.md` (zh/en) | **Modified** — quick links → v2 |
| `V1_TO_V2_MIGRATION.md` | **Confirmed** — migration only |
| `DIRECTORY_PLUGINS.md` | **Confirmed** — plugin `manifest.json` ≠ role pack |
| `RFC_OCLIVE_MONOLITH_MODE.md` | **Confirmed** — design history |

## Remaining acceptable mentions

- **CREATOR_WORKFLOW** studio line: `OCLIVE_LLM_BACKEND` overrides **legacy** `plugin_backends.llm` at runtime (accurate).
- **PLUGIN_V1** legacy Mermaid and enum tables for six-slot fold and Remote/directory wire.
- **handoff/** RFC and implementation plans.

## Verification command

```bash
rg -n "manifest\.json|settings\.json|plugin_backends|set_session_plugin_backend" creator-docs creator-docs-en \
  --glob '*.md' | rg -v 'V1_TO_V2|legacy|deprecated|已废弃|migration|RFC_|CHANGELOG'
```

Review any unmatched lines manually; expect directory-plugin and code-path references.
