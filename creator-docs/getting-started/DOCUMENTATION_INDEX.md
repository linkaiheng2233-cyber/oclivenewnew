# oclive 文档索引与阅读顺序

创作者与插件相关说明位于仓库根目录 **`creator-docs/`**（按主题分子文件夹）。可按角色选择阅读路径。**插件契约等英文镜像**：[`creator-docs-en/README.md`](../../creator-docs-en/README.md)（与中文长文对拍时以 `creator-docs/` 为准；已含 Remote / 目录插件 / 桥接 / 扩展点 / 创作者架构 / FAQ / 兼容表等）。**文档双语收尾**：英文镜像范围、中文-only 长尾与更新纪律见英文 README 小节 [Documentation bilingual closure baseline](../../creator-docs-en/README.md#documentation-bilingual-closure-baseline)。

**若思路较乱、想一次看清三件套与事项分工**：先读 **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)**（项目全貌与总览）。**若要对齐「当前进度 + 未来目标 + 按用途分类的文档地图」**：读 **[PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md)**。**若只要版本号、交付面摘要与 CHANGELOG 入口**：读 **[PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md)**。

---

## 工程纪律（C2）

| 主题 | 文档 |
|------|------|
| **Breaking 变更流程**（定义、六步、兼容层、PR/迁移模板） | **[../../handoff/BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md)** |
| **关键路径交接笔记**（Bus factor：`process_message`、`PluginHost`、错误码、DB、测试/CI 等入口） | **[../../handoff/BUS_FACTOR_NOTES.md](../../handoff/BUS_FACTOR_NOTES.md)** |

---

## 学习路径

按时间盒从开始到发布；**细节仍以各专题长文与源码为准**。下列与「快速入口」中同名行互参。

| 角色 | 文档 |
|------|------|
| **普通用户**（安装 → 导入角色包 → 对话；不写包/插件） | **[USER_MANUAL.md](USER_MANUAL.md)**（[English](../../creator-docs-en/getting-started/USER_MANUAL.md)） |
| **角色包创作者** | **[../role-pack/CREATOR_LEARNING_PATH.md](../role-pack/CREATOR_LEARNING_PATH.md)**（[English](../../creator-docs-en/role-pack/CREATOR_LEARNING_PATH.md)） |
| **插件作者** | **[../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)**（[English](../../creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)） |
| **内核 / 硬件集成方** | **[KERNEL_INTEGRATOR_LEARNING_PATH.md](KERNEL_INTEGRATOR_LEARNING_PATH.md)**（[English](../../creator-docs-en/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)） |

---

## 架构总览（单核双态 · 模块三层）

| 主题 | 文档 |
|------|------|
| **架构简述、第 1–6 模块 / 第 N 设施子模块 / 后端模块插件模块（编号约定）** | **[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)**（[English](../../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)） |
| 内核工厂与三层 | [KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md) |
| 内核居中总览图 | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| Monolith（宏核态） | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) |

---

## 接口与契约速览

| 主题 | 文档 |
|------|------|
| 错误码与用户可见文案 · 提 issue 最少信息 | **[ERROR_CODES.md](ERROR_CODES.md)** |
| **`code` 与 JSON 错误体规范** | **[KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md)** |
| **`plugin_backends` 与第 1–6 模块契约** | **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** · 编号总览 **[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)** |
| Remote HTTP JSON-RPC | **[../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)** |
| 角色包磁盘格式 / `schema_version` | **[../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)** · **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)** |
| OOCP HTTP 黑盒套件（CI） | **[../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md)** |
| 目录插件整壳 `invoke` · 权限与错误码 | **[../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |

---

## 快速入口

| 我想… | 阅读 |
|------|------|
| **按角色：学习路径总览** | **[学习路径](#学习路径)** · **[接口与契约速览](#接口与契约速览)** |
| **纯终端用户：安装到日常对话（用户手册）** | **[USER_MANUAL.md](USER_MANUAL.md)**（[English](../../creator-docs-en/getting-started/USER_MANUAL.md)） |
| **角色包创作者：从零到发布（学习路径）** | **[../role-pack/CREATOR_LEARNING_PATH.md](../role-pack/CREATOR_LEARNING_PATH.md)**（[English](../../creator-docs-en/role-pack/CREATOR_LEARNING_PATH.md)） |
| **插件作者：目录 / Remote / 市场（学习路径）** | **[../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)**（[English](../../creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)） |
| **LLM 目录插件 + llama.cpp（不经 Ollama，按角色切换）** | **[examples/directory-plugin-llamacpp/README.md](../../examples/directory-plugin-llamacpp/README.md)**（[English](../../examples/directory-plugin-llamacpp/README.en.md)） |
| **内核 / 硬件集成方：脚手架到设备（学习路径）** | **[KERNEL_INTEGRATOR_LEARNING_PATH.md](KERNEL_INTEGRATOR_LEARNING_PATH.md)**（[English](../../creator-docs-en/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)） |
| **项目现状（版本、交付面、变更日志入口，短快照）** | **[PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md)** |
| **对齐进度与目标（一页：摘要 + 按用途分类的文档地图）** | **[PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md)** |
| **产品首发门槛 + 内核/平台缺口（与 K 计划、愿景互参）** | **[../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)** · 按复杂度排期 **[../../handoff/PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)**（**§四 · 硬骨头** 为下一工程焦点）· 发版勾选 **[../../handoff/PRODUCT_RELEASE_CHECKLIST.md](../../handoff/PRODUCT_RELEASE_CHECKLIST.md)** |
| **四仓双语基线（CJK 扫描、vue-i18n 挂载）** | **[../../handoff/I18N_FOUR_REPO_BASELINE.md](../../handoff/I18N_FOUR_REPO_BASELINE.md)** |
| **工作室：启动 + 创作、环境诊断、深链接、整合包** | **[../studio/USER_GUIDE.md](../studio/USER_GUIDE.md)**（[English](../../creator-docs-en/studio/USER_GUIDE.md)）· 仓库 [oclive-studio](https://github.com/oclive-app/oclive-studio) |
| **用启动器安装 zip 角色包（已废弃，见工作室）** | **[oclive-launcher README](https://github.com/oclive-app/oclive-launcher/blob/main/README.md)**（**Deprecated** → oclive-studio） |
| **高耦合编译模式（Monolith）** | [RFC 章节](#rfc)（`monolith.toml`、编译期焊接） |
| **理清项目全貌 / 人机分工 / 命令与发版清单** | **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** |
| **架构总述（契约型薄核 · 单核双态构建 · 特点）** | **[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)**（[English](../../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)） |
| **以内核为中心、六槽环绕的总览图（含 Agent/MCP/Monolith 等）** | **[KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md)** |
| **纯净内核边界、灵魂交付、嵌入式范围** | **[PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)** · **[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)**（K0–K5） |
| **平台开发者单线（脚手架 → 部署）** | **[KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md)** |
| **无头联调（`--api`，K1）** | **[examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md)** |
| 报错后如何快速定位与提 issue | **[ERROR_CODES.md](ERROR_CODES.md)**（速查表）· **机器码与 JSON 唯一规范 [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md)** · **A3 结项** [中文](../../handoff/A3_CLOSURE_SUMMARY.md) / [English](../../handoff/A3_CLOSURE_SUMMARY.en.md)（Sentry / 用户可见错误） |
| **GitHub：Dependabot、手动跑 CI、网页上要点的设置** | **[GITHUB_REPO_CHECKLIST.md](GITHUB_REPO_CHECKLIST.md)** |
| 从零了解「可替换模块 + HTTP 侧车 + 更新策略」 | **[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)**（总览，建议先读） |
| **本机侧车 + 用户自带 Key 接闭源云端模型（用户向）** | **[SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md)** |
| **侧车范例：OpenAI 兼容 API（requests）** | **[../examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)** |
| 实现侧车：请求/响应 JSON 长什么样 | **[../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**（协议全文，含示例） |
| `settings.json` 里 `plugin_backends` 每个字段含义 | **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** |
| **目录插件权限规范（A4.2 · manifest / 校验 / 运行时）** | **[../plugin-and-architecture/PLUGIN_V1.md §权限规范](../plugin-and-architecture/PLUGIN_V1.md)**（[English](../../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md)） |
| **官方 CLI 脚手架 `oclive-cli`（A/B/C；质量深耕 Z11–Z19；巩固强化 AA1–AA11；英文输出）** | **[../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)** · [KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md) |
| **内核工厂愿景（U–X + T 协作分发 + Y 持续精进）** | **[KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md)**（[English](../../creator-docs-en/getting-started/KERNEL_FACTORY_VISION.md)） |
| **`plugin_backends` 第 1–6 模块与预设、切换 remote 步骤（权威）** | **[../cli/SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)** |
| **角色包磁盘格式、多发行版对齐、`oclive pack validate`** | **[../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)** |
| **蓝图 v2（`pipeline.ocblueprint` · P0–P8 收口）** | **[../../handoff/RFC_ROLE_BLUEPRINT_V2.md](../../handoff/RFC_ROLE_BLUEPRINT_V2.md)** · **[../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md)** · **[../../handoff/BLUEPRINT_V2_DECISIONS.md](../../handoff/BLUEPRINT_V2_DECISIONS.md)** |
| **内核分层纪律（domain / infrastructure / api）** | **[../../handoff/ARCHITECTURE_LAYERING.md](../../handoff/ARCHITECTURE_LAYERING.md)** |
| **社区角色包索引 JSON 格式** | **[../role-pack/ROLE_PACK_INDEX.md](../role-pack/ROLE_PACK_INDEX.md)** |
| **目录式进程插件**（`plugins/`、`manifest.json`、整壳、`directory_plugin_invoke`、开发者模式；**含插件管理面板** `Ctrl+Shift+F`、启用/停用/拖拽排序/本地 zip 更新） | **[../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)** |
| **整壳 / 插槽 `invoke` 命令表、权限别名、错误码** | **[../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |
| **配置文件位置**（`plugin_state`、`ui.json`、`oclive_last_role_id`） | **[../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)** |
| **开源协议怎么定（主程序/官方插件/第三方）** | **[../LICENSE_POLICY.md](../LICENSE_POLICY.md)** |
| **mumu 默认前端模块、插件 FAQ（Vue 不显示、iframe 调试、依赖等；用户向 Q&A）** | **[../FAQ.md](../FAQ.md)** |
| **mumu 模块发版前验收清单** | **[../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md](../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md)** |
| **编写器与主程序版本兼容（A5 一页表 · 中英）** | **[../COMPATIBILITY.md](../COMPATIBILITY.md)**（镜像 [creator-docs-en/COMPATIBILITY.md](../../creator-docs-en/COMPATIBILITY.md)）；结项 [`../../handoff/A5_CLOSURE_SUMMARY.md`](../../handoff/A5_CLOSURE_SUMMARY.md) |
| **`memory = local`**、`_local_plugins` 清单与桥接契约 | **[../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)** |
| 在 Rust 里新增一种内置后端或注册方式 | **[../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)** |
| 只做角色包内容（manifest、场景、文案） | **[CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md)**（**`OCLIVE_ROLES_DIR`**、编写器分工、**应用内导入 zip/文件夹**）、[roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)、导入验收 [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **核心 / 可变性格档案、`personality_source`、七维视图** | **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)**（与 `roles/README_MANIFEST.md` §5.3 互参） |
| **设计思路为何从「七维为主」走到「档案轴心」** | **[docs/design-axis-evolution.md](../../docs/design-axis-evolution.md)**（旧文档保留，冲突以契约为准） |
| 编写器校验路线（与 `load_role` / crate 中期） | **[../role-pack/EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)** |
| 包版本、`schema_version`、世界观知识 `knowledge/` | **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)** · **[../role-pack/WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md) |
| 扩展点与源码文件 | **[../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)** |
| 愿景与路线图 | **[../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)** |
| **开放实验场（愿景摘要）** | **[../roadmap/VISION_OPEN_LAB.md](../roadmap/VISION_OPEN_LAB.md)** |
| 体验差异化 backlog（试聊 / 启动器依赖 / 市场 · 与愿景对照） | **[../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** |
| 后日待办（工具链 / CI · 性价比备忘，非阻塞） | **[../roadmap/SOMEDAY_TOOLCHAIN_CI.md](../roadmap/SOMEDAY_TOOLCHAIN_CI.md)** |
| 角色包 / 插件市场 · 与启动器联动（发版同发、入口与阶段划分） | **[../roadmap/MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md)** |
| 社区站愿景（网页 · 论坛 / 角色包 / 插件 三板块；Discord 取舍） | **[../roadmap/COMMUNITY_WEB_VISION.md](../roadmap/COMMUNITY_WEB_VISION.md)** |
| **插件区（网站）**信息架构与 `plugins.json` 清单 | **[../roadmap/PLUGIN_WEB_SECTION.md](../roadmap/PLUGIN_WEB_SECTION.md)** |
| **项目全貌（OVERVIEW 入口）** | **[OVERVIEW.md](OVERVIEW.md)**（同义跳转至 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)） |
| **轻量化 / `cargo audit` / `cargo-bloat` 基线** | **[../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md)** |
| **性能与已知限制（A7 · 对外披露）** | **[PERFORMANCE.md](PERFORMANCE.md)**（[English](../../creator-docs-en/getting-started/PERFORMANCE.md)） |
| **支持入口（A9 · GitHub Issues）** | 根目录 [README.md](../../README.md)「**支持**」小节 · [`.github/ISSUE_TEMPLATE`](../../.github/ISSUE_TEMPLATE) |
| **免责声明（A10 · 模型 / 插件 / 数据）** | **[../legal/DISCLAIMER.md](../legal/DISCLAIMER.md)**（[English](../../creator-docs-en/legal/DISCLAIMER.md)） |
| **已知漏洞（cargo-audit）与升级路线** | **[../security/KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md)** |
| **安全审查范围与局限** | **[../security/SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md)** |
| **测试输出契约、OOCP 套件、`invoke` 热路径矩阵、A1 收口摘要、插件集成测说明** | **[../testing/TEST_OUTPUT_SCHEMA.md](../testing/TEST_OUTPUT_SCHEMA.md)** · **[../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md)** · **[../testing/OVERVIEW.md](../testing/OVERVIEW.md)** · **[../../handoff/INVOKE_HOTPATH_MATRIX.md](../../handoff/INVOKE_HOTPATH_MATRIX.md)** · **[../../handoff/A1_CLOSURE_SUMMARY.md](../../handoff/A1_CLOSURE_SUMMARY.md)** · **[../testing/ADAPTING_TEST_PLUGIN.md](../testing/ADAPTING_TEST_PLUGIN.md)** · **[../testing/L03_GENERATION_CANCEL.md](../testing/L03_GENERATION_CANCEL.md)** |
| **AB 确定性加固：`narrative_hint` 契约、模糊测试** | **[../testing/NARRATIVE_HINT_CONTRACT.md](../testing/NARRATIVE_HINT_CONTRACT.md)** · **[../testing/FUZZING.md](../testing/FUZZING.md)** |

---

## RFC

架构级设计与实现路线以 RFC 收敛（**草案不代表已合入代码或脚手架行为**）。

| 文档 | 说明 |
|------|------|
| **[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)** | **高耦合编译模式（Monolith）**：`monolith.toml`、`--monolith`、双 `[[bin]]`；**`build` / `bench`** 子命令与部分焊接（见 RFC 与 CLI 指南）。 |
| **[RFC_STUDIO_MERGE.md](../rfc/RFC_STUDIO_MERGE.md)** | **工作室（合并启动器 + 编写器）**：新仓 `oclive-studio`、两安装物叙事、配置与验收。 |

---

## 推荐阅读顺序（创作者 / 侧车开发者）

1. [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) — 角色包目录与加载方式  
2. [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — `plugin_backends` 六类可替换后端（memory / emotion / event / prompt / llm / **agent**）  
2b. [../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) — 目录式插件（与 `directory` 枚举、`directory_plugins` 槽位）  
3. [../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) — 三种扩展方式、环境变量、与「热更新」边界  
4. [SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md) — **本机侧车 + BYOK**（接闭源 API 的路径；与启动器配合）  
5. [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — JSON-RPC 方法、params/result、**完整 JSON 示例**  
6. [examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md) — 最小 Python 侧车联调  
6b. [examples/directory-plugin-minimal/README.md](../../examples/directory-plugin-minimal/README.md) — 最小目录插件（manifest + 整壳 + Node RPC）  
6c. [examples/directory-plugin-llamacpp/README.md](../../examples/directory-plugin-llamacpp/README.md) — LLM 槽目录插件 + 本机 llama.cpp HTTP（[English](../../examples/directory-plugin-llamacpp/README.en.md)）  
7. [examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md) — OpenAI 兼容 `chat/completions` 范例（BYOK）  
8. [examples/common/README.md](../../examples/common/README.md) — 侧车示例共用 JSON-RPC / 非 LLM 占位模块  

---

## 推荐阅读顺序（宿主 / Rust 贡献者）

1. [../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)  
2. [../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)  
3. 源码：`src-tauri/src/domain/plugin_host.rs`、`src-tauri/src/infrastructure/remote_plugin/`、**`src-tauri/src/infrastructure/directory_plugins/`**（目录插件扫描与懒启动）  
4. 集成烟测（`PluginHost::resolve_for_role` + `builtin_v2` 枚举）：[`src-tauri/tests/plugin_backends_v2_resolve.rs`](../../src-tauri/tests/plugin_backends_v2_resolve.rs)（`cargo test --test plugin_backends_v2_resolve`）

---

## 与仓库根 README 的关系

项目总览、构建命令、测试见仓库根目录 **[README.md](../../README.md)**；**插件与侧车细节以 `creator-docs/` 为准**。

- **[错误码与排障速查](ERROR_CODES.md)**（运行时 HTTP / JSON-RPC 与提 issue 最少信息）

---

[English](../../creator-docs-en/getting-started/DOCUMENTATION_INDEX.md)
