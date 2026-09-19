# 06 · 内核学习路径（Day 1–5）

**范围澄清（2026-09-11）**：本文带你读现有 Rust **参考 Host** 的主链、持久化与 wiring；历史标题不表示这些都属于小 Kernel。先看 [权责与源码对照](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map)，再按下文学习。固定 stage、DB 提交和角色运行态是当前装配事实，不是新公共契约。

> **最后更新**：2026-09-19
> **读者**：准备改 `process_message` / 持久化 / 插件 wiring 的内核贡献者。  
> **读完能做什么**：按时间盒读完主链；完成第一个 domain 单测 PR 草稿。  
> **耗时**：约 3–5 个工作日（维护者带教可 1–2 天）。  
> **下一篇**：[07 常见任务](07_COMMON_TASKS.md)。

> **插件 / 六槽 / 设施作者**：**可跳过本文**，完成 L0–L3 后直接进入 [modules/](modules/README.md) 选对应开工包。

---

## 内核 PR 前必读 Top 6

**只想了解或接入新的 Base，不必先学完整 ChatPro。** 从 [B1 + B2 阶段总收口](../handoff/README.md#six-slot-stage-closure) 进入，再按 [逐槽实现／验收表](../handoff/README.md#six-slot-b2-adaptation) 查看源码、公开说明与独立调用测试；先确认该实现支持的输入和结果承诺是否适合你的用途。有限参考实现不等于全功能模块，Event 的真实模型分析质量及旧 Host 生产接线仍未验证；下面的 Day 1–5 继续只服务于参考 Host 开发。

**如果你接着看六槽设计**：先读 [Base / Extension 边界](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension) 和 [已确认决策](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-confirmed-decisions)，再看 [统一模板候选](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-contract-candidate)、[逐槽审阅](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-consistency-review)、[怎样组合小 Kernel 公共面](../handoff/MODULE_MAP_AND_HANDOFF.md#small-kernel-composition) 与 [接口表达建议](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-interface-proposal)。基础契约是共同语言，不是模块能力上限；表示稿本身不替代现行 wire 或证明旧 Host 已适配。已实现的独立 Base Rust 绑定、示例和局部验收范围见 [B1 本地收口](../handoff/README.md#six-slot-b1-closure)，不等于稳定 API 发布或生产接线完成。下面的 Day 1–5 是参考 Host 的源码学习路线，不是要求先实现其全部领域系统才能接入六槽。

1. [kernel/crates/README.md](../kernel/crates/README.md) — 依赖图与改 X 去哪  
2. [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md) — **模块注册表 · 逐槽**（2026-06 起 SSOT）  
3. [BUS_FACTOR_NOTES.md §0–2](../handoff/BUS_FACTOR_NOTES.md) — `process_message`、`PluginHost`  
4. [01 简架构](01_ARCHITECTURE_SIMPLE.md) — 记忆三套 · 六槽导读  
5. [NAMING_CONVENTIONS §4.2](../creator-docs/NAMING_CONVENTIONS.md#42-canonical-import-路径)  
6. [CONTRIBUTING.md §测试要求](../CONTRIBUTING.md#测试要求合并前建议全绿)

---

## Day 1 · 跑通 + 术语（≈ 半天）

| 步骤 | 文档 / 动作 | 验收 |
|------|-------------|------|
| 1 | [02 三十分钟跑通](02_THIRTY_MINUTE_START.md) | `npm run check` 绿 |
| 2 | [03 术语表](03_GLOSSARY.md) + [04 工程约束](04_ENGINEERING_RULES.md) | 能解释 `srid` / `reply` / 六槽 / **记忆三套** |
| 3 | 浏览 `process_message.rs` 文件头注释 | 能说出 Agent / 共景 / 异地三分支 |

---

## Day 2 · 主链阅读（≈ 1 天）

| 顺序 | 文件 | 关注点 |
|------|------|--------|
| 1 | `process_message.rs` | preflight、普通用户 Agent 短路、`TurnOrigin` / `TurnInput` |
| 2 | `dispatch.rs` + `turn_pipeline/mod.rs` | remote stub / remote-life / co-present 分派；`execute_turn` 四阶段 |
| 3 | `turn_pipeline/pre.rs` | 人格、身份、情绪、关系与 **memory 检索** |
| 4 | `turn_pipeline/co_present/run_middle.rs` | Turn Thinking、event、Event Ring、回想提案与 Prompt |
| 5 | `turn_pipeline/post.rs` + `post/post_llm.rs` + `persistence.rs` | LLM、状态落地、回复后处理与返回 |
| 6 | [MODULE_MAP §4–§9](../handoff/MODULE_MAP_AND_HANDOFF.md) + `plugin_host/mod.rs` | **逐槽**对照源码 |
| 7 | `prompt_builder/mod.rs` | 段落顺序、guardrails |

**验收**：能手绘 Tauri → `process_message` → `turn_pipeline` → `PluginHost`（见 [01 简架构](01_ARCHITECTURE_SIMPLE.md)）。

主动回合另读 [EVENT_RING](../creator-docs/plugin-and-architecture/EVENT_RING.md)：提案 → 授权 → Permit → `process_proactive_turn`。外部观察不能为了复用旧链而冒充用户消息。

---

## Day 3 · 持久化与错误（≈ 1 天）

| 主题 | 入口 |
|------|------|
| Repository trait | `domain/repository.rs` |
| 实现 | `infrastructure/repositories.rs` |
| 迁移 | `kernel/crates/oclive_kernel_host/migrations/` |
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
