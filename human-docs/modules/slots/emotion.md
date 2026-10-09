# 六槽开工包 · `emotion`

> **读者**：编写基础情绪线索实现，或维护参考 Host 的用户句分析与 emotion 后端的工程师。
> **读完能做什么**：区分 Base 线索、丰富分析结果和复杂情感设施，找到各自接入路径。
> **耗时**：基础入口为短导航；参考 Host 流程约 **40 min**
> **SSOT 范围**：人类路由与 checklist；公共接入点见 [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，参考 Host 定义见 [§5](../../../handoff/MODULE_MAP_AND_HANDOFF.md#5-第-2-模块--emotion)
> **最后更新**：2026-10-09
> **下一篇**：[facilities/complex-emotion](../facilities/complex-emotion.md) · [event](event.md)

---

## 0. 先选接入路径

基础线索实现从下方开始；维护 `UserEmotionAnalyzer`、`EmotionResult` 或参考 Host 后端则读 **§1–§6**。丰富结果和设施规则不自动成为所有 Base 实现的前置要求。

### EmotionBase：消费线索，不替 Host 写状态

1. 在[公共契约](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs)找到 `EmotionBase::analyze`，再读现有 [`KeywordEmotionBase`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_emotion.rs) 的词表线索约定与限制；非空额外 context 在该实现中返回 `Unsupported`。
2. 看[`MinimalRoleBaseConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs)如何将本次材料直接委托给选定实现。参考 Host 的 builtin 分析器已有 Base 视图；[`最小会话入口`](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)把线索作为参考材料消费，不把报告转成七维分数或直接应用状态。
3. Base-only 独立实现形状见 [`base_only_fixture.rs`](../../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs)。作者可提供自己的分析约定，不必复刻内置词表；该 fixture 不证明真实语义质量。

**基础路径 checklist**：

- [ ] 区分正常无报告与失败；保留实现的完整线索和限制，不把词表命中当作主体、意图或真实情绪的最终判定。
- [ ] 调用方决定如何使用结果；线索不会自行写入情绪状态、长期记忆或取得新权限。

请求、失败和绑定查[接入点清单](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。高级情绪驱动记忆方向仍 Deferred，不在此增加 Base 强制接口。

---

## 1. 参考 Host：你插在哪

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
