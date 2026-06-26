# 六槽开工包 · `event`

> **读者**：改事件检测、影响因子或 event 后端的工程师。  
> **读完能做什么**：理解规则路径 vs LLM 路径，以及 `event_impact_llm` 开关（HostProfile，非六槽）。  
> **耗时**：约 **45 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §6](../../handoff/MODULE_MAP_AND_HANDOFF.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[prompt](prompt.md) · [orchestration/turn-thinking](../orchestration/turn-thinking.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§6 第 3 模块 · `event`](../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event)  
- **`plugin_backends` 键**：`event`  
- **Trait**：`EventEstimator`  
- **主链 hook**：`co_present` `EventEstimate` stage → `PersonalityEngine::evolve_by_event`

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 规则表 `EventDetector` · LLM `estimate_event_impact` | 把 Turn Thinking 登记为第七槽 |
| `remote` / `directory` backend | 在 Fast 轮强行走 LLM 路径（受 HostProfile 约束） |
| `builtin` 双路径（规则 / LLM） | 角色任务改 `slot_registry`（G1） |

**LLM 开关**：`HostProfile.event_impact_llm` — 见 [DISTRO_CAPABILITY_PROFILE](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)。

---

## 3. 阅读清单

1. [MODULE_MAP §6](../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event)  
2. [MODULE_MAP §12 `event_impact_llm`](../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)  
3. `event_impact_ai.rs` · `EventDetector` 源码  
4. [RFC_TURN_THINKING](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) — Fast 轮不调 LLM event 路径  
5. [PLUGIN_V1](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)

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

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `emotion` | pre 阶段并行输入 |
| PersonalityEngine | 好感 / 性格演化 |
| [turn-thinking](../orchestration/turn-thinking.md) | Fast 档跳过部分 LLM 调用 |
| `memory` | 强事件仍可能写 LTM（与 Fast 策略交叉） |
