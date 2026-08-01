# handoff · 维护者与 AI 工程入口

**SSOT 范围**：本文只登记活跃 handoff 的职责、分层和归档规则。
**最后更新**：2026-08-01。
**新人开发者**从 [human-docs](../human-docs/README.md) 开始；**创作者**从 [创作者黄金路径](../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md) 开始。

## 文档分层

| 层 | 读者 | 只负责 | 入口 |
|----|------|--------|------|
| 根 `README` | 所有人 | 项目定位与身份分流 | [README](../README.md) |
| `human-docs/` | 主仓开发者 | 顺序学习、调试、模块开工 | [学习阶梯](../human-docs/README.md) |
| `creator-docs/` | 用户、创作者、插件作者、集成方 | 现行使用说明与公开契约 | [文档索引](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| `handoff/` | 维护者、AI Agent | 工程边界、关键路径、债务、巡检 | 本页 |
| `handoff/archive/` | 查历史的人 | 阶段记录与已完成报告；**非 truth** | [归档索引](archive/ARCHIVE_PROJECT_HISTORY.md) |
| `*-en/` | 英文读者 | 对应中文 SSOT 的镜像 | [creator-docs-en](../creator-docs-en/README.md) · [human-docs-en](../human-docs-en/README.md) |

## 最短入口

| 任务 | 先读 | 再读 |
|------|------|------|
| AI 改代码或文档 | [AI_CHANGE_BOUNDARIES](AI_CHANGE_BOUNDARIES.md) | [AI_READING_INDEX](AI_READING_INDEX.md) |
| 接手主编排 / DB | [BUS_FACTOR_NOTES](BUS_FACTOR_NOTES.md) | [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md) |
| 改模块或槽位 | [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md) | [SLOT_BACKEND_REALITY_MATRIX](SLOT_BACKEND_REALITY_MATRIX.md) |
| 做全仓审查 | [AI_VERIFICATION_PROTOCOL](AI_VERIFICATION_PROTOCOL.md) | [RECURRING_OPTIMIZATION_PLAYBOOK](RECURRING_OPTIMIZATION_PLAYBOOK.md) |
| 看当前债务 / 冻结 | [TECHNICAL_DEBT_INVENTORY](TECHNICAL_DEBT_INVENTORY.md) | — |
| 改角色包边界 | [ROLE_PACK_BOUNDARY](ROLE_PACK_BOUNDARY.md) | [角色包规范](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |

## 活跃 SSOT

### 架构与边界

| 文件 | 唯一职责 |
|------|----------|
| [MODULE_MAP_AND_HANDOFF.md](MODULE_MAP_AND_HANDOFF.md) | 模块注册表与六槽 / 设施 / 独立通道关系 |
| [SLOT_BACKEND_REALITY_MATRIX.md](SLOT_BACKEND_REALITY_MATRIX.md) | 六槽 × backend 实现真值 |
| [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) | 角色包、蓝图、发行版和会话分责 |
| [CHAT_STORAGE_ARCHITECTURE.md](CHAT_STORAGE_ARCHITECTURE.md) | 聊天日志、短期与长期记忆存储 |
| [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) | domain / infrastructure 依赖方向 |
| [BLUEPRINT_FOLDER_LAYOUT.md](BLUEPRINT_FOLDER_LAYOUT.md) | 蓝图目录和 includes 布局 |
| [KERNEL_SCHEDULER_RESCOPE.md](KERNEL_SCHEDULER_RESCOPE.md) | 内核调度器当前边界 |
| [THREE_DISTRO_KERNEL_CLOSURE.md](THREE_DISTRO_KERNEL_CLOSURE.md) | 三发行版内核结项约束；新能力以 HostProfile SSOT 为准 |

### 关键路径与验证

| 文件 | 唯一职责 |
|------|----------|
| [BUS_FACTOR_NOTES.md](BUS_FACTOR_NOTES.md) | 主编排、DB、错误码与源码锚点 |
| [INVOKE_HOTPATH_MATRIX.md](INVOKE_HOTPATH_MATRIX.md) | Tauri invoke 热路径矩阵 |
| [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) | 破坏性变更流程 |
| [AI_CHANGE_BOUNDARIES.md](AI_CHANGE_BOUNDARIES.md) | AI 改动边界 G1–G17（含关联能力闭环） |
| [AI_READING_INDEX.md](AI_READING_INDEX.md) | AI 按任务深读导航，不承载事实 |
| [AI_VERIFICATION_PROTOCOL.md](AI_VERIFICATION_PROTOCOL.md) | 审查与带数字汇报的核实规则 |
| [RECURRING_OPTIMIZATION_PLAYBOOK.md](RECURRING_OPTIMIZATION_PLAYBOOK.md) | 多轮巡检流程 |

### 状态、性能与专项执行

| 文件 | 唯一职责 |
|------|----------|
| [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) | 活跃债、冻结项与下一动作 |
| [PRODUCT_LINE_TASK_BUCKETS.md](PRODUCT_LINE_TASK_BUCKETS.md) | 产品线执行分桶 |
| [PERF_PHASES.md](PERF_PHASES.md) | 性能阶段与复现入口 |
| [TTFT_BENCHMARK.md](TTFT_BENCHMARK.md) | TTFT 基准 |
| [DEEP_PROMPT_DISTILLATION.md](DEEP_PROMPT_DISTILLATION.md) | Deep 路径与 Prompt 蒸馏专项 |
| [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) | 双核实验交接 |
| [GITHUB_PLUGIN_INDEX_LINE.md](GITHUB_PLUGIN_INDEX_LINE.md) | GitHub 插件索引线 |
| [COMMENT_ENGLISH_MIGRATION_PLAN.md](COMMENT_ENGLISH_MIGRATION_PLAN.md) | 注释英文化计划 |
| [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md) | 新人 issue 策展 |
| [OCLIVE_POSITIONING_DIFFERENTIATION.md](OCLIVE_POSITIONING_DIFFERENTIATION.md) | 产品定位与差异化 |

## 发行版工作区

| 目录 | 入口 |
|------|------|
| `theater/` | [AI Theater](theater/README.md) |
| `vscode/` | [VS Code Flash](vscode/README.md) |
| `pack-editor/` | [角色包编写器](pack-editor/README.md) |
| `launcher/` | [退役启动器](launcher/README.md) |
| `studio/` | [工作室叙事](studio/README.md) |
| `debt-marathon/` | [债务马拉松记录](debt-marathon/README.md) |

## 归档与新增规则

- 已完成的 phase、closure、时点审查和旧实施计划进入 `handoff/archive/`，不得作为现行行为依据。
- 现行行为以源码、上表 SSOT、`creator-docs/` 契约和 `TECHNICAL_DEBT_INVENTORY` 为准。
- 不新建第二份项目总览、模块表、状态页或发版清单；扩展现有 SSOT并从索引链接。
- 根级新增 `handoff/*.md` 必须满足 [AI_CHANGE_BOUNDARIES G11–G16](AI_CHANGE_BOUNDARIES.md) 并登记到本页。

门禁：`node scripts/check-doc-registry.mjs` · `node scripts/check-markdown-links.mjs` · `node scripts/check-stale-paths.mjs`。
