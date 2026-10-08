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

## 相关文档

- [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)
- [历史 trait 审计：KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md](../../handoff/archive/KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md)

[English](../architecture-en/DESIGN_DECISIONS.md)
