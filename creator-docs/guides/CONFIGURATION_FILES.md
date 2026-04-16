# Oclive 配置文件说明

本文档说明 **Oclive 主程序（oclivenewnew）** 运行时会读写的常见配置文件：位置、用途与关键字段。路径以 **桌面端默认布局** 为准；开发时若使用自定义 `roles` 目录，请以实际 **`roles` 父目录** 与 **Tauri 应用数据目录** 为准。

**应用数据目录**（下文记为 **`{app_data}`**）：由 Tauri `path_resolver().app_data_dir()` 解析（Windows 常见为 `%APPDATA%` 下应用标识目录），与 **`app.db`** 同级。目录插件相关文件均放在此目录（**不是** `app_data/oclive/` 子目录名）。

| 文件 | 路径 |
|------|------|
| SQLite 主库 | `{app_data}/app.db` |
| 插件 UI 状态（v2） | `{app_data}/plugin_state.json` |
| 宿主插件选项 | `{app_data}/oclive_host_plugins.json` |
| 上次切换的角色 ID | `{app_data}/oclive_last_role_id.txt` |
| 用户级插件包目录（扫描根之一） | `{app_data}/plugins/` |

**实现参考**：`src-tauri/src/infrastructure/plugin_state.rs`、`src-tauri/src/infrastructure/directory_plugins/runtime.rs`、`src-tauri/src/lib.rs`（`app_data_dir` 解析）。

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

---

## 2. `ui.json`（角色包）

- **位置**：角色包根目录，与 **`settings.json`**、**`manifest.json`** 并列（见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)）。
- **用途**：创作者定义 **推荐前端布局**：整壳插件、各官方插槽的插件顺序与可见性、主题与布局等。
- **格式**：JSON，**机器可读 schema** 见 **[role-pack/ui.json.schema.json](../role-pack/ui.json.schema.json)**。

**字段概览**：

| 区域 | 说明 |
|------|------|
| `shell` | 推荐整壳插件 `manifest.id`（字符串） |
| `slots` | `chat_toolbar`、`settings_panel`、`role_detail`、`sidebar`、`chat_header` 等，每项含 `order`、`visible` |
| `theme` | 主题主色等（若 schema 中有定义） |
| `layout` | 布局相关（若 schema 中有定义） |

与 **`settings.json`** 分工：**`ui.json` 管前端展示与插件布局**；**`settings.json` 管后端能力**（如 **`plugin_backends`**、`directory_plugins` 槽位）。详见下文 §5。

---

## 3. `oclive_last_role_id.txt`

- **位置**：`{app_data}/oclive_last_role_id.txt`
- **用途**：单行文本，记录 **上次成功切换/使用的角色 ID**，供 **`get_directory_plugin_bootstrap`** 等在 `role_id` 省略时解析 **当前角色上下文**（与 `plugin_state` 联动）。

---

## 4. `manifest.json`（目录插件）

- **位置**：每个插件包根目录下的 **`manifest.json`**（扫描根为 `<roles 父目录>/plugins/`、`./plugins/`、`{app_data}/plugins/` 等，见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1）。
- **用途**：声明插件 ID、版本、整壳、子进程、UI 插槽、bridge 白名单、依赖等。
- **详细规范**：见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §2；**版本号**须为宿主可解析的 **SemVer**（`load_from_dir` 校验）。

---

## 5. `settings.json`（角色包核心配置）

- **位置**：角色包根目录。
- **用途**：角色运行时行为：**场景、人格、插件后端枚举 `plugin_backends`**、Ollama/Remote 等（完整字段见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)）。
- **与 `ui.json` 分工**：
  - **`settings.json`**：**后端能力** — 例如 `plugin_backends.memory = "directory"` 与 **`directory_plugins`** 各槽位指向的 **`manifest.id`**。
  - **`ui.json`**：**前端布局** — 哪些插件出现在工具栏/设置页等，以及 **`theme` / `layout`**（若使用）。

---

## 6. `oclive_host_plugins.json`（可选）

- **位置**：`{app_data}/oclive_host_plugins.json`
- **用途**：开发者模式、额外插件扫描根、默认整壳插件 id 等（见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1 与 §「oclive_host_plugins.json」表）。

---

## 相关链接

- [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)
- [../getting-started/ERROR_CODES.md](../getting-started/ERROR_CODES.md)
