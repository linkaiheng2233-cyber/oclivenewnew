# oclive 文档索引与阅读顺序

创作者与插件相关说明位于仓库根目录 **`creator-docs/`**（按主题分子文件夹）。可按角色选择阅读路径。

**若思路较乱、想一次看清三件套与事项分工**：先读 **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)**（项目全貌与总览）。

---

## 快速入口

| 我想… | 阅读 |
|------|------|
| **用启动器安装 zip 角色包、选本机 Ollama 模型、一键 pull** | **[oclive-launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md)**（独立仓库） |
| **理清项目全貌 / 人机分工 / 命令与发版清单** | **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** |
| 报错后如何快速定位与提 issue | **[ERROR_CODES.md](ERROR_CODES.md)** |
| **GitHub：Dependabot、手动跑 CI、网页上要点的设置** | **[GITHUB_REPO_CHECKLIST.md](GITHUB_REPO_CHECKLIST.md)** |
| 从零了解「可替换模块 + HTTP 侧车 + 更新策略」 | **[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)**（总览，建议先读） |
| **本机侧车 + 用户自带 Key 接闭源云端模型（用户向）** | **[SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md)** |
| **侧车范例：OpenAI 兼容 API（requests）** | **[../examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)** |
| 实现侧车：请求/响应 JSON 长什么样 | **[../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**（协议全文，含示例） |
| `settings.json` 里 `plugin_backends` 每个字段含义 | **[../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** |
| **目录式进程插件**（`plugins/`、`manifest.json`、整壳、`directory_plugin_invoke`、开发者模式） | **[../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)** |
| **整壳 / 插槽 `invoke` 命令表、权限别名、错误码** | **[../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)** |
| **配置文件位置**（`plugin_state`、`ui.json`、`oclive_last_role_id`） | **[../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)** |
| **插件 FAQ（Vue 不显示、iframe 调试、依赖等）** | **[../FAQ.md](../FAQ.md)** |
| **编写器与主程序版本兼容** | **[../COMPATIBILITY.md](../COMPATIBILITY.md)** |
| **`memory = local`**、`_local_plugins` 清单与桥接契约 | **[../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)** |
| 在 Rust 里新增一种内置后端或注册方式 | **[../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)** |
| 只做角色包内容（manifest、场景、文案） | **[CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md)**（**`OCLIVE_ROLES_DIR`**、编写器分工、**应用内导入 zip/文件夹**）、[roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)、导入验收 [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **核心 / 可变性格档案、`personality_source`、七维视图** | **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)**（与 `roles/README_MANIFEST.md` §5.3 互参） |
| **设计思路为何从「七维为主」走到「档案轴心」** | **[docs/design-axis-evolution.md](../../docs/design-axis-evolution.md)**（旧文档保留，冲突以契约为准） |
| 编写器校验路线（与 `load_role` / crate 中期） | **[../role-pack/EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)** |
| 包版本、`schema_version`、世界观知识 `knowledge/` | **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)** · **[../role-pack/WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md) |
| 扩展点与源码文件 | **[../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)** |
| 愿景与路线图 | **[../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)** |
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

1. [../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)  
2. [../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)  
3. 源码：`src-tauri/src/domain/plugin_host.rs`、`src-tauri/src/infrastructure/remote_plugin/`、**`src-tauri/src/infrastructure/directory_plugins/`**（目录插件扫描与懒启动）  

---

## 与仓库根 README 的关系

项目总览、构建命令、测试见仓库根目录 **[README.md](../../README.md)**；**插件与侧车细节以 `creator-docs/` 为准**。
