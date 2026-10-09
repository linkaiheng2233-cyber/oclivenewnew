# oclive 架构决策记录（ADR 摘要）

本文档将关键架构取舍从对话与 handoff 中**沉淀为可检索文本**，按主题排列（非严格时间线）。更完整的分层纪律见 [`handoff/ARCHITECTURE_LAYERING.md`](../../handoff/ARCHITECTURE_LAYERING.md)。

**读者**：新贡献者、插件作者、fork 宿主集成方。

**English summary:** [`creator-docs/architecture-en/DESIGN_DECISIONS.md`](../architecture-en/DESIGN_DECISIONS.md) · full EN mirror in [`creator-docs-en/architecture/DESIGN_DECISIONS.md`](../../creator-docs-en/architecture/DESIGN_DECISIONS.md).

---

## 1. 蓝图不再驱动主编排顺序

| 决策 | 为什么这样做 |
|------|----------------|
| **`pipeline.ocblueprint` 不解释普通 Stable 执行 DSL** | 避免「文件里写的流程」与 `process_message` / `co_present` **实际执行顺序**不一致；编排顺序由 **Rust 代码**审计。蓝图提供 `slot_registry`、只读 `groups`、Stable v4 `runtime_config` / `extensions` 等配置；仅冻结 v3 双核 Beta 的 `pipeline.experimental` 是受限实验 DAG。 |
| **入口** | [`kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) |

---

## 2. 防腐层：`domain/ports` 零 trait 定义

| 决策 | 为什么这样做 |
|------|----------------|
| **trait 集中在 `oclive_kernel_contracts`** | 核心抽象与 Tauri 解耦；桌面、无头、嵌入式宿主均可实现同一套端口。 |
| **`distros/desktop-tauri/domain/ports/` 仅 re-export** | 编排依赖 `dyn PluginHostPort` / `LlmClient` 等，不绑定 `PluginHost` 具体类型。 |

---

## 3. `module_relations` 只读派生

| 决策 | 为什么这样做 |
|------|----------------|
| **禁止写入 `pipeline.ocblueprint` 的 `module_relations`** | 手动维护映射易与 `slot_registry` 漂移；**从 registry 推导边**是唯一可靠来源（`oclive_validation` + 前端 `buildBlueprintEdges`）。 |

---

## 4. 蓝图 `groups` 分组

| 决策 | 为什么这样做 |
|------|----------------|
| **分组仅影响创作者 UI** | 模仿 v1「六模块」分类，把同类型多实例收拢到逻辑边框；**不改变** `SlotResolver` 解析顺序或 `SlotRunner` 合并语义。 |

---

## 5. 多实例合并策略（按槽位语义）

| 槽位类型 | 策略 | 为什么 |
|----------|------|--------|
| memory | 串行合并 + **按 id 去重** | 用户需要多路召回的**并集**，同一记忆不应重复注入 Prompt |
| llm | 非流式：`ensemble` 串行 last-wins（默认）/ `fastest` 并发首个成功 / `fallback` 按序首个成功；流式当前串行 last-wins | 允许按资源与可靠性取舍；流式避免并发 token 混写 |
| emotion / event / prompt / complex_emotion | 串行 **last-wins** | 状态类或「最终文本」语义，后次覆盖前次 |
| agent（多目录插件） | **尚未合并执行** | 当前只收集 `merged_agent_directory_plugin_ids` 供诊断，`SlotResolver::wrap_agent_if_merged` 返回原 provider；工具并集见 `K-AGENT-MERGE-01` |

实现与注释：[`kernel/crates/oclive_kernel_host/src/domain/slot_runner.rs`](../../kernel/crates/oclive_kernel_host/src/domain/slot_runner.rs)。

---

## 6. C1 薄包装（会话 API 过渡期）

| 决策 | 为什么这样做 |
|------|----------------|
| **保留旧 Tauri 命令签名，内部委托 `set_session_slot_override`** | 给下游（启动器、旧脚本）**一个版本**的迁移窗口；新代码应使用 slot_registry 覆盖路径。 |

---

## 7. 蓝图加载数据流（配置 → 执行）

```text
distros/chat-pro/roles/{id}/pipeline.ocblueprint
  → schema_version 精确分派 v2 / v3 / v4（解析 + 校验）
  → Role { slot_registry, plugin_backends, slot_groups, runtime_config, extensions }
  → PluginHost::resolve → SlotResolver::resolve
  → process_message → SlotRunner
```

详见：[`kernel/crates/oclive_kernel_host/src/infrastructure/storage/mod.rs`](../../kernel/crates/oclive_kernel_host/src/infrastructure/storage/mod.rs) 模块注释。

---

<a id="full-resilience-deferred"></a>

## 8. 保持 Minimal 韧性，Full 暂缓（2026-10-08）

| 决定 | 依据与冻结范围 |
|------|----------------|
| **保持现有 Minimal；Full 为 Deferred** | 维护者确认现有有限收束继续有效；没有明确故障目标时不因“Full”名称发明通用层。暂缓不是完成，也不是永久取消；本次不增加实现。当前债状态见[唯一台账](../../handoff/TECHNICAL_DEBT_INVENTORY.md)。 |
| **保留小 Kernel / Host 的权责边界** | 沿用[既有权责](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)：Kernel 守住契约、权限合法性、必要因果与终态等硬边界；发行版故障处理和运行环境适配由 Host 在这些边界内承担。本次不新增公共 API 或架构权力。 |
| **不自动引入执行策略** | 不擅自增加自动重试、熔断、模块接管或降级；现有 Agent、工具调用、六槽交互和已验 Remote 行为保持。既有策略仍依原实现与授权运行，暂缓不删除它们。 |

**重新评估条件**：只有出现具体且可复现的故障案例、现有 Minimal 无法满足明确的可靠性需求，并且目标、影响范围及语义风险能够界定，才重新评估 Full。重新评估不等于自动解冻实施；涉及新执行语义、契约或架构权力分配时仍须维护者确认。没有满足这些前提的候选继续登记，不能为了偿债数量自行打开 Full。

原 Minimal 计划与历史效力见[冻结入口](../../handoff/debt-marathon/long-plans/K-RESILIENCE-01.md#full-deferred-freeze)；执行与接手事件见[DCL-65](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261008-65--保持-minimal并将-full-韧性暂缓)。

---

<a id="emotion-memory-extension-deferred"></a>

## 9. 情绪驱动长期记忆：暂缓实施，保留可选扩展（2026-10-09）

维护者选择 **K-EMO-06 Deferred**：当前情绪与记忆的基础能力原则上足以支撑现阶段目标，核心项目优先处理这些基础能力的实际缺陷，不继续开发高级情绪驱动长期记忆策略。这里是本阶段的范围决定，不是复杂情绪质量、所有场景或长期效果已经验证；Deferred 不等于已完成、已实现或永久取消。当前状态仍见[唯一台账](../../handoff/TECHNICAL_DEBT_INVENTORY.md)。

**保留的方向**：情绪参与长期记忆筛选、重要性或权重调整，可以由第三方模块或发行版作为可选扩展探索。例如，高情绪内容是否优先记住、临时抱怨是否进入长期库，属于扩展的产品策略，不成为所有 Memory 或 Emotion 实现的最低义务。扩展仍需遵守现有公共合同和实际授权；记录方向不证明任意高级模块已经可接入，也不授权自动读写 Host 长期库。

**边界**：不纳入小 Kernel 核心职责，不提前增加六槽 Base 的强制方法、字段或默认策略，不把扩展输出赋予领域提交权。沿用[Kernel / Host 分责](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)与[六槽 Base / Extension 边界](../../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension)；本次不改变现有情绪、记忆、Agent、工具调用或六槽交互实现。

**重新评估条件**：出现具体可复现的接入案例，证明现有公开契约阻碍一个明确的高级模块或发行版需求时，记录受阻接口、预期能力、影响范围和最小反例，再评估是否调整公共契约。不能仅因有研究价值或缺少高级算法而自动解冻；新增执行语义、权限或公共强制义务仍须维护者确认。方向、范围和反例已足以支持取舍时停止调查，不穷尽所有未来策略。

---

## 相关文档

- [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)
- [历史 trait 审计：KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md](../../handoff/archive/KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md)

[English](../architecture-en/DESIGN_DECISIONS.md)
