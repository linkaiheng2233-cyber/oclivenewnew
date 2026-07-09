# Human developer docs (English mirror)

> **Audience**: Rust / Vue engineers **not** using Cursor — clone → run → understand the main chain → first kernel PR in 3–5 working days.  
> **Time**: ~2–3 days for L0–L6 (1–2 days with a maintainer).  
> **Chinese SSOT**: [human-docs/README.md](../human-docs/README.md) · **Contracts**: [creator-docs-en/](../creator-docs-en/)

[中文接手包](../human-docs/README.md)

---

## Mirror policy

| Principle | Detail |
|-----------|--------|
| **SSOT** | Chinese `human-docs/` — this tree is a **phased English mirror** of the L0–L8 ladder |
| **Tone** | Human learning pack — not AI gatekeeping (agents use [AGENTS.md](../AGENTS.md)) |
| **Fallback** | Missing EN page → read the linked Chinese file |
| **Sync** | Same PR as Chinese when both exist; see [creator-docs-en/README.md § Sync rules](../creator-docs-en/README.md#sync-rules) |

---

## Learning ladder (L0–L8)

Aligned with [human-docs/README.md §学习阶梯](../human-docs/README.md#学习阶梯).

| Level | English | Chinese | Core question | ~Time |
|-------|---------|---------|---------------|-------|
| **L0** | [00_VISION_AND_POSITIONING.md](00_VISION_AND_POSITIONING.md) | [00](../human-docs/00_VISION_AND_POSITIONING.md) | What this is / is not | 15 min |
| **L1** | [01_ARCHITECTURE_SIMPLE.md](01_ARCHITECTURE_SIMPLE.md) | [01](../human-docs/01_ARCHITECTURE_SIMPLE.md) | One-turn flow · three memory stores · six slots | 45 min |
| **L2** | [02_THIRTY_MINUTE_START.md](02_THIRTY_MINUTE_START.md) | [02](../human-docs/02_THIRTY_MINUTE_START.md) | Clone, run, verify | 30 min |
| **L3** | [03_GLOSSARY.md](03_GLOSSARY.md) + [04_ENGINEERING_RULES_SUMMARY.md](04_ENGINEERING_RULES_SUMMARY.md) | [03](../human-docs/03_GLOSSARY.md) + [04](../human-docs/04_ENGINEERING_RULES.md) | Terms · PR rules · doc discipline | 45 min |
| **L4** | [05_DEBUGGING.md](05_DEBUGGING.md) | [05](../human-docs/05_DEBUGGING.md) | Debug without AI | 30 min |
| **L5** | [06_KERNEL_LEARNING_PATH.md](06_KERNEL_LEARNING_PATH.md) | [06](../human-docs/06_KERNEL_LEARNING_PATH.md) | Main-chain maintainer Day 1–5 | ½–3 days |
| **L6** | [07_COMMON_TASKS.md](07_COMMON_TASKS.md) | [07](../human-docs/07_COMMON_TASKS.md) | Where to edit for task X | On demand |
| **L7** | [08_REFERENCE_MAP.md](08_REFERENCE_MAP.md) | [08](../human-docs/08_REFERENCE_MAP.md) | Deep docs by topic | On demand |
| **L8** | [08_PR_GATE_MATRIX.md](08_PR_GATE_MATRIX.md) · [09_GLOSSARY.md](09_GLOSSARY.md) · [10_SETUP_WINDOWS.md](10_SETUP_WINDOWS.md) | same | CI gates · abbreviations · MSVC | On demand |

**Extra (EN-only shortcut)**: [07_FIRST_PR.md](07_FIRST_PR.md) — first PR recipe; complements L6 [07_COMMON_TASKS.md](07_COMMON_TASKS.md).

**Module packs (slot summaries)**: [modules/README.md](modules/README.md) · full ZH SSOT [human-docs/modules/](../human-docs/modules/README.md)

**Start here if you know Rust**: [02_THIRTY_MINUTE_START.md](02_THIRTY_MINUTE_START.md) — not AGENTS.md.

---

## Mirror status (human-docs-en)

Last reviewed: **2026-07-10**.

| Block | Status | Notes |
|-------|--------|-------|
| **L0–L2** | **Mirrored** | 00–02 |
| **L3–L4** | **Mirrored** | 03–05 (+ 04 summary vs ZH full 04) |
| **L5** | **Mirrored** | 06 kernel path |
| **L6–L7** | **Mirrored** | 07_COMMON_TASKS, 08_REFERENCE_MAP |
| **L8** | **Mirrored** | 08_PR_GATE_MATRIX, 09, 10 |
| **modules/** | **Mirrored** | README + all 19 module EN summaries (ZH checklist links) |
| **paths/** | **Mirrored** | frontend · integrator · plugin-author |
| **team/** | **Pending** | Chinese-only sprint tracks |

When you change Chinese ladder pages **00–10**, update the English mirror in the **same change-set** if it exists.

---

## Documentation discipline

- Human summary: [04 § Documentation](04_ENGINEERING_RULES_SUMMARY.md#documentation-discipline)
- AI rules G10–G16: [AI_CHANGE_BOUNDARIES.md](../handoff/AI_CHANGE_BOUNDARIES.md)
- Five doc layers: [handoff/README.md §文档分责](../handoff/README.md)

Contributing: [CONTRIBUTING.en.md](../CONTRIBUTING.en.md)
