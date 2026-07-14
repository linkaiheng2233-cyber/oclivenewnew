# 六槽开工包 · `emotion`

> **读者**：改用户句情绪分析或 emotion 后端的工程师。  
> **读完能做什么**：区分 **emotion 六槽** 与 **复杂情感设施**，在边界内改分析器。  
> **耗时**：约 **40 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §5](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-07-14
> **下一篇**：[facilities/complex-emotion](../facilities/complex-emotion.md) · [event](event.md)

---

## 1. 你插在哪

- **T0 / T1+ 分层（Draft RFC）**：[RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md](../../../creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md) — 最小闭环 = `analyze`；模拟与展示指标为扩展  
- **MODULE_MAP**：[§5 第 2 模块 · `emotion`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#5-第-2-模块--emotion)
- **`plugin_backends` 键**：`emotion`  
- **Trait**：`UserEmotionAnalyzer`  
- **主链 hook**：`turn_pipeline/pre.rs` → `EmotionResult` → Prompt · Turn Thinking Auto

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 分析器、`remote` / `directory` 协议 | 把 `complex_emotion` 写入 `slot_registry` 冒充六槽 |
| `builtin` · `remote` · `directory` · `none` | 与 [complex-emotion 设施](../facilities/complex-emotion.md) 混为一谈 |

**区分**：本槽分析 **用户句** 情绪；复杂情感设施消费 emotion 产出 → `narrative_hint`。

---

## 3. 阅读清单

1. [MODULE_MAP §5](../../../handoff/MODULE_MAP_AND_HANDOFF.md#5-第-2-模块--emotion)
2. [MODULE_MAP §10 设施①](../../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)
3. [PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) — pre 阶段
4. [`emotion.rs`](../../../kernel/crates/oclive_kernel_types/src/models/emotion.rs) — DTO 枚举（无未定义变体）
5. [orchestration/turn-thinking](../orchestration/turn-thinking.md) — Auto 路由消费情绪

---

## 4. 开发流程

- [ ] 确认改动在 `UserEmotionAnalyzer` 或对应 backend  
- [ ] pre 阶段输出对齐 `EmotionResult`  
- [ ] 若动复杂情感叙事 → 转设施包，非本槽  
- [ ] domain 单测 · `npm run check:rust`

---

## 5. 验收

- [ ] Prompt 收到的情绪字段来自 emotion 槽  
- [ ] 未新增 `plugin_backends` 非法键  
- [ ] `Emotion` 枚举与 dto 一致

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [complex-emotion](../facilities/complex-emotion.md) | 下游设施，非六键 |
| `prompt` | 注入情绪段落 |
| `event` | 并行 pre，独立 trait |
