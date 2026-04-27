## Module 8：Frontend Shell（前端壳 / 发行版 UI 模块）

> 目标：把 “前端 UI” 视为与后端 1–7 模块**并列**的可替换组件，使 oclive 更贴近 Linux 的设计哲学：Kernel 提供稳定能力面（ABI），Shell 作为用户态发行版可替换。

---

### 1. 定义

- **Module 8（Frontend Shell）**：负责 UI/交互/渲染/窗口与快捷键等发行版体验；不实现核心业务规则与引擎策略。
- **Kernel（Modules 1–7 + 基础设施）**：负责状态机、编排、权限与审计、持久化、协议（OOCP / 插件 RPC）。

---

### 2. Shell 与 Kernel 的边界

- **Shell 做**
  - 视图与交互：Vue 组件、页面、主题、布局、快捷键说明等。
  - 发行版体验：插件管理窗口、市场 UI、设置页呈现（但不写业务规则）。
  - UI 扩展点：插槽渲染、整壳替换、插件 WebView/iframe 承载。

- **Kernel 做**
  - 对话闭环与状态：`process_message`、session、scene、time。
  - 后端模块路由：`plugin_backends` 选择 builtin/remote/directory/local。
  - 权限与审计：`plugin_permission_grants`、`plugin_audit_log`。
  - 插件协议面：Directory Plugins 懒启动、Remote JSON-RPC、MCP tools、Agent（第七模块）。

---

### 3. 前端插槽（UI Slots）如何与后端插件能力对接

核心原则：**插槽是 Shell 的能力集（UI 能力），插件是 Kernel 的能力扩展（后端/资产/桥接能力）**。

#### 3.1 插槽的“单一事实来源”

- 插槽名字属于 **Shell 契约**（官方语义插槽名）。
- Kernel 通过 `get_directory_plugin_bootstrap.supportedUiSlots` 暴露“本发行版 Shell 支持哪些插槽”，用于：
  - 不同发行版之间的兼容/降级（例如 VSCode 发行版可能不支持 `overlay.floating`）。
  - 插件作者在运行时按能力渲染/隐藏 UI（避免硬崩）。

#### 3.2 插件如何贡献 UI

目录插件在 `manifest.json` 的 `ui_slots[]` 中声明插槽贡献（可选 `vueComponent`，失败回退 iframe）。Shell 通过：

- `get_directory_plugin_bootstrap.uiSlots` 获取当前角色下的插槽贡献列表
- 结合 `plugin_state.slot_order` / `disabled_slot_contributions` 做排序与启停
- 在对应 UI 区域渲染这些 slot（Vue 原生组件或 iframe）

详见：[DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) 的 “主界面 UI 插槽（ui_slots）”。

#### 3.3 插槽页调用宿主能力（桥接）

插槽页不应直接访问敏感 API；统一通过 **`plugin_bridge_invoke`**（并由权限系统强制）：

- 插槽页/整壳页在 manifest 中声明 `bridge.invoke` / `bridge.events`
- Kernel 依据 `plugin_permission_grants` 决定允许/拒绝，并写审计元数据

---

### 4. 与第七模块（Agent/Skill）的关系

- **Module 7（Agent）** 是后端可插拔模块：`plugin_backends.agent = builtin|remote|directory`
- Skill 可以是：
  - MCP tools（工具服务器），由 Agent 调用
  - 或 Agent 插件内部能力，对外仅暴露 `agent.process`

---

### 5. 兼容策略（发行版之间）

- **Shell 只应依赖契约**：
  - DTO（以 `src-tauri/src/models/dto.rs` 为准）
  - `get_directory_plugin_bootstrap`（包含 `supportedUiSlots`）
  - OOCP capabilities（若走 OOCP）
- **新增插槽**：应视为 Shell 的新能力，需同步：
  - Shell 渲染实现
  - `supportedUiSlots` 输出
  - 文档（本文件 + DIRECTORY_PLUGINS）

