# 六槽开工包 · `emotion`

> **读者**：改用户句情绪分析或 emotion 后端的工程师。
> **读完能做什么**：区分 **emotion 六槽** 与 **复杂情感设施**，在边界内改分析器。
> **耗时**：约 **40 min**
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §5](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-09-05
> **下一篇**：[facilities/complex-emotion](../facilities/complex-emotion.md) · [event](event.md)

---

## 1. 你插在哪

- **T0 / T1+ 分层（Draft RFC）**：[RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md](../../../creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md) — 最小闭环 = `analyze`；模拟与展示指标为扩展
- **MODULE_MAP**：[§5 第 2 模块 · `emotion`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#5-第-2-模块--emotion)
- **当前配置 / 运行时折叠**：蓝图 `slot_registry.type: emotion` → `PluginBackends.emotion`；legacy v1 才是 `settings.json.plugin_backends.emotion`
- **Trait**：`UserEmotionAnalyzer`
- **主链 hook**：`turn_pipeline/pre.rs` → `EmotionResult` → Prompt · Turn Thinking Auto

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 分析器、`remote` / `directory` 协议 | 把 `slot_registry` 中的 `complex_emotion` 设施实例冒充稳定六槽或写进 `plugin_backends` 六键 |
| `builtin` · `remote` · `directory` · `none` | 与 [complex-emotion 设施](../facilities/complex-emotion.md) 混为一谈 |

**区分**：本槽分析 **用户句** 情绪，只能作为 Prompt 线索或复杂情感降级证据之一；复杂情感设施的本轮角色回复情绪以主 LLM 有效 `[EMO]` 为权威，remote / directory 仅在标记缺失或无效时兜底，并把可用 hint 留给下一轮。

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
| `event` | 在 middle 先规则初估，再由 Turn Thinking 决定是否调用独立 `EventEstimator` 槽 |
