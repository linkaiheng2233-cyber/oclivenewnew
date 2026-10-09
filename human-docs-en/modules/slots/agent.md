# Slot pack · `agent` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/agent.md`](../../../human-docs/modules/slots/agent.md)
> Scope: developer entry points and checklist. Public entry points: [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页); reference Host definitions: [§9](../../../handoff/MODULE_MAP_AND_HANDOFF.md#9-第-6-模块--agent).
> Updated: 2026-10-09.

## Choose your entry point

Start below for basic delegated tasks. Use **Reference Host** for `AgentProvider`, MCP or shortcut turns; its configuration and context are not automatically prerequisites for every Base implementation.

### AgentBase: distinguish a pure example from an authorized task

1. Find `AgentBase::execute` in the [public contract](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs). The existing [`ScalarCountAgent`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_agent.rs) supports only its published counting task, calls no model or tool, and is not a production Host backend. Its task vocabulary is not a general Base command set.
2. Follow explicit task delegation in [`MinimalRoleBaseConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs). For the reference Host's existing ReAct capability, use [`BuiltinReActAgent::task_execution_base`](../../../kernel/crates/oclive_kernel_host/src/domain/agent.rs), borrowing the current model, actual role/session identity and existing tool grants. Details are in [`agent_base_binding.rs`](../../../kernel/crates/oclive_kernel_host/src/domain/agent_base_binding.rs).
3. See [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs) for pure assembly. Ordinary minimal chat does not automatically execute Agent; borrow the selected implementation for an explicit task and retain the current single-Agent scope.

**Base checklist**:

- [ ] State supported tasks, result meaning and resource scope; preserve complete failures. A textual report does not prove objective completion, effects rollback or safe retry.
- [ ] Request text grants no tool authority. Calls retain existing grants; dropping the future does not prove a tool or model stopped.

Definitions remain in the [entry-point checklist](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页). This navigation implements no multi-Agent merge or automatic replacement of configured remote/directory backends.

## Reference Host

**You plug in**: blueprint `slot_registry.type: agent` → folded `PluginBackends.agent` (legacy v1: `settings.json.plugin_backends.agent`) · trait `AgentProvider` · ordinary user/non-staged turns try it before `turn_pipeline::pre`; `handled=true` uses `minimal_response` and skips Stable pre / middle / LLM / post.

`agent_context::build_agent_input` independently supplies recent turns, relation key/state, favorability, personality, and MCP tools. The shortcut's `minimal_response` separately invokes the emotion slot and commits minimal state/chat, but it does not run Reply Post-Processor or Reply Mode.

**Do**: MCP with user grants (`network:*`, `process:spawn`) · treat the folded single Agent as the current executable behavior. Multiple Agent entries / `plugins[]` currently expose diagnostic ids only; tool-union execution remains technical debt `K-AGENT-MERGE-01`.

**Don't**: Skip MCP authorization · put ASR in agent slot · bypass `host_flags.skip_agent` · assume Stable memory/prompt stages or post-processing ran on a shortcut.

**Read next**: [PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [EXTENSION_POINTS](../../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md).
