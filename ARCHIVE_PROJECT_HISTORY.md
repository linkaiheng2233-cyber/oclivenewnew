# 项目开发史料归档（oclivenewnew）

> **性质**：由仓库内分散的「开发报告、交接包、周总结」等**归纳而成的索引与摘要**，便于新成员了解历史决策与文档演进；**非产品用户文档**。  
> **维护**：日常开发请更新 `CHANGELOG.md` 与代码；本文件仅在重大整理时修订。

---

## 1. 根目录散落文件

| 文件 | 说明 |
|------|------|
| [CHANGELOG.md](CHANGELOG.md) | 版本变更记录（**仍以本文件为发布说明准**） |

**已清理（避免与 `handoff/` 重复）**：根目录曾有的 `DEVELOPMENT_REPORT_HANDOFF.md`、`DEVELOPMENT_PLAN_v3.8.md` 与体量较大的笔记文件 `Rust.txt` 已删除。开发报告与执行计划请以 **`handoff/01_DEVELOPMENT_REPORT.md`**、**`handoff/02_DEVELOPMENT_PLAN_v3.8.md`** 为准；早期周总结若需可查 git 历史。

---

## 2. `handoff/` 交接包（清单）

目录说明见 [handoff/README.md](handoff/README.md)。以下为文件索引表（与 handoff 内 README 对齐并略有补充）：

| 文件 | 摘要 |
|------|------|
| `00_HANDOFF_SUMMARY.md` | 一页摘要、API 事实、阅读顺序 |
| `01_DEVELOPMENT_REPORT.md` | 已落地功能、限制、路径索引 |
| `02_DEVELOPMENT_PLAN_v3.8.md` | 执行计划、禁止项、ADR |
| `03_DEVELOPMENT_STANDARDS.md` | 开发规范与对外文档勘误 |
| `04_4.6_PROJECT_TRUTH_CHECKLIST.md` | 接手前认知对齐清单 |
| `05_4.6_对接报告.md` | 进度快照与代码锚点 |
| `06_WEEK3-004_DEVELOPMENT_PLAN.md` | WEEK3-004 API 任务与验收 |
| `07_DEVELOPMENT_STANDARDS_WEEK3-004.md` | WEEK3-004 代码/DTO/测试标准 |
| `08_开发落实进度报告.md` | 对接落实与门禁 |
| `10_ERROR_CODE_DICTIONARY.md` | 错误码字典 |
| `11_BACKEND_FREEZE_AND_REGRESSION.md` | 后端封板与回归 |
| `12_BACKEND_PERF_RUNBOOK.md` | 压测运行手册 |
| `13_PERF_BASELINE_2026-04-01.md` | 性能基线 |
| `14_POLICY_PLUGIN_PHASE1.md` | 策略插件化 Phase1 |
| `15_POLICY_PLUGIN_GUIDE_AND_ROADMAP.md` | 策略插件架构与路线图 |
| `16_POLICY_RELEASE_CHECKLIST.md` | 策略发布检查 |
| `17_TIME_SCENE_EXPORT_HANDOFF.md` | 虚拟时间、场景、导出交接 |
| `18_DEVELOPMENT_REPORT_USER_ACTIONS.md` | 开发汇报与待办项 |
| `19_RELEASE_CHECKLIST.md` | 发布检查 |
| `20_SESSION_OPTIMIZATION_REPORT.md` | 会话与优化报告 |
| `21_CREATOR_IDENTITY_BINDING.md` | 创作者：`identity_binding` 说明 |
| `DECISIONS_2026-04-02.md` | 决策记录 |
| `DEVELOPMENT_STANDARDS.md` | 标准副本 |
| `WEEKLY_DEV_GUIDE.md` | 周节奏与门禁 |

---

## 3. 文档体系演进（摘要）

- **创作者与插件契约**：现以根目录 **`creator-docs/`** 为权威（分 `getting-started`、`plugin-and-architecture`、`role-pack`、`roadmap`），旧 **`docs/*.md`** 已迁移，见 [docs/README.md](docs/README.md)。  
- **HTTP Remote 侧车**：宿主实现于 `src-tauri/src/infrastructure/remote_plugin/`；协议见 `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md`。  
- **产品愿景**：`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`。

---

## 4. 与创作者文档的边界

| 读者 | 应读 |
|------|------|
| 角色作者 / 侧车开发者 | **[creator-docs/README.md](creator-docs/README.md)** |
| 接手仓库的工程师 | `handoff/00_HANDOFF_SUMMARY.md` → 本归档 §2 按需深入 |
| 版本发布说明 | **[CHANGELOG.md](CHANGELOG.md)** |

---

## 5. 建议（内部）

- 新增长文开发笔记：优先放入 `handoff/` 并在 `handoff/README.md` 登记；避免在根目录重复堆积。  
- 对外发布：以 **CHANGELOG** + **creator-docs** 为准，不依赖本归档正文。
