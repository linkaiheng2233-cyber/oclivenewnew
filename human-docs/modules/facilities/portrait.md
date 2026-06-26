# 设施开工包 · 立绘

> **读者**：Chat Pro 立绘 catalog、`visual_state_id` 与表现导演链路的工程师。  
> **读完能做什么**：在 RFC 边界内改 post_llm 立绘设施，不把选图逻辑塞进 UI LLM。  
> **耗时**：约 **45 min**  
> **SSOT 范围**：人类 checklist；RFC 见 [RFC_PORTRAIT_FACILITY](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[visual-stage](visual-stage.md) · [team/TRACK_VISUAL_UPGRADE](../../team/TRACK_VISUAL_UPGRADE.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§10 设施③ 立绘](../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)  
- **默认**：**关**（HostProfile / 角色包启用）  
- **主链 hook**：`post_llm` · 表现导演 LLM → 封闭 catalog → `visual_state_id`  
- **RFC**：[RFC_PORTRAIT_FACILITY](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md)

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| catalog 资源 · `visual_state_id` 映射 | UI 内二次调 LLM 选立绘（MODULE_MAP §8 禁止） |
| post_llm 设施接线 | 写入 `plugin_backends` 六键 |
| 姊妹仓 pack-editor 立绘编辑 | 把 Live2D 未决 SDK 当已实现 truth |

---

## 3. 阅读清单

1. [RFC_PORTRAIT_FACILITY](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md)  
2. [MODULE_MAP §10 设施③](../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)  
3. [team/TRACK_VISUAL_UPGRADE](../../team/TRACK_VISUAL_UPGRADE.md)  
4. [PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN](../../handoff/PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN.md)（阶段笔记）  
5. [theater/DEVELOPMENT_ROADMAP](../../handoff/theater/DEVELOPMENT_ROADMAP.md)（剧场线交叉时）

---

## 4. 开发流程

- [ ] 读 RFC 当前 Wave / 默认关  
- [ ] 改 catalog 或 post_llm 锚点  
- [ ] 前端消费 `visual_state_id` → 链 [visual-stage](visual-stage.md)  
- [ ] 垂直 sprint 走 [team/SCOPE_AND_BOUNDARIES](../../team/SCOPE_AND_BOUNDARIES.md)  
- [ ] `npm run test:unit` · 相关 Rust 测试

---

## 5. 验收

- [ ] 选图经封闭 catalog，非任意 URL  
- [ ] llm 槽未承担立绘选择  
- [ ] PR 链 RFC，未复制 MODULE_MAP 设施表全文

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [visual-stage](visual-stage.md) | `visual_state_id` → 帧循环表现 |
| `llm` | 表现导演 LLM 子调用（设施内） |
| `reply` DTO | 前端展示与立绘状态并行 |
