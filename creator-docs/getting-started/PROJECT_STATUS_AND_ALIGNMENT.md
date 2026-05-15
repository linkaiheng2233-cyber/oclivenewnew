# 项目进度与目标对齐（维护者速览）

**一页对照**：当前做到哪里、权威文档在何处、下一程以什么为准。**不替代**各专题正文；细节请 always follow 链接内最新表述。

[English](../../creator-docs-en/getting-started/PROJECT_STATUS_AND_ALIGNMENT.md)

---

## 与相近文档的分工（避免重复阅读）

| 文档 | 侧重 |
|------|------|
| **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** | 三件套仓库分工、常用命令、发版与检查习惯 |
| **[PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md)** | **事实快照**：版本号、交付面摘要、CHANGELOG 中英入口、姊妹仓与 i18n 指针 |
| **[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)** | 内核里程碑 **K0–K5**、北极星、**验收留痕**（本地/CI） |
| **[../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)** | 按月愿景与阶段目标 |
| **[../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** | 体验向 backlog（试聊、启动器、市场等），**不替代**月度路线图 |
| **[../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)** | **产品级 P0–P2** 与 **内核/平台 B 区**缺口清单；与内核计划 §A 互参 |

---

## 文档地图（按用途分类）

### 1. 用户手册 / 用户向

| 主题 | 入口 |
|------|------|
| 总索引与快速入口 | [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) |
| **项目现状（版本、交付面、变更日志入口）** | [PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md) |
| 主程序常见问题（含 mumu 默认模块、插件、界面） | [../FAQ.md](../FAQ.md) · [英文 FAQ](../../creator-docs-en/FAQ.md) |
| 错误码与排障、提 issue 最少信息 | [ERROR_CODES.md](ERROR_CODES.md) |
| 本机侧车 + BYOK（闭源云端） | [SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md) |
| 配置文件与数据路径 | [../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) |
| mumu 模块发版前验收 | [../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md](../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md) |
| 编写器与主程序版本兼容 | [../COMPATIBILITY.md](../COMPATIBILITY.md) |
| 创作者工作流（包、目录、导入） | [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |

### 2. 模块说明 / 架构与契约（六槽、插件、协议）

| 主题 | 入口 |
|------|------|
| 内核与六槽总览（图 + 与源码锚点） | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| `plugin_backends` 字段与枚举 | [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| 目录式插件、管理面板、权限与降级 | [../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| 整壳 / 插槽 `invoke`、权限别名 | [../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md) |
| Remote JSON-RPC 协议与示例 | [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| 扩展方式总览（侧车 / 目录 / 内置） | [../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |
| 扩展点与源码文件索引 | [../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md) |
| `memory = local` 与桥接 | [../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md) |
| 角色包磁盘规范、RobotSoulPack 等 | [../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |

### 3. 内核与平台 / 无头、脚手架、Monolith

| 主题 | 入口 |
|------|------|
| 纯净内核边界与形态表 | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| K0–K5 实施计划与验收留痕 | [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) |
| 平台开发者单线（脚手架 → 部署） | [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) |
| 无头最小闭环（`--api`） | [../../examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md) |
| `oclive-cli` 与 Monolith RFC | [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) · [../rfc/RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| `plugin_backends` 权威键说明 | [../cli/SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |

### 4. 测试与质量

| 主题 | 入口 |
|------|------|
| 测试分层总览 | [../testing/OVERVIEW.md](../testing/OVERVIEW.md) |
| OOCP HTTP 黑盒（S0–S11） | [../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) |
| 测试输出契约 | [../testing/TEST_OUTPUT_SCHEMA.md](../testing/TEST_OUTPUT_SCHEMA.md) |
| 供应链与轻量化基线 | [../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) |
| 已知漏洞与升级路线 | [../security/KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md) |

### 5. 愿景、生态与市场（中长期）

| 主题 | 入口 |
|------|------|
| 开放实验场摘要 | [../roadmap/VISION_OPEN_LAB.md](../roadmap/VISION_OPEN_LAB.md) |
| 按月路线图 | [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) |
| 体验差异化 backlog | [../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| 市场 · 启动器联动等 | [../roadmap/MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md) 等 `roadmap/` 内专题 |

### 6. Handoff 与四仓协作（仓库 `handoff/`）

| 主题 | 入口 |
|------|------|
| 产品级 + 内核缺口合并清单 | [../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) |
| 四仓 i18n 基线与 Han 扫描说明 | [../../handoff/I18N_FOUR_REPO_BASELINE.md](../../handoff/I18N_FOUR_REPO_BASELINE.md) |

---

## 现有进度摘要（与目标对齐）

- **内核（K0–K5）**：除 **P2（OTA / 远程日志）** 外，已在 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 标为收口；**验收留痕**与 CI `oocp-test-suite` 见该文与 [AGENTS.md](../../AGENTS.md)。
- **产品级首发**：仍按 gap 清单 **§A** 为硬门槛集合；与「内核里程碑」解耦排期，见 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§D**。**A1.1a**（HTTP 进程重启）与 **A1.2**（`invoke` 宿主热路径 9 条，[`INVOKE_HOTPATH_MATRIX.md`](../../handoff/INVOKE_HOTPATH_MATRIX.md)）已入库；**下一默认焦点**为 **A1.1b**（GUI / 安装器）及 [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md) **§四** 其余硬骨头，逐项单独立项。
- **体验与生态**：以 [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) + [BACKLOG…](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) 对照，避免与内核表格混为一谈。
- **创作者文档双语（`creator-docs-en/`）**：主干（索引、插件契约、`guides/`、LICENSE、FAQ 等）已与中文总索引对拍收尾；**`roadmap/` 等愿景长文**仍以 `creator-docs/` 为准。后续契约或行为变更时同步更新英文镜像或于 CHANGELOG 声明；约定全文见 [creator-docs-en/README.md](../../creator-docs-en/README.md#documentation-bilingual-closure-baseline)。

---

## 未来目标确认（权威来源一览）

| 方向 | 以何为准 |
|------|-----------|
| 内核后续（含 P2） | [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) §K5 与「近期动作」 |
| 产品 P0–P2 | [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) §A–§C |
| 月度/开放实验叙事 | [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) · [VISION_OPEN_LAB.md](../roadmap/VISION_OPEN_LAB.md) |

**实时对齐习惯**：发版或改契约前跑 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) 中的检查命令；改内核边界时同步 [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) 与内核计划。

---

## 索引去重说明（快速入口表）

[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) 中 **用户 FAQ** 已合并为单行指向 [FAQ.md](../FAQ.md)（原先「mumu 模块」与「插件 FAQ」重复链至同一文件）。**目录式插件**规范与用户操作仍统一以 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) 为准（含插件管理面板快捷键说明）。
