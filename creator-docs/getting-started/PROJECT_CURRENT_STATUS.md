# 项目现状（事实快照）

**SSOT 范围**：当前版本与候选发布状态的事实快照；版本规则由 [RELEASE_VERSIONING](../development/RELEASE_VERSIONING.md) 拥有，运行时职责由 MODULE_MAP 拥有。其他入口链接本页，不复制当前版本表。

[English](../../creator-docs-en/getting-started/PROJECT_CURRENT_STATUS.md)

**快照日期**：2026-11-01。随版本或交付状态变化更新；文档更新本身不产生新的运行验收。

---

## 应用与仓库版本

| 项 | 值 |
|----|-----|
| 桌面应用语义化版本 | **0.6.0-rc.4**（[package.json](../../package.json)、[Tauri 配置](../../distros/desktop-tauri/tauri.conf.json)、[桌面 manifest](../../distros/desktop-tauri/Cargo.toml) 对齐；候选版，不等于公开发行） |
| Windows MSI 数值版本 | **0.6.0.4**；与桌面候选版对应，独立于 CLI 和内核版本 |
| CLI / runtime / validation | **0.1.0 / 0.2.0 / 0.1.0**；分别见 [CLI](../../kernel/crates/oclive-cli/Cargo.toml)、[runtime](../../kernel/crates/oclive_kernel_runtime/Cargo.toml)、[validation](../../kernel/crates/oclive_validation/Cargo.toml) |
| 默认 HTTP API（`--api`） | `http://127.0.0.1:8420`（`GET /health` 公开探活；其余路由默认强制 `OCLIVE_API_TOKEN`） |
| 用户可见变更流水 | **[CHANGELOG.md](../../CHANGELOG.md)**（中文）· **[CHANGELOG.en.md](../../CHANGELOG.en.md)**（英文，与中文同步维护条目） |

---

## 交付面（本仓库 `oclivenewnew`）

- **2026-11-01 rc.4 修补候选**：针对 Windows 迁移失败后，SQLite WAL/SHM 等待最后一个句柄关闭而返回拒绝访问的情况，增加仅限附属文件删除的有界等待。持续失败仍保留主要迁移原因和备份；不扩大备份读取或主库复制的重试。本候选的门禁、构建和验收应分别绑定最终 SHA 与随包清单，不能继承 rc.3 的通过结论。首发使用路径按维护者选择在本机隔离用户数据验收；全新 Windows 系统仍列为未验边界。

- **2026-10-30 rc.3 内测候选**：本轮汇总角色工作区与后续可靠性修补，执行与发包边界见 [rc.3 波次](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261030-CHATPRO-RC3.md)。桌面旧 rc.2 目录和 ZIP 已校验归档；新的安装器、随包内核与文档须绑定同一冻结提交，以 `PACKAGE-MANIFEST.txt` 和 `SHA256SUMS.txt` 核对本机交付物。未公开发行。

- **Chat Pro 内测候选**：默认角色库 → 当前角色运行空间，各角色最近对话分开，身份/关系、记录、模型与连接、分类设置、帮助与恢复为日常入口；使用说明见 [USER_MANUAL](USER_MANUAL.md)，旧壳和延后能力见 [MODULE_MAP §13.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#chatpro-deferred-surfaces)。安装包不附带模型运行器或权重。
- **候选验收边界**：rc.3 独立执行源码门禁、隔离安装/升级与原生启动检查，具体结果按本轮对应 SHA 的记录和随包清单判定。上轮 rc.2 的本机通过记录仅属于 `b4341eb22e987ffef4a19ccad3784e2402cd2bd1`，不代证新包。安装器未签名；全新 Windows 环境与完整产品人工验收仍须单独安排。
- **运行时**：Tauri 桌面端、角色包导入、`process_message` 主链、六槽模块、目录 / Remote 插件、本地 HTTP `--api`，以及不启动 Tauri/HTTP 的稳定 Rust 进程内门面 **`oclive_kernel_host::OcliveKernel`**；架构见 [OCLIVE_ARCHITECTURE_OVERVIEW](OCLIVE_ARCHITECTURE_OVERVIEW.md)。
- **候选合入范围**：内部 workspace 解析、认证诊断脱敏与云端探测的保存/生命周期边界见 [首轮修补](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261027-CHATPRO-ROUND-REVIEW.md#review-fixes)；角色/授权/插件/快照恢复见 [分批修补](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261028-REVIEW-FIXES.md)，RPC/迁移/镜像顺序见 [Host 续接](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261029-HOST-RELIABILITY.md)。各波次证据仍按其源码 SHA 阅读，旧 rc.2 包不自动包含这些改动。
- **内核工程**：里程碑 **K0–K5** 的代码路径已收口；进程内完整回合已验证。**P2（OTA / 远程日志等）** 与 **V-EMBED-01 的 Linux/ARM/真机、资源预算和长时 soak** 仍未完成；验收留痕见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 与根目录 [AGENTS.md](../../AGENTS.md)。
- **产品级「首发」门槛**：以 [CONTRIBUTING](../../CONTRIBUTING.md)、CI workflow 与 [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 为准。

---

## 姊妹仓与国际化

- **编写器** `oclive-pack-editor`、**VS Code 扩展** `oclive-vscode`、**插件市场站** `oclive-plugin-market`：通过角色包、插件契约与发行版 profile 协作；启动器已归档。
- **四仓 UI 文案双语基线**：见历史 [I18N_FOUR_REPO_BASELINE.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/archive/I18N_FOUR_REPO_BASELINE.md)。
- **创作者英文文档（`creator-docs-en/`）**：收尾范围与更新约定见 [creator-docs-en/README.md](../../creator-docs-en/README.md#documentation-bilingual-closure-baseline)（与中文 `creator-docs/` 对拍；路线图等长尾仍为中文）。

## 路线图与对齐习惯

| 需求 | 文档 |
|------|------|
| 按月愿景与阶段 | [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) |
| 体验向 backlog（试聊、市场等） | [../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| 当前工程债与冻结 | [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |

发版或改契约前：根 [README.md](../../README.md) 中的 `npm run check` / `check:release` 与 CHANGELOG 双语更新。
