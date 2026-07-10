# 06 · Kernel learning path (Day 1–5 summary)

> **Last updated:** 2026-06-26  
> **Audience:** Contributors touching `process_message`, persistence, or plugin wiring.  
> **Full human edition (CN):** [human-docs/06](../human-docs/06_KERNEL_LEARNING_PATH.md) · **Module registry SSOT:** [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md)

---

## Top 6 before a kernel PR

1. [kernel/crates/README.md](../kernel/crates/README.md) — dependency graph  
2. [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md) — module registry · per-slot boundaries  
3. [BUS_FACTOR_NOTES.md §0–2](../handoff/BUS_FACTOR_NOTES.md) — `process_message`, `PluginHost`  
4. [01 Architecture (simple)](01_ARCHITECTURE_SIMPLE.md) — three memory stores · six slots  
5. [NAMING_CONVENTIONS §4.2](../creator-docs/NAMING_CONVENTIONS.md#42-canonical-import-路径)  
6. [CONTRIBUTING.md §Tests](../CONTRIBUTING.md#测试要求合并前建议全绿)

---

## Day 1 · Run + vocabulary (~half day)

| Step | Doc / action | Done when |
|------|--------------|-----------|
| 1 | [02 Thirty-minute start](02_THIRTY_MINUTE_START.md) | `npm run check` green |
| 2 | [03 Glossary](03_GLOSSARY.md) + [04 Engineering rules](04_ENGINEERING_RULES_SUMMARY.md) | Explain `srid`, `reply`, six slots, **three memory stores** |
| 3 | Skim `process_message.rs` header comments | Name Agent / co-present / remote branches |

---

## Day 2 · Main chain (~1 day)

| Order | File | Focus |
|-------|------|--------|
| 1 | `process_message.rs` | `run()`: `srid`, health check, branches |
| 2 | `turn_pipeline/mod.rs` | `execute_turn` four phases |
| 3 | `turn_pipeline/pre.rs` | Prompt input, complex emotion, **memory retrieval** |
| 4 | [MODULE_MAP §4–§9](../handoff/MODULE_MAP_AND_HANDOFF.md) + `plugin_host/mod.rs` | Per-slot wiring |
| 5 | `prompt_builder/mod.rs` | Section order, guardrails |

**Checkpoint:** sketch Tauri → `process_message` → `turn_pipeline` → `PluginHost`.

---

## Day 3 · Persistence & errors (~1 day)

| Topic | Entry |
|-------|--------|
| Repository trait | `domain/repository.rs` |
| Implementations | `infrastructure/repositories.rs` |
| Migrations | `kernel/crates/oclive_kernel_host/migrations/` |
| Error codes | `AppError::to_kernel_json()`, [ERROR_CODES](../creator-docs/getting-started/ERROR_CODES.md) |

**Checkpoint:** new DB field → migration SQL first, then trait/impl.

---

## Day 4 · Tests & debugging (~half day)

| Type | Command / location |
|------|-------------------|
| Daily gates | `npm run check` |
| Release | `npm run check:release` |
| Domain unit tests | `AppState::new_in_memory_with_llm` (see [07 First PR](07_FIRST_PR.md)) |
| Logs | [05 Debugging](../human-docs/05_DEBUGGING.md) (CN) |

---

## Day 5 · First PR draft (~half day)

Start with a **pure domain unit test** or **doc-only** change (zero behavior change).

Flow: [CONTRIBUTING §PR](../CONTRIBUTING.md#pr-流程) · Dimension 5: `node scripts/dimension5-acceptance.mjs --ci`

**Checkpoint:** PR description lists motivation, self-check commands, linked tests.

---

## L6 · Common tasks

Task recipes (CN SSOT): [human-docs/07_COMMON_TASKS.md](../human-docs/07_COMMON_TASKS.md) · English first-PR shortcut: [07_FIRST_PR.md](07_FIRST_PR.md).
