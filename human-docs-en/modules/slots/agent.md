# Slot pack · `agent` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/agent.md`](../../../human-docs/modules/slots/agent.md)
> Definition SSOT: [MODULE_MAP §9](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: blueprint `slot_registry.type: agent` → folded `PluginBackends.agent` (legacy v1: `settings.json.plugin_backends.agent`) · trait `AgentProvider` · ordinary user/non-staged turns try it before `turn_pipeline::pre`; `handled=true` uses `minimal_response` and skips Stable pre / middle / LLM / post.

`agent_context::build_agent_input` independently supplies recent turns, relation key/state, favorability, personality, and MCP tools. The shortcut's `minimal_response` separately invokes the emotion slot and commits minimal state/chat, but it does not run Reply Post-Processor or Reply Mode.

**Do**: MCP with user grants (`network:*`, `process:spawn`) · treat the folded single Agent as the current executable behavior. Multiple Agent entries / `plugins[]` currently expose diagnostic ids only; tool-union execution remains technical debt `K-AGENT-MERGE-01`.

**Don't**: Skip MCP authorization · put ASR in agent slot · bypass `host_flags.skip_agent` · assume Stable memory/prompt stages or post-processing ran on a shortcut.

**Read next**: [PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [EXTENSION_POINTS](../../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md).
