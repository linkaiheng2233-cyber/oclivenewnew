# 项目现状（事实快照）

**用途**：给协作者与发布流程一个**短、可核对**的当前状态摘要（版本、交付面、变更日志入口）。文档查找统一从 [DOCUMENTATION_INDEX](DOCUMENTATION_INDEX.md) 开始。

[English](../../creator-docs-en/getting-started/PROJECT_CURRENT_STATUS.md)

**快照日期**：2026-08-20（随大里程碑或版本号 bump 时应更新本页首段与日期）

---

## 应用与仓库版本

| 项 | 值 |
|----|-----|
| 桌面应用语义化版本 | **0.5.0**（以根目录 `package.json`、`distros/desktop-tauri/tauri.conf.json`、`distros/desktop-tauri/Cargo.toml` 对齐为准） |
| 默认 HTTP API（`--api`） | `http://127.0.0.1:8420`（`GET /health` 公开探活；其余路由默认强制 `OCLIVE_API_TOKEN`） |
| 用户可见变更流水 | **[CHANGELOG.md](../../CHANGELOG.md)**（中文）· **[CHANGELOG.en.md](../../CHANGELOG.en.md)**（英文，与中文同步维护条目） |

---

## 交付面（本仓库 `oclivenewnew`）

- **运行时**：Tauri 桌面端、角色包导入、`process_message` 主链、六槽模块、目录 / Remote 插件与本地 HTTP `--api`；架构见 [OCLIVE_ARCHITECTURE_OVERVIEW](OCLIVE_ARCHITECTURE_OVERVIEW.md)。
- **内核工程**：里程碑 **K0–K5** 在计划中除 **P2（OTA / 远程日志等）** 外已收口；验收留痕与 CI 见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 与根目录 [AGENTS.md](../../AGENTS.md)。
- **产品级「首发」门槛**：以 [CONTRIBUTING](../../CONTRIBUTING.md)、CI workflow 与 [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 为准。

---

## 姊妹仓与国际化

- **编写器** `oclive-pack-editor`、**VS Code 扩展** `oclive-vscode`、**插件市场站** `oclive-plugin-market`：通过角色包、插件契约与发行版 profile 协作；启动器已归档。
- **四仓 UI 文案双语基线**：见历史 [I18N_FOUR_REPO_BASELINE.md](../../handoff/archive/I18N_FOUR_REPO_BASELINE.md)。
- **创作者英文文档（`creator-docs-en/`）**：收尾范围与更新约定见 [creator-docs-en/README.md](../../creator-docs-en/README.md#documentation-bilingual-closure-baseline)（与中文 `creator-docs/` 对拍；路线图等长尾仍为中文）。

## 路线图与对齐习惯

| 需求 | 文档 |
|------|------|
| 按月愿景与阶段 | [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) |
| 体验向 backlog（试聊、市场等） | [../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| 当前工程债与冻结 | [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |

发版或改契约前：根 [README.md](../../README.md) 中的 `npm run check` / `check:release` 与 CHANGELOG 双语更新。
