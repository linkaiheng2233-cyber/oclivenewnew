# Human developer handoff (English mirror)

[中文](../human-docs/README.md)

**Readers**: Rust / Vue contributors working on the main repository.
**Goal**: start with the thirty-minute setup guide, locate your module, then read only task-relevant references.
**Updated**: 2026-09-11.

This is a learning guide, not another architecture contract. Creators should use the [creator golden path](../creator-docs-en/getting-started/CREATOR_GOLDEN_PATH.md); AI agents start with [AGENTS](../AGENTS.md). For a conceptual introduction, read 00 → 01 → a module's current source; use technical debt for completion status. Distinguish [current implementation, candidates, prepared work, and pauses](../handoff/README.md#documentation-status).

## Start working

1. Follow [02 · Thirty-minute start](02_THIRTY_MINUTE_START.md).
2. Read [04 · Engineering rules](04_ENGINEERING_RULES_SUMMARY.md).
3. Choose a [module start pack](modules/README.md).
4. Follow its source anchors and applicable verification commands; do not default to reading all of `handoff/`.

## Choose a path

| Task | Route |
|---|---|
| Vue / Chat Pro | [Frontend path](paths/frontend.md) → [module packs](modules/README.md) |
| Kernel contracts / Rust reference Host | [01 Architecture](01_ARCHITECTURE_SIMPLE.md) → [responsibility/source map](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map) → [06 Reference main chain](06_KERNEL_LEARNING_PATH.md) |
| Six-slot module | [03 Glossary](03_GLOSSARY.md) → [module packs](modules/README.md) |
| Plugin development | [Plugin author path](paths/plugin-author.md) → [current plugin contract](../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) |
| Hardware / headless / new distro | [Integrator path](paths/integrator.md) → [HostProfile](../creator-docs-en/kernel/DISTRO_CAPABILITY_PROFILE.md) |
| First small PR | [07 Common tasks](07_COMMON_TASKS.md) → [PR gates](08_PR_GATE_MATRIX.md) |

## Learning ladder

| Level | Document | Question |
|---|---|---|
| L0 | [00 Vision](00_VISION_AND_POSITIONING.md) | Why the project exists |
| L1 | [01 Architecture](01_ARCHITECTURE_SIMPLE.md) | Kernel / slots / Host first; reference turns, events, and memory second |
| L2 | [02 Setup](02_THIRTY_MINUTE_START.md) | Build and local verification |
| L3 | [03 Terms](03_GLOSSARY.md) · [04 Rules](04_ENGINEERING_RULES_SUMMARY.md) | Vocabulary and contribution discipline |
| L4 | [05 Debugging](05_DEBUGGING.md) | Locate common failures |
| L5 | [06 Reference main chain](06_KERNEL_LEARNING_PATH.md) | Explore `process_message`; not a universal minimal Kernel pipeline |
| L6 | [07 Common tasks](07_COMMON_TASKS.md) | Map tasks to source and tests |
| L7 | [08 Reference map](08_REFERENCE_MAP.md) | Find topic SSOTs |

Additional entries: [Windows setup](10_SETUP_WINDOWS.md), [PR gates](08_PR_GATE_MATRIX.md), [first PR shortcut](07_FIRST_PR.md).

## Documentation discipline

- This directory explains learning and getting started; it does not duplicate long contract tables.
- Responsibilities: [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md). Current integration contracts: [creator-docs index](../creator-docs-en/getting-started/DOCUMENTATION_INDEX.md).
- Progress, gaps, and pauses: [TECHNICAL_DEBT](../handoff/TECHNICAL_DEBT_INVENTORY.md). Archives are historical evidence, not current behavior.
- Chinese pages own the learning content; English may be a labeled summary. If a mirror exists, update it in the same change-set; otherwise link to Chinese explicitly. A mirror's existence does not prove semantic parity.
- AI change rules: [AI_CHANGE_BOUNDARIES](../handoff/AI_CHANGE_BOUNDARIES.md); document ownership and maintenance: [handoff/README](../handoff/README.md#documentation-maintenance).

Ready means you can locate the task's module, source and SSOT, and identify applicable checks—not that you have memorized the whole documentation set.
