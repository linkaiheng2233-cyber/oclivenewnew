# 目录式进程插件（Directory Plugins）— 架构与契约

本文描述用户选型 **A1–C1** 下**当前实现**：`plugins/` 扫描、`manifest.json`、子进程 JSON-RPC、**整壳 UI**（`https://ocliveplugin.localhost/…`）、**统一门面命令** `directory_plugin_invoke`（等价于「动态 Tauri 命令」），以及**开发者模式**从额外根目录加载。

**Wire 格式**：与现有 Remote 侧车一致（HTTP POST JSON-RPC 2.0、请求头 `x-oclive-remote-protocol` 等），见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。

**与 `plugin_backends` 的关系**：各模块枚举值可为 **`directory`**；同包（或会话覆盖）内嵌对象 **`directory_plugins`** 为各槽位指定 **`manifest.id`**（见下节）。就绪行：子进程 stdout 打印 **`{ready_prefix} {rpc_url}`**（默认前缀 `OCLIVE_READY`，与一行 URL，空格分隔）。

---

## 1. 目录布局与扫描顺序

宿主合并以下**存在的**扫描根，每个根下的一级子目录若含 `manifest.json` 则视为一个插件包（以 manifest 内 `id` 注册；重复 `id` 时后扫描到的根覆盖并打日志）：

1. **`<roles 父目录>/plugins/`**（与 `roles/` 同级；开发时常为仓库根下 `plugins/`）
2. **`./plugins/`**（相对进程当前工作目录）
3. **`{app_data}/plugins/`**（与 `app.db` 同级的应用数据目录下的 `plugins/`）

**开发者模式（C1）**：当 `app_data/oclive_host_plugins.json` 中 **`developer_mode`: true**，或环境变量 **`OCLIVE_DEVELOPER=1`**（`true`/`yes` 亦可）时，额外扫描 **`extra_plugin_roots`** 中每一项（须为已存在目录）；行为同上。

### `oclive_host_plugins.json`（可选，位于应用数据目录根）

| 字段 | 类型 | 说明 |
|------|------|------|
| `developer_mode` | `boolean?` | 为真时启用 `extra_plugin_roots` |
| `extra_plugin_roots` | `string[]?` | 额外插件容器目录（其下一级子目录为插件根） |
| `shell_plugin_id` | `string?` | 指定用于整壳替换的插件 `manifest.id` |

环境变量 **`OCLIVE_SHELL_PLUGIN_ID`**（非空 trim）优先于文件中的 `shell_plugin_id`。

---

## 2. `manifest.json`（插件根目录）

| 字段 | 类型 | 说明 |
|------|------|------|
| `schema_version` | `number` | 当前仅接受 **`1`** |
| `id` | `string` | 全局唯一；与 `directory_plugins.*` 槽位对应 |
| `version` | `string` | 建议 SemVer 文本 |
| `shell` | `object?` | **`entry`**：相对插件根的 HTML 入口（整壳 B1） |
| `process` | `object?` | **`command`** / **`args[]`** / **`cwd?`**（`cwd` 相对插件根，可省略则默认为插件根） |
| `ready_prefix` | `string?` | 默认 **`OCLIVE_READY`**；就绪行 = 此前缀 + 空格 + **JSON-RPC 根 URL**（须 `http://` 或 `https://`） |

**懒启动**：首次需要该插件的 RPC（五模块 `directory`、`directory_plugin_invoke`、或需解析 shell manifest）时启动子进程，并缓存 **RPC URL** 与 **子进程**（当前实现不随角色切换回收子进程；应用退出时释放）。并发多次触发同一 `id` 时，宿主对单次启动加锁，避免重复子进程。

---

## 3. 后端五模块（A2）

在 `settings.json`（或等价磁盘设置）的 `plugin_backends` 中：

- `memory` / `emotion` / `event` / `prompt` 为 **`directory`** 时，使用 **`directory_plugins.<slot>`** 中的插件 `id` 懒启动后，对该 URL 走与 env-remote 相同的 HTTP 客户端（方法名分别为 `memory.rank` 等）。
- `llm` 为 **`directory`** 时，使用 **`directory_plugins.llm`** 指向的插件 URL，须实现 **`llm.generate` / `llm.generate_tag`**（超时默认按 LLM 档读取，见环境变量）。

若对应槽位 **id 缺失**、**运行时未注入目录插件**、**spawn 或握手失败**，宿主记日志并回退：**memory/emotion/event/prompt → builtin**，**llm → Ollama**。

### `plugin_backends` 与 `directory_plugins` 示例（节选）

```json
{
  "plugin_backends": {
    "memory": "directory",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "directory",
    "directory_plugins": {
      "memory": "com.example.myplugin",
      "llm": "com.example.myplugin"
    }
  }
}
```

**`directory_plugins` 槽位来源**：以角色包 **`settings.json` → `plugin_backends.directory_plugins`** 为准。`PluginBackendsOverride` 在 Rust 中**支持**按槽合并 `directory_plugins`（见 `apply_to`），但当前 Tauri 命令 **`set_session_plugin_backend` 仅覆盖五模块枚举与 `local_memory_provider_id`**，**不**传入 `directory_plugins`；多会话场景下若需不同目录插件 id，请通过角色包或后续扩展的会话 API 提供。

---

## 4. 整壳 UI（B1）

当 **`shell_plugin_id`**（文件或 `OCLIVE_SHELL_PLUGIN_ID`）指向已扫描到的插件，且其 manifest 含 **`shell.entry`** 时，内置前端在挂载前调用 **`get_directory_plugin_bootstrap`**；若返回 **`shellUrl`**，则对主窗口执行 **`location.replace(shellUrl)`**。

**`shellUrl` 形态**：`https://ocliveplugin.localhost/<manifest.id>/<entry>`（Windows WebView2 下由 Tauri 将自定义协议映射为该 HTTPS 主机名）。

**静态资源**：由宿主 **`register_uri_scheme_protocol("ocliveplugin", …)`** 从磁盘插件根读取（路径穿越会 403）。需在 **`tauri.conf.json`** 中配置 **`dangerousRemoteDomainIpcAccess`**，使该来源页面可调用 Tauri IPC（与内置 `invoke` 一致）。条目内需 **`enableTauriAPI`: true**（字段名随 Tauri 版本以 schema 为准；勿使用无效的 `enable`）。

```json
"dangerousRemoteDomainIpcAccess": [
  {
    "domain": "https://ocliveplugin.localhost",
    "windows": ["main"],
    "enableTauriAPI": true,
    "plugins": ["*"]
  }
]
```

---

## 5. 门面命令（B2）

运行时无法向 `generate_handler!` 动态注册符号；采用**固定命令名** + 插件侧方法名：

| Tauri 命令 | 作用 |
|------------|------|
| **`get_directory_plugin_bootstrap`** | 返回 `shellUrl`、`shellPluginId`、`pluginIds`、`developerMode`（字段名为 camelCase JSON） |
| **`directory_plugin_invoke`** | 懒启动目标插件后，向其 RPC URL 发送一次 JSON-RPC **`method`** / **`params`** |

**前端 `invoke` 载荷**（与仓库其它命令一致，单结构体参数包在 **`req`** 下）：

```json
{
  "req": {
    "pluginId": "com.example.myplugin",
    "method": "my.extension",
    "params": {}
  }
}
```

**环境变量（可选）**

| 变量 | 说明 |
|------|------|
| `OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS` | 非 LLM 类目录 RPC 调用超时（毫秒），默认 `8000` |
| `OCLIVE_DIRECTORY_LLM_TIMEOUT_MS` | `RemoteLlmHttp` 使用目录 URL 时的超时，默认 `120000` |
| `OCLIVE_DIRECTORY_PLUGIN_TOKEN` | 可选 Bearer，写入 `Authorization` |

---

## 6. 开发者模式（C1）小结

- **`developer_mode`** 或 **`OCLIVE_DEVELOPER=1`**：`extra_plugin_roots` 参与扫描。
- 未开启时忽略 `extra_plugin_roots`，降低误加载任意路径的风险。

---

## 7. 源码索引（实现）

| 区域 | 路径 |
|------|------|
| 扫描 / manifest / 懒启动 / shell URL | `src-tauri/src/infrastructure/directory_plugins/` |
| 枚举与 `directory_plugins` 槽位 | `src-tauri/src/models/plugin_backends.rs` |
| 五模块解析与 HTTP 复用 | `src-tauri/src/domain/plugin_host.rs`、`src-tauri/src/infrastructure/remote_plugin/` |
| Tauri 命令 | `src-tauri/src/api/directory_plugin.rs` |
| 自定义协议 + 启动 | `src-tauri/src/lib.rs` |
| 内置 UI 启动引导 | `src/main.js` |
| 前端封装 | `src/utils/tauri-api.ts`（`getDirectoryPluginBootstrap`、`directoryPluginInvoke`） |

---

## 8. 仓库内最小示例

见 **`examples/directory-plugin-minimal/`**：可复制到 `plugins/<id>/` 或加入 `extra_plugin_roots` 后，配置 `shell_plugin_id` 与（可选）`plugin_backends` 做联调。

---

## 9. 排错（常见问题）

| 现象 | 可能原因 |
|------|----------|
| 仍走 builtin / Ollama，日志提示 directory 缺失槽位 | `plugin_backends.* = directory` 但 **`directory_plugins.<槽>`** 未填或与 manifest **`id`** 不一致 |
| 整壳未跳转 | **`shell_plugin_id`** 未设或插件未扫描到；manifest 缺 **`shell.entry`**；`get_directory_plugin_bootstrap` 返回的 **`shellUrl`** 为空 |
| 整壳页里 **`invoke` 失败** | **`tauri.conf.json`** 未为 **`https://ocliveplugin.localhost`** 配置 **`dangerousRemoteDomainIpcAccess`**；或页面未在 Tauri WebView 内打开 |
| 子进程启动失败 / 无 RPC | **`process.command`** 在 PATH 中不可用（如未装 Node）；**`manifest.json`** 语法错误；子进程未在超时内向 stdout 打印 **`OCLIVE_READY <url>`** |
| **`directory_plugin_invoke`** 报错 | **`pluginId`** 未扫描到；目标插件缺 **`process`** 节无法懒启动 RPC |

过滤宿主日志 target：**`oclive_plugin`**。
