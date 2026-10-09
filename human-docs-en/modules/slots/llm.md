# Slot pack · `llm` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/llm.md`](../../../human-docs/modules/slots/llm.md)
> Scope: developer entry points and checklist. Public entry points: [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页); reference Host definitions: [§8](../../../handoff/MODULE_MAP_AND_HANDOFF.md#8-第-5-模块--llm).
> Updated: 2026-10-09.

## Choose your entry point

Start below for basic text generation. Use **Reference Host** for `LlmClient`, streaming, directory plugins or blueprint backends; its configuration and product checks are not automatically prerequisites for every Base implementation.

### LlmBase: use the caller-selected generation capability

1. Find `LlmBase::generate` in the [public contract](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs). It receives prepared input; the caller composes the model, client and resource grants. Request text grants no authority.
2. The reference Host exposes [`OcliveKernel::text_generation_base()`](../../../kernel/crates/oclive_kernel_host/src/role_kernel.rs). Its internal [`HostTextGenerationBase`](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/minimal_llm.rs) borrows the existing client without rebuilding a provider. There is no separate runtime `base_llm.rs`; independent Hosts can implement the public trait without copying the reference Host.
3. Follow the explicit Prompt→LLM operation in [`MinimalRoleTextConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs), or the six-Base consumer's independent `generate` delegation. The former calls LLM only after normal Prompt completion; the latter does not implicitly call Prompt. Existing in-memory assembly is shown in [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs).

**Base checklist**:

- [ ] Match the selected implementation to its input agreement; keep empty text and complete failures, without inferring timeout or cancellation kinds from diagnostic wording.
- [ ] Keep resources, grants and the executor caller-owned. Dropping the future does not prove remote execution stopped or retry is safe.

Definitions remain in the [entry-point checklist](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页). Basic text results do not promise SSE, a unified remote wire or real model quality; validate those for the actual Host and provider.

## Reference Host

**You plug in**: blueprint `slot_registry.type: llm` → folded `PluginBackends.llm` (legacy v1: `settings.json.plugin_backends.llm`) · trait `LlmClient` · hook `co_present` generate/stream.

**Do**: Ollama / remote / directory backends · blueprint `slot_registry` · for multiple LLMs, respect the last LLM entry's non-streaming `policy`: default `ensemble` (serial last-wins), `fastest`, or `fallback`. Streaming currently normalizes all three to serial last-wins.

**Don't**: Call LLM from UI for portrait pick · use `none` on co-present path · stack logic in Tauri `api/*.rs`.

**Read next**: [PLUGIN_V1](../../../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) (if mirrored) or [ZH PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [DIRECTORY_PLUGINS](../../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md).
