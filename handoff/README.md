# handoff/ — 活跃工程交接文档

本目录仅保留**当前仍被 AGENTS.md、CI 或贡献流程直接引用**的短文；属 **AI 接手包**（维护者深读）。**新人请先** [human-docs/06_KERNEL_LEARNING_PATH.md](../human-docs/06_KERNEL_LEARNING_PATH.md) 与 [human-docs/08_REFERENCE_MAP.md](../human-docs/08_REFERENCE_MAP.md)，再按需打开本目录。

历史批次报告、closure summary、旧周报与编号开发计划已迁入 [`archive/`](archive/)。

## 活跃文件（根目录 · 跨发行版）

| 文件 | 用途 |
|------|------|
| [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) | 破坏性变更流程 |
| [PRODUCT_LINE_TASK_BUCKETS.md](PRODUCT_LINE_TASK_BUCKETS.md) | 产品线任务分桶 |
| [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) | 技术债清单 |
| [RECURRING_OPTIMIZATION_PLAYBOOK.md](RECURRING_OPTIMIZATION_PLAYBOOK.md) | 巡检手册（§8 日志） |
| [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md) | 新人 issue 策展 |
| [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) | 双核实验运行时交接 |
| [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) | 角色包 vs 蓝图边界 |
| [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) | 分层架构说明 |
| [BUS_FACTOR_NOTES.md](BUS_FACTOR_NOTES.md) | 关键路径 bus factor |
| [INVOKE_HOTPATH_MATRIX.md](INVOKE_HOTPATH_MATRIX.md) | Tauri invoke 热路径矩阵 |
| [04_4.6_PROJECT_TRUTH_CHECKLIST.md](04_4.6_PROJECT_TRUTH_CHECKLIST.md) | 项目认知清单 |
| [PERF_PHASES.md](PERF_PHASES.md) | 性能/包体与协议验证快照 |
| [CHAT_STORAGE_ARCHITECTURE.md](CHAT_STORAGE_ARCHITECTURE.md) | 聊天混合存储架构 |
| [GITHUB_PLUGIN_INDEX_LINE.md](GITHUB_PLUGIN_INDEX_LINE.md) | GitHub 插件索引线 |
| [BLUEPRINT_FOLDER_LAYOUT.md](BLUEPRINT_FOLDER_LAYOUT.md) | 蓝图目录布局 |
| [COMMENT_ENGLISH_MIGRATION_PLAN.md](COMMENT_ENGLISH_MIGRATION_PLAN.md) | 注释英文化计划 |
| [OCLIVE_POSITIONING_DIFFERENTIATION.md](OCLIVE_POSITIONING_DIFFERENTIATION.md) | 定位与差异化 |
| [THREE_DISTRO_KERNEL_CLOSURE.md](THREE_DISTRO_KERNEL_CLOSURE.md) | 三发行版内核结项 |
| [KERNEL_SCHEDULER_RESCOPE.md](KERNEL_SCHEDULER_RESCOPE.md) | 内核调度范围重划 |

**Chat Pro（`desktop`）** = 主应用默认发行版；契约与工程文档在 `creator-docs/` 与本目录根级文件，**不单建 `handoff/desktop/`**。

## 发行版附带文档（工作文档 · 按 distro 归位）

契约 SSOT 仍在 [`creator-docs/`](../creator-docs/)；下表为各发行版**协调与工作文档**入口。

| 目录 | 发行版 | 入口 |
|------|--------|------|
| [theater/](theater/) | **AI 剧场** | [README](theater/README.md) · [DEVELOPMENT_ROADMAP](theater/DEVELOPMENT_ROADMAP.md) |
| [vscode/](vscode/) | **VS Code Flash** | [README](vscode/README.md) |
| [launcher/](launcher/) | 启动器（姊妹仓） | [README](launcher/README.md) |
| [pack-editor/](pack-editor/) | 角色包编写器（姊妹仓） | [README](pack-editor/README.md) |
| [studio/](studio/) | 工作室（合并叙事） | [README](studio/README.md) |

## 归档规则

- **迁入 `archive/`**：`*_CLOSURE_SUMMARY*`、旧 `0x_` / `1x_` 开发报告与计划、`WEEKLY_DEV_GUIDE`、已合并进 `creator-docs/` 的阶段性总结。
- **留在根目录**：上表「跨发行版」所列；新增 handoff 前请确认是否应进 `creator-docs/`、`handoff/<distro>/` 或 `archive/`。
- **勿删** `archive/` 内文件（史料与审计对照）；链接失效时从 `archive/` 恢复或改链到 `creator-docs/`。

性能阶段总表见本目录 [PERF_PHASES.md](PERF_PHASES.md) 与 [`creator-docs/development/`](../creator-docs/development/)。
