# 编排行开工包 · Model Tier（摘要）

> **读者**：改 `ModelTier` Small/Large 启发式或 Deep Tier0 / PersonaSource 的工程师。  
> **读完能做什么**：在 `model_tier.rs` 边界内改动，理解其与 Turn Thinking 的编排行关系。  
> **耗时**：约 **30 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §12](../../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)
> **最后更新**：2026-07-14
> **下一篇**：[turn-thinking](turn-thinking.md) · [slots/prompt](../slots/prompt.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§12 `ModelTier` · `PersonaSource`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)
- **代码**：`model_tier.rs` · `co_present`  
- **Deep Tier0**：角色 `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt`  
- **非**独立 `plugin_backends` 键

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| Ollama 模型名启发式 Small/Large | 把 ModelTier 登记为六槽 |
| Deep capsule 与 FullCore 切换 | 运行时 LLM 压缩 prompt（见 prompt 槽） |
| `DEEP_PROMPT_DISTILLATION` 设计对齐 | 在 UI 层选模型绕过 llm 槽 |

---

## 3. 阅读清单

1. [MODULE_MAP §12](../../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)
2. [DEEP_PROMPT_DISTILLATION](../../../handoff/DEEP_PROMPT_DISTILLATION.md)
3. [TTFT_BENCHMARK](../../../handoff/TTFT_BENCHMARK.md)
4. [turn-thinking](turn-thinking.md)  
5. [slots/llm](../slots/llm.md)

---

## 4. 开发流程

- [ ] 确认改动在 `model_tier.rs` 或 `co_present` PersonaSource 分支  
- [ ] Deep 路径测 `deep_capsule.txt` 存在性  
- [ ] bench 回归若动 TTFT 关键路径  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] Small+Deep 接线行为与 DEEP_PROMPT_DISTILLATION 一致  
- [ ] llm 槽仍负责实际 generate  
- [ ] 未新增第七槽配置键

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [turn-thinking](turn-thinking.md) | Deep 档触发 |
| [prompt](../slots/prompt.md) | Tier0 来源选择 |
| [llm](../slots/llm.md) | 模型调用 |
