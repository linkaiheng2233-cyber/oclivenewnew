# 06 · 内核学习路径（Day 1–5）

> **读者**：准备改 `process_message` / 持久化 / 插件 wiring 的内核贡献者。  
> **读完能做什么**：按时间盒读完主链；完成第一个 domain 单测 PR 草稿。  
> **耗时**：约 3–5 个工作日（维护者带教可 1–2 天）。  
> **下一篇**：[07 常见任务](07_COMMON_TASKS.md)。

---

## 内核 PR 前必读 Top 5

1. [crates/README.md](../crates/README.md) — 依赖图与改 X 去哪  
2. [BUS_FACTOR_NOTES.md §0–2](../handoff/BUS_FACTOR_NOTES.md) — `process_message`、`PluginHost`  
3. [OCLIVE_ARCHITECTURE_OVERVIEW](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) — 单节「模块三层」  
4. [NAMING_CONVENTIONS §4.2](../creator-docs/NAMING_CONVENTIONS.md#42-canonical-import-路径)  
5. [CONTRIBUTING.md §测试要求](../CONTRIBUTING.md#测试要求合并前建议全绿)

---

## Day 1 · 跑通 + 术语（≈ 半天）

| 步骤 | 文档 / 动作 | 验收 |
|------|-------------|------|
| 1 | [02 三十分钟跑通](02_THIRTY_MINUTE_START.md) | `npm run check` 绿 |
| 2 | [03 术语表](03_GLOSSARY.md) + [04 工程约束](04_ENGINEERING_RULES.md) | 能解释 `srid` / `reply` / 六槽 |
| 3 | 浏览 `process_message.rs` 文件头注释 | 能说出 Agent / 共景 / 异地三分支 |

---

## Day 2 · 主链阅读（≈ 1 天）

| 顺序 | 文件 | 关注点 |
|------|------|--------|
| 1 | `process_message.rs` | `run()`：`srid`、健康检查、分支 |
| 2 | `turn_pipeline/mod.rs` | `execute_turn` 四阶段 |
| 3 | `turn_pipeline/pre.rs` | Prompt 输入、复杂情感 |
| 4 | `plugin_host/mod.rs` + `slot_resolver.rs` | 六槽解析 |
| 5 | `prompt_builder/mod.rs` | 段落顺序、guardrails |

**验收**：能手绘 Tauri → `process_message` → `turn_pipeline` → `PluginHost`（见 [01 简架构](01_ARCHITECTURE_SIMPLE.md)）。

---

## Day 3 · 持久化与错误（≈ 1 天）

| 主题 | 入口 |
|------|------|
| Repository trait | `domain/repository.rs` |
| 实现 | `infrastructure/repositories.rs` |
| 迁移 | `crates/oclive_kernel_host/migrations/` |
| 错误码 | `AppError::to_kernel_json()`、[ERROR_CODES](../creator-docs/getting-started/ERROR_CODES.md) |

**验收**：新增字段时知道先写迁移 SQL，再改 trait/impl。

---

## Day 4 · 测试与调试（≈ 半天）

| 类型 | 命令 / 位置 |
|------|-------------|
| 日常门禁 | `npm run check` |
| 发版 | `npm run check:release` |
| domain 单测 | `AppState::new_in_memory_with_llm`（见 [07 常见任务](07_COMMON_TASKS.md)） |
| 日志 | [05 调试](05_DEBUGGING.md) |

---

## Day 5 · 首 PR 草稿（≈ 半天）

建议首个 PR：**纯 domain 单测**或 **文档/注释**（零行为变更），例如：

- 为 `conversation_state_role_id` 补边界测试  
- 或修正一处注释 / 死链  

流程：[CONTRIBUTING.md §PR 流程](../CONTRIBUTING.md#pr-流程) · Dimension 5：`node scripts/dimension5-acceptance.mjs --ci`

**验收**：PR 描述含动机、自检命令、关联 `stage` 或测试名。

---

## 深度链接

- [INVOKE_HOTPATH_MATRIX](../handoff/INVOKE_HOTPATH_MATRIX.md)
- [OOCP_TEST_SUITE](../creator-docs/testing/OOCP_TEST_SUITE.md)
- [ARCHITECTURE_LAYERING](../handoff/ARCHITECTURE_LAYERING.md)
