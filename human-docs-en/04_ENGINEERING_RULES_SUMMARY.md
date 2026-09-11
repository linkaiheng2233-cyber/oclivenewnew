# Engineering rules (English summary)

> **Last updated:** 2026-09-11 (documentation ownership and reference-Host scope)
> Full Chinese SSOT: [human-docs/04_ENGINEERING_RULES.md](../human-docs/04_ENGINEERING_RULES.md) (includes **§8 documentation discipline — human edition**).

**Scope clarification (2026-09-11):** the code rules below apply to the current reference Host. They do not define a fixed pipeline or domain model for every small-Kernel integration; see [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities).

## Non-negotiables (code)

1. **Orchestration** lives in `kernel/crates/oclive_kernel_host/.../process_message.rs` — do not add stages from API or Tauri layers.
2. **Persistence** follows the current repository ports and actual migrations; do not infer the complete current schema from the initial migration or invent tables.
3. **Tauri commands** in `distros/desktop-tauri/src/api/*.rs`, registered only in `lib.rs` via `generate_handler!`.
4. **DTO contract** is `oclive_kernel_types::models::dto` — reply field is **`reply`**, not `response`.
5. **Prompt** via `PromptBuilder::build_prompt(input: &PromptInput<'_>) -> String` (not `Result`). Pack `reply_quality_anchor` cannot replace `KERNEL_DIALOGUE_GUARDRAILS`.

## Import SSOT

See [creator-docs/NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md) §4.2:

- DTO → `oclive_kernel_types`
- Traits → `oclive_kernel_contracts`
- Orchestration → `oclive_kernel_host`

## Documentation discipline

**Efficiency comes from constraints.** Human docs may be long and readable; AI docs stay short and link out.

| Rule | Action |
|------|--------|
| Find SSOT first | [handoff/README §文档分责](../handoff/README.md) before creating any new `.md` |
| Module / slot map | Edit only [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md) |
| No duplicate tables | Link instead of copying PLUGIN_V1 / MODULE_MAP |
| Learning vs status | [Human learning route](README.md); gaps/pauses stay in [TECHNICAL_DEBT](../handoff/TECHNICAL_DEBT_INVENTORY.md), not a second progress table |
| AI agents | [AI_CHANGE_BOUNDARIES G10–G16](../handoff/AI_CHANGE_BOUNDARIES.md) |

`creator-docs/` serves users/authors/integrators, not AI alone. When architecture changes, follow the [maintenance loop](../handoff/README.md#documentation-maintenance): update the fact owner, relevant learning pages, AI routes, and existing English mirrors. Do not duplicate progress tables.

## PR gates

See [08_PR_GATE_MATRIX.md](08_PR_GATE_MATRIX.md) and run `npm run check:ci-local` before opening a PR.
