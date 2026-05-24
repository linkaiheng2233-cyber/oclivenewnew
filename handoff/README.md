# handoff/ — 活跃工程交接文档

本目录仅保留**当前仍被 AGENTS.md、CI 或贡献流程直接引用**的短文。历史批次报告、closure summary、旧周报与编号开发计划已迁入 [`archive/`](archive/)。

## 活跃文件（≤15）

| 文件 | 用途 |
|------|------|
| [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) | 破坏性变更流程 |
| [PRODUCT_LINE_TASK_BUCKETS.md](PRODUCT_LINE_TASK_BUCKETS.md) | 产品线任务分桶 |
| [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) | 技术债清单 |
| [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) | 双核实验运行时交接 |
| [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) | 角色包 vs 蓝图边界 |
| [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) | 分层架构说明 |
| [BUS_FACTOR_NOTES.md](BUS_FACTOR_NOTES.md) | 关键路径 bus factor |
| [INVOKE_HOTPATH_MATRIX.md](INVOKE_HOTPATH_MATRIX.md) | Tauri invoke 热路径矩阵 |
| [04_4.6_PROJECT_TRUTH_CHECKLIST.md](04_4.6_PROJECT_TRUTH_CHECKLIST.md) | 项目认知清单 |

## 归档规则

- **迁入 `archive/`**：`*_CLOSURE_SUMMARY*`、旧 `0x_` / `1x_` 开发报告与计划、`WEEKLY_DEV_GUIDE`、已合并进 `creator-docs/` 的阶段性总结。
- **留在根目录**：上表所列；新增 handoff 前请确认是否应进 `creator-docs/` 或 `archive/`。
- **勿删** `archive/` 内文件（史料与审计对照）；链接失效时从 `archive/` 恢复或改链到 `creator-docs/`。

性能阶段总表见 [`creator-docs/development/`](../creator-docs/development/)（原 `PERF_PHASES` 类文档已归文档站维护）。
