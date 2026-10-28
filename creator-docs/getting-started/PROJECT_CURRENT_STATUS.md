# 项目现状（事实快照）

**SSOT 范围**：当前版本与候选发布状态的事实快照；版本规则由 [RELEASE_VERSIONING](../development/RELEASE_VERSIONING.md) 拥有，运行时职责由 MODULE_MAP 拥有。其他入口链接本页，不复制当前版本表。

[English](../../creator-docs-en/getting-started/PROJECT_CURRENT_STATUS.md)

**快照日期**：2026-10-29。随版本或交付状态变化更新；文档更新本身不产生新的运行验收。

---

## 应用与仓库版本

| 项 | 值 |
|----|-----|
| 桌面应用语义化版本 | **0.6.0-rc.2**（[package.json](../../package.json)、[Tauri 配置](../../distros/desktop-tauri/tauri.conf.json)、[桌面 manifest](../../distros/desktop-tauri/Cargo.toml) 对齐；候选版，不等于公开发行） |
| Windows MSI 数值版本 | **0.6.0.2**；与桌面候选版对应，独立于 CLI 和内核版本 |
| CLI / runtime / validation | **0.1.0 / 0.2.0 / 0.1.0**；分别见 [CLI](../../kernel/crates/oclive-cli/Cargo.toml)、[runtime](../../kernel/crates/oclive_kernel_runtime/Cargo.toml)、[validation](../../kernel/crates/oclive_validation/Cargo.toml) |
| 默认 HTTP API（`--api`） | `http://127.0.0.1:8420`（`GET /health` 公开探活；其余路由默认强制 `OCLIVE_API_TOKEN`） |
| 用户可见变更流水 | **[CHANGELOG.md](../../CHANGELOG.md)**（中文）· **[CHANGELOG.en.md](../../CHANGELOG.en.md)**（英文，与中文同步维护条目） |

---

## 交付面（本仓库 `oclivenewnew`）

- **2026-10-28–29 源码后续修补**：角色文件/数据库失败恢复、授权落盘一致性、插件地址/资产/解压边界、网页安装确认、认证快照轮询与渲染恢复见[分批执行记录](../../handoff/debt-marathon/waves/WAVE-20261028-REVIEW-FIXES.md)；同步 RPC、迁移恢复诊断和镜像写序见[Host 可靠性续接](../../handoff/debt-marathon/waves/WAVE-20261029-HOST-RELIABILITY.md)，按各记录核当前验证状态。未重打安装包、未公开发布，下面已有候选包不包含这些修复；不要把源码测试当作新安装包或远端 CI 的验收。

- **Chat Pro 内测候选**：默认角色库 → 当前角色运行空间，各角色最近对话分开，身份/关系、记录、模型与连接、分类设置、帮助与恢复为日常入口；使用说明见 [USER_MANUAL](USER_MANUAL.md)，旧壳和延后能力见 [MODULE_MAP §13.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#chatpro-deferred-surfaces)。安装包不附带模型运行器或权重。
- **候选验收边界**：本地标签 `baseline/chatpro-0.6.0-rc.2-internal-20261027` 绑定 `b4341eb22e987ffef4a19ccad3784e2402cd2bd1`；该源码的发行/集成门禁、浏览器回归及隔离安装/启动已在本机通过。候选包未签名、未公开发布；该提交的远端 CI、全新 Windows 环境和这份新包的真实模型聊天仍未验。后续纯文档提交不冒充重新执行这批程序测试。
- **运行时**：Tauri 桌面端、角色包导入、`process_message` 主链、六槽模块、目录 / Remote 插件、本地 HTTP `--api`，以及不启动 Tauri/HTTP 的稳定 Rust 进程内门面 **`oclive_kernel_host::OcliveKernel`**；架构见 [OCLIVE_ARCHITECTURE_OVERVIEW](OCLIVE_ARCHITECTURE_OVERVIEW.md)。
- **候选之后的源码修补**：内部 workspace 安装解析、认证诊断脱敏及云端探测的保存/生命周期边界已修补，执行证据与未验范围见[本轮专项报告续接](../../handoff/debt-marathon/waves/WAVE-20261027-CHATPRO-ROUND-REVIEW.md#review-fixes)。桌面已有的上述 rc.2 安装包仍绑定 `b4341eb2`，没有自动包含这些后续代码。
- **内核工程**：里程碑 **K0–K5** 的代码路径已收口；进程内完整回合已验证。**P2（OTA / 远程日志等）** 与 **V-EMBED-01 的 Linux/ARM/真机、资源预算和长时 soak** 仍未完成；验收留痕见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 与根目录 [AGENTS.md](../../AGENTS.md)。
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
