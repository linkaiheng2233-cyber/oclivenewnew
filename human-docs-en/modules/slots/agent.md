# Slot pack · `agent` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/agent.md`](../../human-docs/modules/slots/agent.md)  
> Definition SSOT: [MODULE_MAP §9](../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `plugin_backends` key `agent` · trait `AgentProvider` · may **short-circuit** `process_message`.

**Do**: MCP with user grants (`network:*`, `process:spawn`) · merge multiple agents as tool **union**.

**Don't**: Skip MCP authorization · put ASR in agent slot · bypass `host_flags.skip_agent`.

**Read next**: [PLUGIN_V1](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [EXTENSION_POINTS](../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md).
