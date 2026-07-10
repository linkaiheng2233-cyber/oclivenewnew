# 设施开工包 · 视觉表现

> **读者**：改 `performance_directive`、宿主 UI 帧循环与视觉表现的工程师。  
> **读完能做什么**：在设施④边界内改表现层，**无** AI 选图。  
> **耗时**：约 **40 min**  
> **SSOT 范围**：人类 checklist；RFC 见 [RFC_VISUAL_PRESENTATION](../../creator-docs/rfc/RFC_VISUAL_PRESENTATION.md)（草案链 RFC_PORTRAIT 族）  
> **最后更新**：2026-06-26  
> **下一篇**：[portrait](portrait.md) · [surfaces/frontend-chat-pro](../surfaces/frontend-chat-pro.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§10 设施④ 视觉表现](../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)  
- **输入**：`visual_state_id`（来自设施③）  
- **输出**：`performance_directive` → 宿主 UI 帧循环  
- **默认**：**关**

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| Vue 帧循环 · 表现指令消费 | 在视觉层调用 LLM 选图 |
| 与 [portrait](portrait.md) 的 ID 契约 | 写入六槽 `plugin_backends` |
| Chat Pro `distros/chat-pro` UI 表现 | 渗透进内核 `process_message` 顺序 |

---

## 3. 阅读清单

1. [MODULE_MAP §10 设施④](../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)  
2. [RFC_PORTRAIT_FACILITY](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md) — 表现导演与舞台  
3. [team/TRACK_VISUAL_UPGRADE](../../team/TRACK_VISUAL_UPGRADE.md)  
4. [surfaces/frontend-chat-pro](../surfaces/frontend-chat-pro.md)  
5. [01 简架构 §设施](../../01_ARCHITECTURE_SIMPLE.md)

---

## 4. 开发流程

- [ ] 确认 `visual_state_id` 由 portrait 设施产出  
- [ ] 改前端表现组件 / directive 解析  
- [ ] 不测 LLM 选图 — 只测 catalog ID → 表现  
- [ ] `npm run test:unit` · `npm run build`

---

## 5. 验收

- [ ] UI 帧循环不发起模型推理选图  
- [ ] 设施默认关时 UI 优雅降级  
- [ ] 与 portrait 包分工清晰

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [portrait](portrait.md) | 上游 `visual_state_id` |
| [frontend-chat-pro](../surfaces/frontend-chat-pro.md) | 宿主渲染 |
| `llm` | **无**直接选图关系 |
