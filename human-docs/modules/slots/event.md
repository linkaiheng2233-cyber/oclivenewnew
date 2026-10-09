# 六槽开工包 · `event`

> **读者**：编写基础事件分析实现，或维护参考 Host 的影响估计与 event 后端的工程师。
> **读完能做什么**：区分 Base 按需分析、legacy event.impact 与 Event Ring，找到各自接入路径。
> **耗时**：基础入口为短导航；参考 Host 流程约 **45 min**
> **SSOT 范围**：人类路由与 checklist；公共接入点见 [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，参考 Host 定义见 [§6](../../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event)
> **最后更新**：2026-10-09
> **下一篇**：[prompt](prompt.md) · [orchestration/turn-thinking](../orchestration/turn-thinking.md)

---

## 0. 先选接入路径

基础按需分析从下方开始；维护 `EventEstimator`、影响估计或 legacy 兼容桥则读 **§1–§6**。丰富路径的状态应用和策略不自动成为所有 Base 实现的前置要求。

### EventBase：调用前看清分析成本与结果边界

1. 在[公共契约](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs)找到 `EventBase::analyze`。现有 [`LlmEventAnalyzer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_event.rs) 借用调用方选定的 `LlmBase`，实际分析调用生成器一次；它的私有输入/回复约定不是 Base 通用协议，选定生成器须与之相容。
2. 看[`MinimalRoleBaseConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs)的独立 Event 委托。参考 Host 通过 [`OcliveKernel::event_analysis_base()`](../../../kernel/crates/oclive_kernel_host/src/role_kernel.rs) 提供显式借用，内部接线在 [`minimal_event.rs`](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/minimal_event.rs)。构造视图不发起分析；普通聊天不会因提供此入口自动追加一次分析。
3. 实现与消费例子可读 [`base_only_fixture.rs`](../../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs) 和 [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs)。有具体需求再选择与调度，不为跑齐六槽强行调用。

**基础路径 checklist**：

- [ ] 明确实现的分析约定和调用成本；区分正常无分析正文与失败，保留生成器完整错误和格式错误。
- [ ] Host 决定结果使用；Base 分析不发布 Ring 事件、不应用角色状态，也不签发主动回合许可。

请求、失败和绑定查[接入点清单](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。参考分析实现不承诺真实模型已遵守私有格式或具有足够分析质量。

---

## 1. 参考 Host：你插在哪

- **MODULE_MAP**：[§6 第 3 模块 · `event`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event)
- **当前配置 / 运行时折叠**：蓝图 `slot_registry.type: event` → `PluginBackends.event`；legacy v1 才是 `settings.json.plugin_backends.event`
- **Trait**：`EventEstimator`
- **主链 hook**：`co_present/run_middle.rs` `EventEstimate` → `publish_legacy_event_impact` → `PersonalityEngine::evolve_by_event`

本槽只估计对话事件影响。Event Ring 的信封、注册、决策模块与主动回合不属于本槽，见 [EVENT_RING](../../../creator-docs/plugin-and-architecture/EVENT_RING.md)。

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 规则表 `EventDetector` · LLM `estimate_event_impact` | 把 Turn Thinking 登记为第七槽 |
| `remote` / `directory` backend | 在 Fast 轮强行走 LLM 路径（受 HostProfile 约束） |
| `builtin` 双路径（规则 / LLM） | 把本槽称为整个 Event Ring 或自行签发主动回合许可 |

**LLM 开关**：`HostProfile.event_impact_llm` — 见 [DISTRO_CAPABILITY_PROFILE](../../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)。

---

## 3. 阅读清单

1. [MODULE_MAP §6](../../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event)
2. [MODULE_MAP §12 `event_impact_llm`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)
3. `event_impact_ai.rs` · `EventDetector` 源码
4. [RFC_TURN_THINKING](../../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) — Fast 轮不调 LLM event 路径
5. [PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)
6. [EVENT_RING](../../../creator-docs/plugin-and-architecture/EVENT_RING.md) — legacy 兼容桥与四层权威

---

## 4. 开发流程

- [ ] 区分规则路径与 LLM 路径改动
- [ ] 若涉发行版默认 → `distro.oclive.toml` HostProfile
- [ ] 改 evolve 逻辑 → `PersonalityEngine` + event 产出
- [ ] 单测覆盖规则表边界
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] Fast Turn Thinking 下 LLM event 路径符合 HostProfile
- [ ] 强事件（如 Quarrel）持久化行为符合 RFC
- [ ] 未新增六槽外 `plugin_backends` 键
- [ ] 无 Event 模块时 Ring 兼容桥保持估计结果不变

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `emotion` | pre 阶段并行输入 |
| PersonalityEngine | 好感 / 性格演化 |
| [turn-thinking](../orchestration/turn-thinking.md) | Fast 档跳过部分 LLM 调用 |
| `memory` | 强事件仍可能写 LTM（与 Fast 策略交叉） |
