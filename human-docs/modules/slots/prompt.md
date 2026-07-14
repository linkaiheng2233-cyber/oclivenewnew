# 六槽开工包 · `prompt`

> **读者**：改 Prompt 段落公式、overlay 或 prompt 后端的工程师。  
> **读完能做什么**：在 `PromptBuilder::build_prompt` 边界内改组装逻辑，守 guardrails 纪律。  
> **耗时**：约 **50 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §7](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-07-14
> **下一篇**：[07 §2](../../07_COMMON_TASKS.md#2-改-prompt-段落) · [llm](llm.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§7 第 4 模块 · `prompt`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#7-第-4-模块--prompt)
- **`plugin_backends` 键**：`prompt`  
- **Trait**：`PromptAssembler` → 内置 **`PromptBuilder::build_prompt`**  
- **主链 hook**：`co_present` `BuildPrompt` · `PromptInput`  
- **代码 SSOT**：`kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/`

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| `sections.rs` 段落公式 · concise overlay | 运行时 LLM 压缩 prompt |
| `reply_quality_anchor`（包级 **可替** 默认锚点） | 用 capsule **替换** `KERNEL_DIALOGUE_GUARDRAILS` |
| `builtin` · `remote` · `directory` | 共景路径 `none` backend |
| `prompts/deep_capsule.txt`（Wave D · 已接线） | `build_prompt` 返回 `Result`（须返回 `String`） |

---

## 3. 阅读清单

1. [MODULE_MAP §7](../../../handoff/MODULE_MAP_AND_HANDOFF.md#7-第-4-模块--prompt)
2. [04 工程约束 §5–§6](../../04_ENGINEERING_RULES.md) — PromptBuilder · guardrails  
3. [07 §2 改 Prompt 段落](../../07_COMMON_TASKS.md#2-改-prompt-段落)  
4. [DEEP_PROMPT_DISTILLATION](../../../handoff/DEEP_PROMPT_DISTILLATION.md) — Deep capsule
5. [ROLE_PACK_BOUNDARY](../../../handoff/ROLE_PACK_BOUNDARY.md) — Tier0 真源

---

## 4. 开发流程

- [ ] 改段落 → `sections.rs`；改顺序 → `mod.rs`  
- [ ] 新 `PromptInput` 字段 → `pre.rs` 注入 + dto 若需暴露  
- [ ] 角色包只改 `core_personality.txt` / 锚点 → [role-pack-content](../packs/role-pack-content.md)  
- [ ] 单测：`narrative_hint_prompt_roundtrip` 等  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] `build_prompt(&PromptInput)` 返回 `String`  
- [ ] 每轮仍追加 `KERNEL_DIALOGUE_GUARDRAILS`  
- [ ] Tier0 来自 `core_personality.txt`  
- [ ] 设施段落（复杂情感等）经 `PromptInput` 注入，非第七槽

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `memory` / `emotion` | pre 注入 `PromptInput` |
| [complex-emotion](../facilities/complex-emotion.md) | `previous_complex_emotion_narrative_hint` |
| `llm` | 下游消费完整 prompt 字符串 |
| [model-tier](../orchestration/model-tier.md) | Deep Tier0 / PersonaSource |
