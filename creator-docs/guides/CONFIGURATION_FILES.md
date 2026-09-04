# Oclive 配置文件说明

本文档说明 **Oclive 主程序（oclivenewnew）** 运行时会读写的常见配置文件：位置、用途与关键字段。路径以 **桌面端默认布局** 为准；开发时若使用自定义 `roles` 目录，请以实际 **`roles` 父目录** 与 **Tauri 应用数据目录** 为准。

**应用数据目录**（下文记为 **`{app_data}`**）：指运行时有效的 `OCLIVE_APP_DATA`；桌面 / shared kernel 默认使用品牌 canonical 目录（Windows 为 `%LOCALAPPDATA%/OCLive/data`），显式环境变量可覆盖。旧 Tauri `%APPDATA%/com.oclivenewnew.app` 只作为一次性迁移来源。`app.db`、用户插件和宿主插件配置都以该有效目录为根；完整解析顺序见 [OCLIVE_APP_DATA.md](../kernel/OCLIVE_APP_DATA.md)。

| 文件 | 路径 |
|------|------|
| SQLite 主库 | `{app_data}/app.db` |
| 插件 UI 状态（v2） | `{app_data}/plugin_state.json` |
| 宿主插件选项 | `{app_data}/oclive_host_plugins.json` |
| 上次切换的角色 ID | `{app_data}/oclive_last_role_id.txt` |
| 用户级插件包目录（扫描根之一） | `{app_data}/plugins/` |

**实现参考**：`kernel/crates/oclive_kernel_host/src/infrastructure/plugin_state.rs`、`kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/runtime/mod.rs`、`distros/desktop-tauri/src/lib.rs`（`app_data_dir` 解析）。

---

## 1. `plugin_state.json`

- **位置**：`{app_data}/plugin_state.json`
- **用途**：持久化用户对 **目录插件 UI** 的调整（按 **角色 ID** 隔离）：整壳选择、插槽内插件顺序、某插槽内隐藏某插件贡献、全局禁用列表、**强制 iframe 模式** 等。
- **格式**：JSON，`schema_version` 为 **`2`** 时使用 **`roles`** 映射；旧版全局块会迁移到 **`legacy_v1`**。

**关键字段（v2）**：

| 字段 | 说明 |
|------|------|
| `schema_version` | 固定 **`2`** 表示按角色存储 |
| `roles` | `role_id` → **`RolePluginState`**：含 `shell_plugin_id` 与 `slots`（扁平为 `PluginStateFile`） |
| `roles[...].slots.disabled_plugins` | 全局禁用的插件 id 列表 |
| `roles[...].slots.slot_order` | 如 `chat_toolbar` → 插件 id 顺序 |
| `roles[...].slots.disabled_slot_contributions` | 某插槽内不渲染的插件 id |
| `roles[...].slots.force_iframe_mode` | 为真时宿主 **忽略** manifest 的 **`vueComponent`**，插槽与整壳一律 iframe |
| `legacy_v1` | 仅迁移期保留的旧版全局状态 |

首次加载某角色时，若尚无记录，可由角色包 **`ui.json`** 生成初始状态（见 `RolePluginState::from_ui_config`）。

**用户操作入口**：主界面按 **`Ctrl+Shift+F`** 打开 **插件管理**，可修改上述状态（启用/停用、插槽排序、按插槽隐藏、重置为角色包推荐）。

---

## 2. `ui.json`（角色包）

- **位置**：角色包根目录，与 **`pipeline.ocblueprint`** 并列（见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)）。
- **用途**：创作者定义 **推荐前端布局**：整壳插件、各官方插槽的插件顺序与可见性、主题与布局等。
- **格式**：JSON，**机器可读 schema** 见 **[role-pack/ui.json.schema.json](../role-pack/ui.json.schema.json)**。

**字段概览**：

| 区域 | 说明 |
|------|------|
| `shell` | 推荐整壳插件 `manifest.id`（字符串） |
| `slots` | `chat_toolbar`、`settings_panel`、`role_detail`、`sidebar`、`chat_header` 等，每项含 `order`、`visible` |
| `theme` | 主题主色等（若 schema 中有定义） |
| `layout` | 布局相关（若 schema 中有定义） |

与后端配置分工：**`ui.json` 管前端展示与插件布局**；**`pipeline.ocblueprint.slot_registry` 管后端能力实例**。legacy `settings.json` 只用于 v1 迁移。详见下文 §5。

---

## 3. `oclive_last_role_id.txt`

- **位置**：`{app_data}/oclive_last_role_id.txt`
- **用途**：单行文本，记录 **上次成功切换/使用的角色 ID**，供 **`get_directory_plugin_bootstrap`** 等在 `role_id` 省略时解析 **当前角色上下文**（与 `plugin_state` 联动）。

---

## 4. `manifest.json`（目录插件）

- **位置**：每个插件包根目录下的 **`manifest.json`**（扫描根为 `<roles 父目录>/plugins/`、`./plugins/`、`{app_data}/plugins/` 等；本 monorepo 对应 `distros/chat-pro/plugins/`，见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1）。
- **用途**：声明插件 ID、版本、整壳、子进程、UI 插槽、bridge 白名单、依赖等。
- **详细规范**：见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §2；**版本号**须为宿主可解析的 **SemVer**（`load_from_dir` 校验）。

---

## 5. `pipeline.ocblueprint`（角色包核心配置）

- **位置**：角色包根目录；新包不得与 legacy `manifest.json` / `settings.json` 双轨并存。
- **用途**：`meta` 保存角色元数据、人格与场景，**`slot_registry`** 保存开放实例及其 `type`、`backend`、`plugin` / `plugins`、模型等（完整字段见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) 与 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)）。
- **与 `ui.json` 分工**：
  - **`pipeline.ocblueprint.slot_registry`**：**后端能力** — 例如某个 `type: memory` 实例使用 `backend: directory`，其 `plugin` 指向目录插件 **`manifest.id`**。
  - **`ui.json`**：**前端布局** — 哪些插件出现在工具栏/设置页等，以及 **`theme` / `layout`**（若使用）。

legacy `settings.json` 的 `plugin_backends.directory_plugins` 仍可由迁移工具读取，但不是 v2/v3/v4 新包的配置真源。

---

## 6. `adult_extension.json`（角色包可选 · Chat Pro）

- **位置**：角色包根目录，与 `pipeline.ocblueprint` 并列。
- **用途**：保存仅在成年确认、Chat Pro 全局开关和当前角色开关同时开启时加载的成人状态人设、对话指导、基础场景成人走向与节奏建议。
- **兼容性**：不是通用基础角色包必需文件；其他发行版可以忽略。基础人设、身份与场景必须在缺少该文件时独立运行。
- **校验**：`schema_version` 当前为 `1`，`character_is_adult` 必须为 `true`，场景键必须引用基础包场景。完整字段见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md#chat-pro-成人角色扩展adult_extensionjson--可选)。

---

## 7. `oclive_host_plugins.json`（可选）

- **位置**：`{app_data}/oclive_host_plugins.json`
- **用途**：开发者模式、额外插件扫描根、默认整壳插件 id 等（见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1 与 §「oclive_host_plugins.json」表）。

---

## 相关链接

- [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)
- [../getting-started/ERROR_CODES.md](../getting-started/ERROR_CODES.md)
