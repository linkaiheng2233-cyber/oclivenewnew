# oclive 文档索引与阅读顺序

- [中文](#oclive-文档索引与阅读顺序)
- [English](#documentation-index--reading-order)

---

## Documentation index & reading order

Creator and plugin docs live under the repo root `creator-docs/` (grouped by topic).

- If you want a one-pass mental model of the toolchain and responsibilities, start with **[`PROJECT_OVERVIEW.md`](PROJECT_OVERVIEW.md)**.
- Most readers can then pick a path from the “Quick links” table below.

创作者与插件相关说明位于仓库根目录 **`creator-docs/`**（按主题分子文件夹）。可按角色选择阅读路径。

**若思路较乱、想一次看清三件套与事项分工**：先读 **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)**（项目全貌与总览）。

---

## 快速入口

| 我想… | 阅读 |
|------|------|
| **第一次克隆仓库、搭环境、跑通桌面首次对话（开发者最短路径）** | **[DEVELOPER_QUICKSTART.md](DEVELOPER_QUICKSTART.md)** |
| **编写或调试 `process_message` 入口蓝图（`pipeline.ocblueprint`、BRANCH、PARALLEL）** | **[../kernel/PIPELINE_SCHEMA.md](../kernel/PIPELINE_SCHEMA.md)** · 示例 **[`examples/blueprints/`](../../examples/blueprints/)** |
| **用启动器安装 zip 角色包、选本机 Ollama 模型、一键 pull** | **[oclive-launcher README](https://github.com/oclive-app/oclive-launcher/blob/main/README.md)**（独立仓库） |
| **理清项目全貌 / 人机分工 / 命令与发版清单** | **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** |
| 报错后如何快速定位与提 issue | **[ERROR_CODES.md](ERROR_CODES.md)** |
| **GitHub：Dependabot、手动跑 CI、网页上要点的设置** | **[GITHUB_REPO_CHECKLIST.md](GITHUB_REPO_CHECKLIST.md)** |
| 从零了解「可替换模块 + HTTP 侧车 + 更新策略」 | **[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)**（总览，建议先读） |
| **本机侧车 + 用户自带 Key 接闭源云端模型（用户向）** | **[SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md)** |
| **侧车范例：OpenAI 兼容 API（requests）** | **[../examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)** |
| 实现侧车：请求/响应 JSON 长什么样 | **[../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**（协议全文，含示例） |
| `settings.json` 里 `plugin_backends` 每个字段含义 | **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** |
| **目录式进程插件**（`plugins/`、`manifest.json`、整壳、`directory_plugin_invoke`、开发者模式） | **[../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)** |
| **本地 Llama 目录插件（官方附带）**：启用、模型目录、验收 | **[../plugin-and-architecture/LLAMA_DIRECTORY_PLUGIN_V1.md](../plugin-and-architecture/LLAMA_DIRECTORY_PLUGIN_V1.md)** |
| **整壳 / 插槽 `invoke` 命令表、权限别名、错误码** | **[../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |
| **插件市场索引 `plugins.json`（plugin/module/profile 三类条目）** | **[../plugin-and-architecture/PLUGIN_MARKET_INDEX_V1.md](../plugin-and-architecture/PLUGIN_MARKET_INDEX_V1.md)** |
| **角色包市场索引 `roles.json`（多镜像 + SHA-256 校验）** | **[../role-pack/ROLE_MARKET_INDEX_V1.md](../role-pack/ROLE_MARKET_INDEX_V1.md)** |
| **角色包使用后反馈（半私密）**（用户提交 → 编写器收件箱） | **[../role-pack/ROLE_FEEDBACK_V1.md](../role-pack/ROLE_FEEDBACK_V1.md)** |
| **本地导入（文件夹投放）**（imports/ 目录、Module/Profile 同款格式、安全确认） | **[../LOCAL_IMPORTS_V1.md](../LOCAL_IMPORTS_V1.md)** |
| **配置文件位置**（`plugin_state`、`ui.json`、`oclive_last_role_id`） | **[../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)** |
| **统一设置中心**（L1–L4 分级、沉浸总闸、侧栏分组提示、快捷键页内提示、全局恢复默认） | **[../kernel/SETTINGS_TIERING.md](../kernel/SETTINGS_TIERING.md)** · IA 与总闸 **[../../handoff/SETTINGS_CENTER_MASTER_SWITCH_IA.md](../../handoff/SETTINGS_CENTER_MASTER_SWITCH_IA.md)** |
| **开源协议怎么定（主程序/官方插件/第三方）** | **[../LICENSE_POLICY.md](../LICENSE_POLICY.md)** |
| **我想管理插件（启用/停用/拖拽排序/本地 zip 更新）** | **[../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)**（插件管理面板，快捷键 `Ctrl+Shift+F`） |
| **第九模块：专家模型设施（ExpertGraph / 本地侧车 + 云端 + 事件记忆 / Prompt 风格覆盖）** | **[../kernel/MODULE_9_EXPERT_MODELS_FACILITY.md](../kernel/MODULE_9_EXPERT_MODELS_FACILITY.md)** · **`.oclexpert` 格式** **[../kernel/OCLEXPERT_FORMAT.md](../kernel/OCLEXPERT_FORMAT.md)** · 边界总览 **[../kernel/KERNEL_BOUNDARY.md](../kernel/KERNEL_BOUNDARY.md)** · Tauri 命令归属 **[../kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md](../kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md)** |
| **内核迁入收尾** | **[../../handoff/KERNEL_MIGRATION_COMPLETE.md](../../handoff/KERNEL_MIGRATION_COMPLETE.md)** |
| **极致轻量化（runtime `Cargo` 特性、OOCP、`invoke` 分组、依赖/`http_api` 拟定）** | **[../kernel/LIGHTWEIGHT_PROFILE.md](../kernel/LIGHTWEIGHT_PROFILE.md)**（与 [KERNEL_BOUNDARY.md](../kernel/KERNEL_BOUNDARY.md) §5.1 互参）；设施 `classic` 门控审计 **[../kernel/FACILITY_CLASSIC_ALGORITHMS_AUDIT.md](../kernel/FACILITY_CLASSIC_ALGORITHMS_AUDIT.md)** |
| **内核工程质量与生态路线（P0–P2，测试/SDK/crates.io）** | **[../../handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md](../../handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md)** |
| **内核 SDK（KernelAppState、process_message、kernel_server）** | **[../kernel/KERNEL_SDK.md](../kernel/KERNEL_SDK.md)** |
| **可编程对话流水线 · 蓝图 `pipeline.ocblueprint`（JSON Schema、白名单原子、`onFailure`）** | **[../kernel/PIPELINE_SCHEMA.md](../kernel/PIPELINE_SCHEMA.md)** · 官方示例目录 **[`examples/blueprints/`](../../examples/blueprints/)**（含 `simple_companion` / `minimal_chat` / `memory_heavy` / `deep_empathy`） |
| **如何接入内核（内嵌 / HTTP / OOCP 三模式、环境变量、排障）** | **[INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)** |
| **Python OOCP 客户端 SDK（`/health`、`/chat`、Bearer）** | **[../../sdk/python/README.md](../../sdk/python/README.md)** |
| **事件引擎剥离状态（builtin / directory 示例）** | **[../kernel/EVENT_ENGINE_EXTRACTION_STATUS.md](../kernel/EVENT_ENGINE_EXTRACTION_STATUS.md)** |
| **Linux 无头内核引擎（部署、Docker、systemd、多模态外挂）** | **权威部署** **[../../docs/LINUX_KERNEL_DEPLOY.md](../../docs/LINUX_KERNEL_DEPLOY.md)** · 日志 **[../../docs/LOGGING_GUIDE.md](../../docs/LOGGING_GUIDE.md)** · 路线说明 **[../../docs/LINUX_KERNEL_ENGINE.md](../../docs/LINUX_KERNEL_ENGINE.md)** · 合成模板 **[../../delivery/README.md](../../delivery/README.md)** |
| **HTTP 调用内核试聊（`/health`、`/chat`）** | **[../../examples/kernel_remote_simple/README.md](../../examples/kernel_remote_simple/README.md)** |
| **场景化建设：AI 情感陪伴玩偶（硬件 + 角色包 + 侧车 + systemd）** | **[../scenarios/DOLL_GUIDE.md](../scenarios/DOLL_GUIDE.md)** |
| **目录插件极简侧车（manifest + JSON-RPC）** | **[../../examples/kernel_directory_plugin_simple/README.md](../../examples/kernel_directory_plugin_simple/README.md)** |
| **PluginHost 调度、降级链、接新后端 / 设施 crate** | **[../kernel/PLUGIN_HOST_DEVELOPER_GUIDE.md](../kernel/PLUGIN_HOST_DEVELOPER_GUIDE.md)** |
| **Kernel Baseline v1.0（已冻结语义基线）** | **[../kernel/KERNEL_BASELINE_V1.md](../kernel/KERNEL_BASELINE_V1.md)** · 发版前检查 **[../kernel/KERNEL_RELEASE_CHECKLIST_V1.md](../kernel/KERNEL_RELEASE_CHECKLIST_V1.md)** |
| **mumu 默认前端模块（chat.header / chat_toolbar / role.detail / sidebar / settings.panel）** | **[../FAQ.md](../FAQ.md)**（用户向 Q&A） |
| **mumu 模块发版前验收清单** | **[../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md](../guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md)** |
| **插件 FAQ（Vue 不显示、iframe 调试、依赖等）** | **[../FAQ.md](../FAQ.md)** |
| **编写器与主程序版本兼容** | **[../COMPATIBILITY.md](../COMPATIBILITY.md)** |
| **`memory = local`**、`_local_plugins` 清单与桥接契约 | **[../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)** |
| 在 Rust 里新增一种内置后端或注册方式 | **[../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)** |
| 只做角色包内容（manifest、场景、文案） | **[CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md)**（**`OCLIVE_ROLES_DIR`**、编写器分工、**应用内导入 zip/文件夹**）、[roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)、导入验收 [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **核心 / 可变性格档案、`personality_source`、七维视图** | **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)**（与 `roles/README_MANIFEST.md` §5.3 互参） |
| **设计思路为何从「七维为主」走到「档案轴心」** | **[docs/design-axis-evolution.md](../../docs/design-axis-evolution.md)**（旧文档保留，冲突以契约为准） |
| 编写器校验路线（与 `load_role` / crate 中期） | **[../role-pack/EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)** |
| 包版本、`schema_version`、世界观知识 `knowledge/` | **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)** · **[../role-pack/WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)** |
| 扩展点与源码文件 | **[../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)** |
| 愿景与路线图 | **[../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)** |
| **开放实验场（愿景摘要）** | **[../roadmap/VISION_OPEN_LAB.md](../roadmap/VISION_OPEN_LAB.md)** |
| 体验差异化 backlog（试聊 / 启动器依赖 / 市场 · 与愿景对照） | **[../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** |
| 后日待办（工具链 / CI · 性价比备忘，非阻塞） | **[../roadmap/SOMEDAY_TOOLCHAIN_CI.md](../roadmap/SOMEDAY_TOOLCHAIN_CI.md)** |
| 角色包 / 插件市场 · 与启动器联动（发版同发、入口与阶段划分） | **[../roadmap/MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md)** |
| 社区站愿景（网页 · 论坛 / 角色包 / 插件 三板块；Discord 取舍） | **[../roadmap/COMMUNITY_WEB_VISION.md](../roadmap/COMMUNITY_WEB_VISION.md)** |
| **插件区（网站）**信息架构与 `plugins.json` 清单 | **[../roadmap/PLUGIN_WEB_SECTION.md](../roadmap/PLUGIN_WEB_SECTION.md)** |

---

## 推荐阅读顺序（创作者 / 侧车开发者）

1. [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) — 角色包目录与加载方式  
2. [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — `plugin_backends` 五类后端  
2b. [../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) — 目录式插件（与 `directory` 枚举、`directory_plugins` 槽位）  
3. [../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) — 三种扩展方式、环境变量、与「热更新」边界  
4. [SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md) — **本机侧车 + BYOK**（接闭源 API 的路径；与启动器配合）  
5. [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — JSON-RPC 方法、params/result、**完整 JSON 示例**  
6. [examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md) — 最小 Python 侧车联调  
6b. [examples/directory-plugin-minimal/README.md](../../examples/directory-plugin-minimal/README.md) — 最小目录插件（manifest + 整壳 + Node RPC）  
7. [examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md) — OpenAI 兼容 `chat/completions` 范例（BYOK）  
8. [examples/common/README.md](../../examples/common/README.md) — 侧车示例共用 JSON-RPC / 非 LLM 占位模块  

---

## 推荐阅读顺序（宿主 / Rust 贡献者）

1. [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md) — 内嵌 / HTTP / OOCP 三模式与排障入口  
2. [../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)  
3. [../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)  
4. [../kernel/PLUGIN_HOST_DEVELOPER_GUIDE.md](../kernel/PLUGIN_HOST_DEVELOPER_GUIDE.md) — `PluginHost` / `BackendRegistry`、会话覆盖与降级链  
5. [../kernel/MODULE_9_EXPERT_MODELS_FACILITY.md](../kernel/MODULE_9_EXPERT_MODELS_FACILITY.md) — 第九模块术语与与 `plugin_backends` 的边界  
6. 源码：`crates/oclive_kernel_runtime` 下 `domain/plugin_host`、`infrastructure/remote_plugin`、`infrastructure/directory_plugins`；桌面另见 `src-tauri/.../directory_plugins/watcher.rs` 与 `plugin_installer.rs`（路径 + `rescan`）  

---

## 与仓库根 README 的关系

项目总览、构建命令、测试见仓库根目录 **[README.md](../../README.md)**；**插件与侧车细节以 `creator-docs/` 为准**。
