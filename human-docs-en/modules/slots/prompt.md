# Slot pack · `prompt` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/prompt.md`](../../../human-docs/modules/slots/prompt.md)
> Scope: developer entry points and checklist. Public entry points: [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页); reference Host definitions: [§7](../../../handoff/MODULE_MAP_AND_HANDOFF.md#7-第-4-模块--prompt).
> Updated: 2026-10-09.

## Choose your entry point

Start below for basic assembly. Use **Reference Host** for `PromptBuilder`, overlays or blueprint backends; its rich-path checks are not automatically prerequisites for every Base implementation.

### PromptBase: assembly and consumption

1. Find `PromptBase::assemble` in the [public contract](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs). Read the existing [`LiteralMaterialAssembler`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_prompt.rs): it concatenates materials verbatim in order and rejects non-empty extra requirements as `Unsupported`. This is the implementation's agreement, not a mandatory algorithm for all Prompts.
2. Follow [`MinimalRolePromptConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs), which prepares the persona and current materials before delegating to the caller-selected Prompt. The selected implementation should not add that persona again; headings are not a downstream injection guarantee.
3. The reference Host exposes [`process_minimal_message_with_prompt`](../../../kernel/crates/oclive_kernel_host/src/role_kernel.rs). See [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs) for independent Host assembly; this navigation does not promise configuration-based plugin registration.

**Base checklist**:

- [ ] State accepted materials and requirements; check original text, order, empty results and complete failures through the consumer.
- [ ] The caller selects capabilities, materials and the executor. Assembly grants no resource authority and imposes no fixed six-slot turn.

Definitions remain in the [entry-point checklist](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页). This navigation does not establish real model output quality.

## Reference Host

**You plug in**: blueprint `slot_registry.type: prompt` → folded `PluginBackends.prompt` (legacy v1: `settings.json.plugin_backends.prompt`) · trait `PromptAssembler` → builtin **`PromptBuilder::build_prompt`** · hook `co_present` `BuildPrompt` · `PromptInput`. Code SSOT: `kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/`.

**Do**: Edit `sections.rs` formulas · concise overlay · package-level `reply_quality_anchor` (replaces default anchor only) · `builtin` / `remote` / `directory` · `prompts/deep_capsule.txt` (Wave D, wired).

**Don't**: Runtime LLM prompt compression · replace `KERNEL_DIALOGUE_GUARDRAILS` with capsule · `none` backend on co-present path · return `Result` from `build_prompt` (must return `String`).

**Read next**: [04 engineering rules §5–§6 (ZH)](../../../human-docs/04_ENGINEERING_RULES.md) · [DEEP_PROMPT_DISTILLATION](../../../handoff/DEEP_PROMPT_DISTILLATION.md) · [role-pack-content](../packs/role-pack-content.md).
