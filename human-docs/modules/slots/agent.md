# 六槽开工包 · `agent`

> **读者**：编写基础任务实现，或维护参考 Host 的 ReAct / MCP 与 Agent 目录插件的工程师。
> **读完能做什么**：选择 Base 委托任务或丰富 Agent 路径，辨认案例、生产借用和授权边界。
> **耗时**：基础入口为短导航；参考 Host 流程约 **45 min**
> **SSOT 范围**：人类路由与 checklist；公共接入点见 [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，参考 Host 定义见 [§9](../../../handoff/MODULE_MAP_AND_HANDOFF.md#9-第-6-模块--agent)
> **最后更新**：2026-10-09
> **下一篇**：[llm](llm.md) · [memory](memory.md)

---

## 0. 先选接入路径

基础委托任务从下方开始；维护 `AgentProvider`、MCP 或参考 Host 短路流程则读 **§1–§6**。丰富路径的配置和上下文不自动成为所有 Base 实现的前置要求。

### AgentBase：区分纯案例与已有授权下的真实任务

1. 在[公共契约](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs)找到 `AgentBase::execute`。现有 [`ScalarCountAgent`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_agent.rs) 只支持其声明的计数任务，不调用模型或工具，也不是生产 Host 后端；任务词汇不是 Base 通用指令集。
2. 看[`MinimalRoleBaseConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs)如何直接委托明确任务。需要参考 Host 已有 ReAct 能力时，入口是 [`BuiltinReActAgent::task_execution_base`](../../../kernel/crates/oclive_kernel_host/src/domain/agent.rs)，借用当前模型、实际角色/会话身份与已有工具授权；接线细节见 [`agent_base_binding.rs`](../../../kernel/crates/oclive_kernel_host/src/domain/agent_base_binding.rs)。
3. 纯装配例子见 [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs)。普通最小聊天不会自动执行 Agent；有明确任务再借用选定实现，保持当前单 Agent 范围。

**基础路径 checklist**：

- [ ] 先说明支持的任务、结果含义和资源范围；保留完整失败，不把文字报告当作目标完成、效果回滚或安全重试证明。
- [ ] 请求文本不授予工具权限；实际调用仍守已有授权，丢弃 future 不当作工具或模型已经停止。

请求、失败和绑定查[接入点清单](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。本导航不实现多 Agent 合并，也不自动替换配置的 remote / directory 后端。

---

## 1. 参考 Host：你插在哪

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
