# oclive 架构决策记录（ADR 摘要）

本文档将关键架构取舍从对话与 handoff 中**沉淀为可检索文本**，按主题排列（非严格时间线）。更完整的分层纪律见 [`handoff/ARCHITECTURE_LAYERING.md`](../../handoff/ARCHITECTURE_LAYERING.md)。

**读者**：新贡献者、插件作者、fork 宿主集成方。

---

## 1. 蓝图不再驱动主编排顺序

| 决策 | 为什么这样做 |
|------|----------------|
| **`pipeline.ocblueprint` 不解释执行 DSL** | 避免「文件里写的流程」与 `process_message` / `co_present` **实际执行顺序**不一致；编排顺序由 **Rust 代码**审计，蓝图只提供配置（`slot_registry`、`groups`）。 |
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
| llm | 串行 **last-wins** | 只需**一条**最终回复；共享同一 prompt，避免并发打满资源 |
| emotion / event / prompt / complex_emotion | 串行 **last-wins** | 状态类或「最终文本」语义，后次覆盖前次 |
| agent（多目录插件） | **PluginHost** 合并目录 ID | 多工具无强顺序依赖时合并工具集；执行逻辑见 `plugin_host` / `SlotResolver::wrap_agent_if_merged` |

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
  → load_blueprint_v2_for_role_dir（解析 + 校验）
  → Role { slot_registry, plugin_backends, slot_groups }
  → PluginHost::resolve → SlotResolver::resolve
  → process_message → SlotRunner
```

详见：[`kernel/crates/oclive_kernel_host/src/infrastructure/storage.rs`](../../kernel/crates/oclive_kernel_host/src/infrastructure/storage.rs) 模块注释。

---

## 相关文档

- [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)
- [KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md](../../handoff/KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md)

[English](../architecture-en/DESIGN_DECISIONS.md)
