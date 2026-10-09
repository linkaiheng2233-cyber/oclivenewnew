# 六槽开工包 · `prompt`

> **读者**：编写基础 Prompt 实现，或维护参考 Host 的段落公式、overlay 与 prompt 后端的工程师。
> **读完能做什么**：选择基础组装或丰富 Prompt 路径，找到实现、消费者和 Host 接线。
> **耗时**：基础入口为短导航；参考 Host 流程约 **50 min**
> **SSOT 范围**：人类路由与 checklist；公共接入点见 [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，参考 Host 定义见 [§7](../../../handoff/MODULE_MAP_AND_HANDOFF.md#7-第-4-模块--prompt)
> **最后更新**：2026-10-09
> **下一篇**：[07 §2](../../07_COMMON_TASKS.md#2-改-prompt-段落) · [llm](llm.md)

---

## 0. 先选接入路径

基础组装实现从下方开始；维护 `PromptBuilder`、overlay 或蓝图后端则读 **§1–§6**。后者的检查属于参考 Host 丰富路径，不自动成为所有 Base 实现的前置要求。

### PromptBase：找到组装与消费入口

1. 在[公共契约](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs)找到 `PromptBase::assemble`，再读现有 [`LiteralMaterialAssembler`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_prompt.rs) 的实现约定：材料按原顺序原样拼接，非空额外要求返回 `Unsupported`。这是该实现的范围，不是所有 Prompt 实现必须采用的算法。
2. 看[`MinimalRolePromptConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs)如何先准备最小角色人设和本次材料，再委托调用方选定的 Prompt。这里已准备人设，选定实现不应再次重复注入；材料标题也不构成下游防注入保证。
3. 参考 Host 的显式入口见 [`process_minimal_message_with_prompt`](../../../kernel/crates/oclive_kernel_host/src/role_kernel.rs)。独立 Host 装配可读现有 [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs)；这是已有案例导航，不承诺配置式插件注册。

**基础路径 checklist**：

- [ ] 写清实现接受的材料与要求；经消费者核对原文、顺序、空结果和完整失败。
- [ ] 调用方选择能力、材料和执行器；组装不会自行取得资源权限，也不安排固定六槽回合。

请求、失败和绑定的定义只查[接入点清单](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)。本导航不代表真实模型输出质量已验收。

---

## 1. 参考 Host：你插在哪

- **MODULE_MAP**：[§7 第 4 模块 · `prompt`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#7-第-4-模块--prompt)
- **当前配置 / 运行时折叠**：蓝图 `slot_registry.type: prompt` → `PluginBackends.prompt`；legacy v1 才是 `settings.json.plugin_backends.prompt`
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
- [ ] 设施信息经 `PromptInput` 进入 Prompt 时不冒充第七槽；复杂情感只允许上一轮 hint 触发去内容连续性信号，不注入原文或本轮 hint

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `memory` / `emotion` | pre 注入 `PromptInput` |
| [complex-emotion](../facilities/complex-emotion.md) | `previous_complex_emotion_narrative_hint` 仅作上一轮余韵存在信号 |
| `llm` | 下游消费完整 prompt 字符串 |
| [model-tier](../orchestration/model-tier.md) | Deep Tier0 / PersonaSource |
