# Engineering rules (English summary)

> Full Chinese SSOT: [human-docs/04_ENGINEERING_RULES.md](../human-docs/04_ENGINEERING_RULES.md)

## Non-negotiables

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

## PR gates

See [08_PR_GATE_MATRIX.md](08_PR_GATE_MATRIX.md) and run `npm run check:ci-local` before opening a PR.
