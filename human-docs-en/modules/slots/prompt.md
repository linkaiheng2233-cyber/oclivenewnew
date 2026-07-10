# Slot pack · `prompt` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/prompt.md`](../../human-docs/modules/slots/prompt.md)  
> Definition SSOT: [MODULE_MAP §7](../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `plugin_backends` key `prompt` · trait `PromptAssembler` → builtin **`PromptBuilder::build_prompt`** · hook `co_present` `BuildPrompt` · `PromptInput`. Code SSOT: `kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/`.

**Do**: Edit `sections.rs` formulas · concise overlay · package-level `reply_quality_anchor` (replaces default anchor only) · `builtin` / `remote` / `directory` · `prompts/deep_capsule.txt` (Wave D, wired).

**Don't**: Runtime LLM prompt compression · replace `KERNEL_DIALOGUE_GUARDRAILS` with capsule · `none` backend on co-present path · return `Result` from `build_prompt` (must return `String`).

**Read next**: [04 engineering rules §5–§6 (ZH)](../../human-docs/04_ENGINEERING_RULES.md) · [DEEP_PROMPT_DISTILLATION](../../handoff/DEEP_PROMPT_DISTILLATION.md) · [role-pack-content](../packs/role-pack-content.md).
