# 错误码与排障速查

面向用户与开发者的统一错误语义。目标：**先自助定位，再高质量提 issue**。

## 1) 运行时 HTTP API（`/chat`）错误体（与内核 / Tauri 同源）

`POST /chat` 失败时返回 JSON，**`error` 与 `oclive_kernel_runtime::KernelErrorBody` 同形**（与 Tauri `invoke` 失败载荷为**同一 JSON 单行**时可互解析）：

- `code`：**`SCREAMING_SNAKE_CASE`** 机器码，与内核 [`AppError::code`](../../crates/oclive_kernel_runtime/src/error.rs) 一致。
- `message`：内核 `Display`（默认英文技术句）；本地化由发行版用 `code` 映射。
- `hint`：可选「下一步」；HTTP 可为试聊附加中文提示。

返回体（示例）：

```json
{
  "error": {
    "code": "INVALID_ROLE_PATH",
    "message": "role_path is not a directory: D:\\roles\\demo",
    "hint": "请传入包含 manifest.json 的角色目录绝对路径"
  }
}
```

| code | 含义 | 常见原因 | 建议 |
|------|------|----------|------|
| `EMPTY_MESSAGE` | 消息为空 | 输入只有空格/换行 | 输入至少 1 个可见字符 |
| `INVALID_ROLE_PATH` | 角色路径不是目录 | 路径拼错、指到了文件 | 传入 `{roles_root}/{role_id}` 目录绝对路径 |
| `ROLE_NOT_FOUND` 等 | 目录存在但包无效 | `manifest/settings` 缺失或结构错误 | 用编写器“运行全部检查”；`code` 与 Tauri `load_role` 一致 |
| `LLM_ERROR`、`DB_ERROR`、`TXN_*` 等 | 对话引擎内失败 | 模型、DB、事务等 | 查看 `oclive_chat` / `oclive_plugin`；与桌面同一码表 |
| `LOAD_ROLE_TASK_PANIC` | 加载任务 panic | 极少见 | 带日志提 issue |

### 1.5) 首装常见：Ollama 与角色目录（A2.1 子集）

| 现象 | 常见原因 | 建议下一步 |
|------|----------|------------|
| 对话失败、日志或 UI 提示无法连接 **Ollama** | 本机未安装、服务未启动、端口非默认、模型未 `pull` | 安装并启动 [Ollama](https://ollama.com)；终端执行 `ollama list` / `ollama pull <模型>`；核对角色包或环境变量中的模型名 |
| **`INVALID_ROLE_PATH` / `ROLE_NOT_FOUND` 等** | **`OCLIVE_ROLES_DIR`** 未指向含子目录的 roles 根，或子目录缺 `manifest.json` | 将变量设为 **各角色文件夹的父目录**；用 [启动器](https://github.com/linkaiheng2233-cyber/oclive-launcher) 一键配置或对照 [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |
| **目录不可写 / 权限**（系统弹窗或 Rust I/O 错误） | 杀毒拦截、用户目录权限、指向了只读介质 | 换可写路径或排除误拦；勿在只读共享盘上放 `app.db`（路径见 [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)） |
| **设置页「环境自检」** | 需快速确认本机 Ollama / 角色根 / 数据目录 | 应用内 **Ctrl+Shift+S** 打开设置 → **常规** → **环境自检** → **运行检测**（对应 Tauri `run_environment_diagnostics`） |

### 1.6) 离线 / 弱网（A2.3）

| 场景 | 运行时行为 | 建议 |
|------|------------|------|
| **社区插件索引**（插件工作台 → 社区索引 →「同步在线索引」） | 在线 `plugins.json` 失败时自动读 `app_data` 下 **`plugin_index_cache.json`**，返回 `offlineMode=true` 与 `warning`（技术原因）；界面与 Toast 走 i18n 说明 | 检查网络、代理、防火墙；可设 **`OCLIVE_PLUGIN_INDEX_URL`** 指向可访问的索引镜像；联网后再次同步 |
| **首次从未同步成功** | 缓存可能为空，列表无条目 | 至少成功同步一次，或使用「从文件夹 / zip 安装」等离线路径 |
| **Ollama / Remote LLM** | 超时或不可达时由对话路径返回带 `[CODE]` 的错误 | 见 **§1.5** 与前端 `apiErrors` 映射 |
| **Tauri `[CODE]` 常见补充** | 首轮对话前自检失败、未先加载角色 | `[STARTUP_HEALTH_FAILED]`：manifest / 槽位 / DB；`[ROLE_RUNTIME_NOT_READY]`：请先 `load_role` 或在 UI 选择角色；`[PLUGIN_BACKENDS_DIRECTORY_SLOT]`：directory 槽未填插件 id |

GUI 侧若仍展示英文底层错误句，属于 **A3.2 / A6** 持续扫尾；发版前可先依赖上述文档自助排障。

---

## 2) Remote JSON-RPC 错误码（侧车建议）

宿主会记录 `code/message/data`，并在失败时回退内置实现。推荐约定：

| code | name | 语义 |
|------|------|------|
| `-32700` | `parse_error` | 请求体不是合法 JSON |
| `-32600` | `invalid_request` | JSON-RPC 包结构错误 |
| `-32601` | `method_not_found` | 方法不存在 |
| `-32602` | `invalid_params` | 参数缺失或类型不匹配 |
| `-32603` | `internal_error` | 侧车内部错误 |
| `-32010` | `plugin_timeout` | 侧车上游调用超时 |
| `-32011` | `auth_failed` | token 无效或权限不足 |
| `-32012` | `rate_limited` | 命中限流 |
| `-32013` | `upstream_unavailable` | 上游服务不可用 |

---

## 3) 提 issue 最少信息（建议）

1. `error.code`、`error.message`、`error.hint`（若有）  
2. 触发动作（检测 API / 发送消息 / 自动启动）  
3. 环境变量是否设置（仅变量名，不贴密钥值）：  
   - `OCLIVE_LLM_BACKEND`（`ollama` / `remote`；由启动器注入时可覆盖角色包 `plugin_backends.llm`）
   - `OCLIVE_REMOTE_PLUGIN_URL`
   - `OCLIVE_REMOTE_LLM_URL`
   - `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS`
   - `OCLIVE_REMOTE_LLM_TIMEOUT_MS`
   - `OCLIVE_PLUGIN_INDEX_URL`（社区插件 `plugins.json` 镜像；离线排障见 **§1.6**）
4. 关键日志片段（`oclive_chat` / `oclive_plugin`）

---

[English](../../creator-docs-en/getting-started/ERROR_CODES.md)