# 创作者与开源文档（oclive）

本目录为 **面向创作者、侧车开发者与插件扩展者** 的文档根目录，按主题分子文件夹；亦是 **AI 接手包** 的契约百科组成部分（与 [AGENTS.md](../AGENTS.md)、[handoff/](../handoff/) 并列）。

**受众导航**：**创作者 / 插件契约** → 本目录；**终端用户** → [getting-started/USER_MANUAL.md](getting-started/USER_MANUAL.md)；**AI 维护 / 工程交接** → [handoff/](../handoff/) + [AGENTS.md](../AGENTS.md)。

**人类开发者（主仓贡献 · 不用 Cursor）**：请先 [human-docs/README.md](../human-docs/README.md)（窄入口）；需要契约细节时再回本文。

**不再使用**旧的扁平 `docs/*.md` 布局（见 `docs/README.md` 说明）。

---

## 目录结构

| 文件夹 | 内容 |
|--------|------|
| **[getting-started/](getting-started/)** | 文档总索引、[架构总览 · 单核双态](getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)（[English](../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)）、[用户手册](getting-started/USER_MANUAL.md)、[进度与目标对齐](getting-started/PROJECT_STATUS_AND_ALIGNMENT.md)、[项目现状快照](getting-started/PROJECT_CURRENT_STATUS.md)、入门与角色包工作流、[GitHub 仓库清单](getting-started/GITHUB_REPO_CHECKLIST.md) |
| **[guides/](guides/)** | [配置文件说明](guides/CONFIGURATION_FILES.md)（`plugin_state`、`ui.json`、应用数据路径等）、[mumu 模块验收清单](../handoff/distros/MUMU_UI_ACCEPTANCE_CHECKLIST.md)、[复杂情感回归清单](guides/REGRESSION_COMPLEX_EMOTION_QA.md) · 英 [`../creator-docs-en/guides/`](../creator-docs-en/guides/) |
| **[plugin-and-architecture/](plugin-and-architecture/)** | `plugin_backends` 契约、扩展点、HTTP JSON-RPC 协议、[整壳桥接 API 参考](plugin-and-architecture/BRIDGE_API_REFERENCE.md)、替换模块 · 英文总表 [`../creator-docs-en/README.md`](../creator-docs-en/README.md) |
| **[FAQ.md](FAQ.md)** | 插件与角色包常见问题 · 英文 [`../creator-docs-en/FAQ.md`](../creator-docs-en/FAQ.md) |
| **[LICENSE_POLICY.md](LICENSE_POLICY.md)** | 主程序与插件的开源协议策略（发布前最小检查） |
| **[COMPATIBILITY.md](COMPATIBILITY.md)** | 编写器与主程序版本、`ui.json` 兼容表 · 英文 [`../creator-docs-en/COMPATIBILITY.md`](../creator-docs-en/COMPATIBILITY.md) |
| **[video-script/](video-script/)** | [5 分钟工具栏插件视频脚本](video-script/PLUGIN_DEVELOPMENT_SCRIPT.md) |
| **[role-pack/](role-pack/)** | 包版本、场景、用户身份、角色包自定义 |
| **[roadmap/](roadmap/)** | [开放实验场 · 愿景摘要](roadmap/VISION_OPEN_LAB.md)、愿景与按月路线图、[体验差异化 backlog](roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)、[后日待办 · 工具链与 CI](roadmap/SOMEDAY_TOOLCHAIN_CI.md)、[市场 · 启动器联动](roadmap/MARKET_LAUNCHER_INTEGRATION.md)、[社区站愿景（三板块）](roadmap/COMMUNITY_WEB_VISION.md)、[插件区网站 IA](roadmap/PLUGIN_WEB_SECTION.md) |
| **[development/](development/)** | [轻量化与供应链基线](development/LIGHTWEIGHT_PROFILE.md)（Release、`cargo audit`、`cargo tree -d`、`cargo-bloat`） |
| **[security/](security/)** | [已知漏洞跟踪](security/KNOWN_VULNERABILITIES.md)、[安全审查范围](security/SECURITY_AUDIT_SCOPE.md) |
| **[testing/](testing/)** | [矩阵/冷启动/长稳测试指南](testing/TESTING_GUIDE.md)、[测试输出契约](testing/TEST_OUTPUT_SCHEMA.md)、[OOCP 套件](testing/OOCP_TEST_SUITE.md) |
| **[rfc/](rfc/)** | [RFC：高耦合编译模式 Monolith](rfc/RFC_OCLIVE_MONOLITH_MODE.md)（`monolith.toml`、`oclive-cli --monolith`；第一阶段已实现占位焊接） |
| **[cli/](cli/)** | [oclive-cli 脚手架指南](cli/OCLIVE_CLI_GUIDE.md) · [`plugin_backends` 参考](cli/SETTINGS_REFERENCE.md)（内核 / 无头最小工程生成） |

### 文档双语收尾基线

- **权威**：契约与角色包仍以本目录 **`creator-docs/`** 中文为准。  
- **英文镜像**：[`../creator-docs-en/README.md`](../creator-docs-en/README.md)（见该页 **Documentation bilingual closure baseline**）：入门与索引、插件契约与 `guides/`、FAQ/兼容/协议策略等已对齐；**`roadmap/`、视频脚本、部分角色包深度文**等仍为 **中文-only**，英文总索引链过去属正常。  
- **后续开发**：改契约或宿主/插件行为时，**同一变更周期**内同步更新已有英文镜像，或在 `CHANGELOG` 中注明「文档仅中文更新」。

---

## 按角色导航

| 我是谁 | 从这里开始 |
|--------|------------|
| **普通用户**（只使用桌面应用；不写角色包/插件） | [getting-started/USER_MANUAL.md](getting-started/USER_MANUAL.md) · [English](../creator-docs-en/getting-started/USER_MANUAL.md) |
| **我想做角色包（v2 蓝图）** | [role-pack/CREATOR_LEARNING_PATH.md](role-pack/CREATOR_LEARNING_PATH.md) · [ROLE_PACK_SPEC.md](role-pack/ROLE_PACK_SPEC.md)（`slot_registry` + 可选 `groups`）· v1 迁移 [V1_TO_V2_MIGRATION.md](role-pack/V1_TO_V2_MIGRATION.md)（已废弃） |
| **我想开发插件** | [plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) · 契约 [plugin-and-architecture/PLUGIN_V1.md](plugin-and-architecture/PLUGIN_V1.md) |
| **我想做硬件 / 无头内核集成** | [getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md](getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) · 总览图 [getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) |
| **维护者**（Breaking 流程、关键路径交接） | [../handoff/BREAKING_CHANGE_PROCESS.md](../handoff/BREAKING_CHANGE_PROCESS.md) · [../handoff/BUS_FACTOR_NOTES.md](../handoff/BUS_FACTOR_NOTES.md) |

---

## 从这里开始

1. 维护者先核对 **版本与变更日志入口**：[getting-started/PROJECT_CURRENT_STATUS.md](getting-started/PROJECT_CURRENT_STATUS.md)；再读 **进度与目标对齐**：[getting-started/PROJECT_STATUS_AND_ALIGNMENT.md](getting-started/PROJECT_STATUS_AND_ALIGNMENT.md)；日常仍从 **[getting-started/DOCUMENTATION_INDEX.md](getting-started/DOCUMENTATION_INDEX.md)**「快速入口」选读。  
2. 先看错误与排障：**[getting-started/ERROR_CODES.md](getting-started/ERROR_CODES.md)**（含 HTTP/Tauri 错误体、**§1.7 Sentry**）；机器码与 JSON 规范 **[KERNEL_ERROR_CODE_CONVENTION.md](getting-started/KERNEL_ERROR_CODE_CONVENTION.md)**；A3 结项 **[../handoff/A3_CLOSURE_SUMMARY.md](../handoff/A3_CLOSURE_SUMMARY.md)** / **[../handoff/A3_CLOSURE_SUMMARY.en.md](../handoff/A3_CLOSURE_SUMMARY.en.md)**。  
3. 做角色包内容： **[getting-started/CREATOR_WORKFLOW.md](getting-started/CREATOR_WORKFLOW.md)**（运行时与**独立编写器**分工、**`OCLIVE_ROLES_DIR`**）+ 仓库 **[distros/chat-pro/roles/README_MANIFEST.md](../distros/chat-pro/roles/README_MANIFEST.md)**；性格档案轴心见 **[docs/personality-archive-notes.md](../docs/personality-archive-notes.md)**，思路演进见 **[docs/design-axis-evolution.md](../docs/design-axis-evolution.md)**。编写器为另仓（如 **`oclive-pack-editor`**），包为唯一对接面。  
4. 做 HTTP 侧车： **[plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)** + **[plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**。  
5. **用户向：本机侧车 + 自带 Key 接闭源模型**：[getting-started/SIDECAR_LLM_USER_GUIDE.md](getting-started/SIDECAR_LLM_USER_GUIDE.md)。  
6. 联调示例： **[examples/remote_plugin_minimal/README.md](../examples/remote_plugin_minimal/README.md)**；**OpenAI 兼容（requests + BYOK）**：[examples/remote_plugin_openai_compat/README.md](../examples/remote_plugin_openai_compat/README.md)。
7. 管理目录插件：主界面 **`Ctrl+Shift+F`** 打开插件管理；用户常见问题见 **[FAQ.md](FAQ.md)**（含 mumu 默认模块与重置布局说明）。

---

## 与仓库其他文档的关系

| 位置 | 说明 |
|------|------|
| 根目录 **[README.md](../README.md)** | 项目简介、构建命令 |
| 根目录 **[ARCHIVE_PROJECT_HISTORY.md](../handoff/archive/ARCHIVE_PROJECT_HISTORY.md)** | 开发日志与交接材料归档索引（非创作者必读） |
| **[CONTRIBUTING.md](../CONTRIBUTING.md)** / **[SECURITY.md](../SECURITY.md)** | 贡献与安全 |
| **[CHANGELOG.md](../CHANGELOG.md)** | 版本变更 |

---

[English](../creator-docs-en/README.md)
