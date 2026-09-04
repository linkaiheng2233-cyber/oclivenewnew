# 设施开工包 · 复杂情感

> **读者**：改 `narrative_hint`、复杂情感叙事链路的工程师。
> **读完能做什么**：区分 **emotion 六槽** 与本设施，在跨轮读取、post 解析和持久化边界内改动。
> **耗时**：约 **40 min**
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §10 设施①](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-09-04
> **下一篇**：[slots/emotion](../slots/emotion.md) · [prompt](../slots/prompt.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§10 设施① 复杂情感](../../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)
- **非六键**：**不**写入 `plugin_backends` 六键；蓝图可在开放 `slot_registry` 中声明 `type: complex_emotion` 设施实例
- **代码锚点**：`complex_emotion.rs` · `turn_pipeline/pre.rs` · `turn_pipeline/co_present/run_middle.rs` · `turn_pipeline/post/post_llm.rs` · `complex_emotion_store.rs`
- **跨轮输入**：`PromptInput.previous_complex_emotion_narrative_hint` 只表示上一轮余韵是否存在；Prompt 不接收 hint 原文
- **本轮输出**：主 LLM 的有效 `[EMO]` 为权威；remote / directory 只在标记缺失或无效时兜底；hint 持久化后由下一轮消费

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| `[EMO]` 解析、插件降级、hint TTL / 持久化；蓝图任务中配置 `type: complex_emotion` 设施实例 | 把它写成 `plugin_backends` 六键或称为稳定第七槽 |
| 显式 `builtin` 开启 hint 读写；`remote` / `directory` 增加插件兜底；省略或 `none` 关闭；发行版仍可用 skip 标志封顶 | 与 emotion 槽合并为一个「情绪模块」文档 |
| 把 emotion 的用户情绪结果作为降级证据之一 | 把用户情绪直接当成角色本轮唯一情绪结论，或在 RFC 未登记前 silent 扩成第七槽 |

---

## 3. 阅读清单

1. [MODULE_MAP §10](../../../handoff/MODULE_MAP_AND_HANDOFF.md#10-第-n-设施子模块编排行内--非六键)
2. [slots/emotion](../slots/emotion.md)
3. [slots/prompt](../slots/prompt.md)
4. [NARRATIVE_HINT_CONTRACT](../../../creator-docs/testing/NARRATIVE_HINT_CONTRACT.md)
5. `complex_emotion.rs`、`post/post_llm.rs` 与 `complex_emotion_store.rs` 源码
6. [AI_CHANGE_BOUNDARIES G1](../../../handoff/AI_CHANGE_BOUNDARIES.md) — 新设施须 RFC

---

## 4. 开发流程

- [ ] 确认需求属于设施而非 emotion 槽分析器
- [ ] 分清改动属于“pre 读取旧 hint”“middle Fast 强度降级”“post 解析/兜底”还是“store 持久化”
- [ ] 保持有效 `[EMO]` 高于插件输出，所有协议标记都从用户可见回复剥离
- [ ] 单测 roundtrip、backend 矩阵与 Unicode 截断（见 `NARRATIVE_HINT_CONTRACT`）
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] `plugin_backends` 无 `complex_emotion` 键
- [ ] 下一轮 Prompt 只收到去内容连续性信号，收不到 hint 原文
- [ ] 本轮 Prompt 没有读取本轮尚未解析的 hint
- [ ] MODULE_MAP §10 仍准确描述行为

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `emotion` | 用户句情绪，仅作为降级证据之一 |
| `llm` | 输出正文和可选权威 `[EMO]` |
| `prompt` | 下一轮只消费去内容连续性信号 |
| Turn Thinking | Auto 路由可能读情绪上下文 |
