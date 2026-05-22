# Chinese–English documentation parity check

**Date:** 2026-05-20  
**ZH tree:** `creator-docs/` — **74** `.md` files  
**EN tree:** `creator-docs-en/` — **52** `.md` files  

**Policy:** Technical authority = **Chinese** (`creator-docs/`). English mirror = contract/plugin/getting-started hubs + learning paths; long-tail zh-only docs are **acceptable** with index pointers.

## Mirror status: core contracts

| Document | ZH | EN | Status |
|----------|----|----|--------|
| ROLE_PACK_SPEC | ✓ | ✓ | **Aligned** (v2-first; en synced) |
| PLUGIN_V1 | ✓ | ✓ (summary) | **Aligned** (en points to zh for full tables) |
| ERROR_CODES | ✓ | ✓ | **Aligned** |
| PERFORMANCE | ✓ | ✓ | **Aligned** |
| SETTINGS_REFERENCE | ✓ | ✓ | **Aligned** (en header fixed 2026-05-20) |
| OCLIVE_CLI_GUIDE | ✓ | ✓ | **Aligned** |
| V1_TO_V2_MIGRATION | ✓ | ✓ | **Aligned** |
| DOCUMENTATION_INDEX | ✓ | ✓ | **Aligned** (quick links v2) |

## Chinese-only (acceptable / planned)

| Path | Rationale |
|------|-----------|
| `architecture/DESIGN_DECISIONS.md` | EN stub at `architecture-en/DESIGN_DECISIONS.md` (in zh tree) |
| `roadmap/*` (6 files) | Product planning; index links zh |
| `role-pack/PACK_VERSIONING.md`, `WORLDVIEW_KNOWLEDGE.md`, … | Creator long-tail; EN index links zh |
| `testing/FUZZING.md`, `NARRATIVE_HINT_*`, `L03_*` | Specialist; EN testing hub links zh |
| `development/LIGHTWEIGHT_PROFILE.md` | EN mirror: `creator-docs-en/LIGHTWEIGHT_PROFILE.md` (root-level) |
| `video-script/PLUGIN_DEVELOPMENT_SCRIPT.md` | Internal script |

## English-only

| Path | Rationale |
|------|-----------|
| `creator-docs-en/LIGHTWEIGHT_PROFILE.md` | Mirror of zh `development/LIGHTWEIGHT_PROFILE.md` (different path) |

## Bilingual footer links

- Pattern: `[English](../../creator-docs-en/...)` / `[中文](../../creator-docs/...)`
- Spot-checked: KERNEL_AND_MODULES, ROLE_PACK_SPEC, SETTINGS_REFERENCE, TESTING_GUIDE — **OK**
- **Note:** `DESIGN_DECISIONS.md` uses `architecture-en/` under **zh** tree for English body (intentional).

## Content drift fixes (2026-05-20)

| Pair | Fix |
|------|-----|
| studio/USER_GUIDE | en/zh v2 workflow |
| PURE_KERNEL_BOUNDARY | en/zh soul = blueprint |
| CREATOR_LEARNING_PATH | en/zh acceptance |
| SETTINGS_REFERENCE | en title + v2 preamble |
| KERNEL_AND_MODULES | en/zh API labels |

## Needs follow-up (non-blocking)

- Full EN translation of roadmap and `PACK_VERSIONING` (optional).
- Expand `creator-docs-en/getting-started/CREATOR_WORKFLOW.md` beyond checklist when studio UX stabilizes.

## Regenerate file list

```bash
node scripts/doc-parity-list.mjs
```
