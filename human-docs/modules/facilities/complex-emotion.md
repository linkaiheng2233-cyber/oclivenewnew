# 设施开工包 · 复杂情感

> **读者**：改 `narrative_hint`、复杂情感叙事链路的工程师。  
> **读完能做什么**：区分 **emotion 六槽** 与本设施，在 `complex_emotion.rs` 边界内改动。  
> **耗时**：约 **40 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §10 设施①](../../handoff/MODULE_MAP_AND_HANDOFF.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[slots/emotion](../slots/emotion.md) · [prompt](../slots/prompt.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§10 设施① 复杂情感](../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)  
- **非六键**：**不**写入 `plugin_backends` 六键  
- **代码锚点**：`complex_emotion.rs` · `turn_pipeline/pre.rs`  
- **输出**：`PromptInput.previous_complex_emotion_narrative_hint`（下一轮消费）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| `complex_emotion.rs` 叙事逻辑 | 在 `slot_registry` 加 `complex_emotion` 键 |
| 默认 on（可 skip 标志） | 与 emotion 槽合并为一个「情绪模块」文档 |
| 消费 emotion + 上下文 | RFC 未登记前 silent 扩成第七槽 |

---

## 3. 阅读清单

1. [MODULE_MAP §10](../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)  
2. [slots/emotion](../slots/emotion.md)  
3. [slots/prompt](../slots/prompt.md)  
4. `complex_emotion.rs` 源码  
5. [AI_CHANGE_BOUNDARIES G1](../../handoff/AI_CHANGE_BOUNDARIES.md) — 新设施须 RFC

---

## 4. 开发流程

- [ ] 确认需求属于设施而非 emotion 槽分析器  
- [ ] 改 `complex_emotion.rs` + pre 注入  
- [ ] 单测 roundtrip（参考 `narrative_hint_prompt_roundtrip`）  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] `plugin_backends` 无 `complex_emotion` 键  
- [ ] 下一轮 Prompt 收到 `narrative_hint`  
- [ ] MODULE_MAP §10 仍准确描述行为

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `emotion` | 上游用户句情绪 |
| `prompt` | 下游段落注入 |
| Turn Thinking | Auto 路由可能读情绪上下文 |
