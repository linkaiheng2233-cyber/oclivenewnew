# 六槽开工包 · `agent`

> **读者**：开发 ReAct / MCP 工具编排或 Agent 目录插件的工程师。
> **读完能做什么**：接入 `agent` 槽、配置 MCP，理解短路主链语义。
> **耗时**：约 **45 min**
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §9](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-09-05
> **下一篇**：[llm](llm.md) · [memory](memory.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§9 第 6 模块 · `agent`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#9-第-6-模块--agent)
- **当前配置 / 运行时折叠**：蓝图 `slot_registry.type: agent` → `PluginBackends.agent`；legacy v1 才是 `settings.json.plugin_backends.agent`
- **Trait**：`AgentProvider`（`oclive_kernel_contracts`）
- **主链 hook**：普通用户、非 staged 回合在 `turn_pipeline::pre` 之前尝试；`handled=true` 时走 `minimal_response` 并 **短路** Stable `pre / middle / llm / post`

Agent 输入由 `agent_context::build_agent_input` 独立组装近期对话、关系键、好感、人格与 MCP 工具，不表示 Stable 的 memory / prompt 阶段已经执行。短路后的 `minimal_response` 会单独调用 emotion 槽并提交最小状态与聊天记录，但不会进入 Reply Post-Processor 或 Reply Mode。

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| Agent 协议、MCP 客户端、调试 trace、`directory` / `remote` backend | 跳过 MCP `network:*` / `process:spawn` 用户授权 |
| 单一 Agent backend、MCP 工具与授权边界 | 把尚未实现的多 Agent 工具并集当成现状；把 ASR / 语音写进 agent 槽 |
| `builtin` · `remote` · `directory` · `none` | 发行版 `host_flags.skip_agent` 时强制 `none` — 勿硬绕 |

---

## 3. 阅读清单

1. [MODULE_MAP §9](../../../handoff/MODULE_MAP_AND_HANDOFF.md#9-第-6-模块--agent)
2. [PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) — Agent 阶段与短路
3. [EXTENSION_POINTS](../../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)
4. [BUS_FACTOR §1](../../../handoff/BUS_FACTOR_NOTES.md) — Agent 分支锚点
5. MCP 配置：`{app_data}/mcp-servers/*.json` · `high_risk_grants.json`

---

## 4. 开发流程

- [ ] 确认发行版未设 `skip_agent`（见 [DISTRO_CAPABILITY_PROFILE](../../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)）
- [ ] 实现或选用 `AgentProvider` backend
- [ ] MCP server JSON + 用户授权流程走通
- [ ] 蓝图 `slot_registry` 声明 `type: agent`（**非**角色包任务，G1）
- [ ] 验证短路：工具回合可能不再进入 LLM 闲聊链
- [ ] 验证短路输出保持原文，不误期待 Reply Post-Processor / Reply Mode 运行
- [ ] `npm run check` · 相关 `invoke_hotpath_matrix` 测试

---

## 5. 验收

- [ ] 授权前 MCP 调用被拒绝；授权后可工具调用
- [ ] 当前实际只调用折叠后的单一 Agent；若声明多个 Agent/`plugins[]`，不得把诊断 ID 集合误当成已执行的工具并集（见 `K-AGENT-MERGE-01`）
- [ ] 短路回合响应仍走 DTO **`reply`** 契约
- [ ] 未把渗透 / VS Code 逻辑塞进 agent 槽

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `llm` | 非短路时下游生成自然语言 |
| `emotion` | `minimal_response` 为响应字段与最小提交单独分析用户句 |
| `memory` / `prompt` | Stable pre / middle 被跳过；Agent 只消费自己的上下文 DTO |
| 独立通道 | 当前短路不进入 `reply_post_process` / `reply_mode` |
