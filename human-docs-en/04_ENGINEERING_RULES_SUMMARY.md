# Engineering rules (English summary)

> Full Chinese SSOT: [human-docs/04_ENGINEERING_RULES.md](../human-docs/04_ENGINEERING_RULES.md) (includes **§8 documentation discipline — human edition**).

## Non-negotiables (code)

1. **Orchestration** lives in `kernel/crates/oclive_kernel_host/.../process_message.rs` — do not add stages from API or Tauri layers.
2. **Persistence** goes through `domain/repository.rs` traits; SQL schema is `migrations/001_init.sql` — no invented table names.
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
| Human vs AI progress | [human-docs README §进度](../human-docs/README.md#文档包进度与-ai-包同步--2026-06-25) · code debt [TECHNICAL_DEBT §1](../handoff/TECHNICAL_DEBT_INVENTORY.md) |
| AI agents | [AI_CHANGE_BOUNDARIES G10–G16](../handoff/AI_CHANGE_BOUNDARIES.md) |

When changing architecture in a PR: update MODULE_MAP (if modules change) + relevant human-docs sections + progress date in human-docs README.

## PR gates

See [08_PR_GATE_MATRIX.md](08_PR_GATE_MATRIX.md) and run `npm run check:ci-local` before opening a PR.
