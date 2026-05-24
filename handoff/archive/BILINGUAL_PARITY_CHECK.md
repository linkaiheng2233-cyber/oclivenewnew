# Chinese–English documentation parity check

**Date:** 2026-05-20 (batch 3)  
**ZH tree:** `creator-docs/` — **81** `.md` files  
**EN tree:** `creator-docs-en/` — **62** `.md` files  

**Policy:** Technical authority = **Chinese** (`creator-docs/`). English mirror = contract/plugin/getting-started hubs + learning paths; long-tail zh-only docs are **acceptable** with index pointers.

## Mirror status: core contracts

| Document | ZH | EN | Status |
|----------|----|----|--------|
| ROLE_PACK_SPEC | ✓ | ✓ | **Aligned** (v2-first; en synced) |
| PLUGIN_V1 | ✓ | ✓ (summary) | **Aligned** (en points to zh for full tables) |
| ERROR_CODES | ✓ | ✓ | **Aligned** |
| PERFORMANCE | ✓ | ✓ | **Aligned** |
| SETTINGS_REFERENCE | ✓ | ✓ | **Aligned** |
| OCLIVE_CLI_GUIDE | ✓ | ✓ | **Aligned** |
| V1_TO_V2_MIGRATION | ✓ | ✓ | **Aligned** |
| DOCUMENTATION_INDEX | ✓ | ✓ | **Aligned** |

## Roadmap / vision (2026-05-20)

| Document | EN path | Status |
|----------|---------|--------|
| VISION_ROADMAP_MONTHLY | `creator-docs-en/roadmap/VISION_ROADMAP_MONTHLY.md` | **EN mirror** |
| VISION_OPEN_LAB | `creator-docs-en/roadmap/VISION_OPEN_LAB.md` | **EN mirror** |
| BACKLOG_EXPERIENCE_AND_ECOSYSTEM | `creator-docs-en/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md` | **EN mirror** |
| MARKET_LAUNCHER_INTEGRATION | `creator-docs-en/roadmap/MARKET_LAUNCHER_INTEGRATION.md` | **EN summary** |
| COMMUNITY_WEB_VISION | `creator-docs-en/roadmap/COMMUNITY_WEB_VISION.md` | **EN summary** |
| PLUGIN_WEB_SECTION | `creator-docs-en/roadmap/PLUGIN_WEB_SECTION.md` | **EN summary** |
| SOMEDAY_TOOLCHAIN_CI | `creator-docs-en/roadmap/SOMEDAY_TOOLCHAIN_CI.md` | **EN summary** |

## Chinese-only (acceptable / planned)

| Path | Rationale |
|------|-----------|
| `architecture/DESIGN_DECISIONS.md` | EN stub at `architecture-en/DESIGN_DECISIONS.md` (in zh tree) |
| `role-pack/PACK_VERSIONING.md`, `WORLDVIEW_KNOWLEDGE.md`, … | Creator long-tail; EN index links zh |
| `testing/FUZZING.md`, `NARRATIVE_HINT_*`, `L03_*` | Specialist; EN testing hub links zh |

## Regenerate file list

```bash
node scripts/doc-parity-list.mjs
```
