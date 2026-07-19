# handoff 历史归档索引

**性质**：历史决策、阶段报告、已完成计划和时点审查的存放区。
**最后更新**：2026-07-19。
**禁止**：将本目录作为当前架构、进度、路径或测试数字的 truth。

## 如何查历史

| 要查什么 | 做法 |
|----------|------|
| 某版本用户可见变化 | 查根目录 [CHANGELOG](../../CHANGELOG.md) |
| 当前模块与边界 | 查 [MODULE_MAP](../MODULE_MAP_AND_HANDOFF.md) |
| 当前关键源码路径 | 查 [BUS_FACTOR_NOTES](../BUS_FACTOR_NOTES.md) |
| 当前技术债与冻结 | 查 [TECHNICAL_DEBT_INVENTORY](../TECHNICAL_DEBT_INVENTORY.md) |
| 历史决策或阶段过程 | 在本目录按文件名搜索，必要时使用 `git log -- <path>` |

## 归档类别

- 早期编号交接包与周报：`00_*`、`01_*`、`02_*` 等。
- 已完成阶段与 closure：`*_CLOSURE_*`、`*_PHASE*`、`*_IMPLEMENTATION_PLAN*`。
- 时点审查与基准：`QUALITY_REVIEW_*`、`P4_*`、`OPUS_*`。
- 已被现行 SSOT替代的设计报告：聊天 mirror collapse、用户身份 / 回复后处理 Phase 2、旧立绘与视觉实施计划。

归档文件保留用于追溯，不再维护正文，也不应链接进新人默认阅读路径。
