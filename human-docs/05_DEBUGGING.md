# 05 · 调试（不用 AI）

> **读者**：本地复现问题、要读日志或查库的工程师。  
> **读完能做什么**：配置 `RUST_LOG`、定位 `app.db`、用 `ProcessMessageError` 的 `stage` 缩小范围。  
> **耗时**：约 30 分钟。  
> **下一篇**：[06 内核学习路径](06_KERNEL_LEARNING_PATH.md)。

---

## RUST_LOG 配方

默认 `info`；由 [`init_tracing`](../crates/oclive_kernel_host/src/lib.rs) 初始化。设置环境变量 **`RUST_LOG`** 后重启应用。

**PowerShell 示例**：

```powershell
$env:RUST_LOG = "info,oclive_chat=debug,oclive_plugin=debug,oclive_llm=debug"
npm run tauri:dev
```

**bash 示例**：

```bash
RUST_LOG=info,oclive_chat=debug,oclive_plugin=debug npm run tauri:dev
```

含 `json` 子串时输出 JSON 行格式（见 `OCLIVE_LOG_FORMAT`）。

---

## tracing target 表（仓库内显式 `target:`）

在 `crates/` 与 `src-tauri/` 中检索 `target: "oclive_*"` 得到的主要 target：

| target | 典型场景 |
|--------|----------|
| **`oclive_chat`** | `process_message` 失败、回合编排错误 |
| **`oclive_plugin`** | 目录插件解析、directory 槽降级 |
| **`oclive_llm`** | LLM 调用、Ollama / Remote |
| **`oclive_deep_link`** | `oclive://` 深链安装 |
| **`oclive_hotkey`** | 全局快捷键注册 |
| **`oclive_desktop`** | 桌面宿主集成 |

**模块级过滤**：也可用 crate 路径，例如 `RUST_LOG=oclive_kernel_host::domain::chat_engine=debug`。

**检索命令**（自行更新表）：

```bash
rg 'target: "oclive' crates src-tauri
```

---

## 日志文件

| 变量 / 模式 | 效果 |
|-------------|------|
| **`OCLIVE_LOG_DIR`** | 同时写入 rolling 文件 |
| **`--api` 无头模式** | 默认 `temp/oclive_api_app_data/logs/` |

---

## app.db 与 SQLite

| 项 | 说明 |
|----|------|
| **路径** | `{app_data}/app.db`（Windows 常见 `%APPDATA%` 下应用标识目录） |
| **文档** | [CONFIGURATION_FILES.md](../creator-docs/guides/CONFIGURATION_FILES.md) |
| **打开** | [DB Browser for SQLite](https://sqlitebrowser.org/) 或 `sqlite3` CLI |
| **迁移 SSOT** | [`crates/oclive_kernel_host/migrations/`](../crates/oclive_kernel_host/migrations/) |

常用表：`role_runtime`（按 **`srid`** 键）、`chat_messages`、`long_term_memory`（与聊天存储解耦）。

---

## ProcessMessageError 与 stage

错误形如 `send_message[{stage}]: …`。`stage` 字符串用于定位失败阶段，例如：

- `ensure_role_loaded`
- `startup_health`
- `dual_core_experimental`（仅 `dual_core` feature）

定义：[`message_error.rs`](../crates/oclive_kernel_host/src/domain/chat_engine/message_error.rs)

日志检索：`oclive_chat` target + 错误全文中的 `stage` 名。

---

## 跳过探针（本地加速）

| 变量 | 作用 |
|------|------|
| `OCLIVE_SKIP_STARTUP_HEALTH` | 跳过首轮健康检查 |
| `OCLIVE_SKIP_LLM_STARTUP_PROBE` | 跳过 LLM 启动探测 |
| `OCLIVE_HTTP_API_MOCK_LLM=1` | HTTP 烟测 mock LLM |

---

## 验收

- [ ] 能用 `RUST_LOG` 只看 `oclive_chat` debug
- [ ] 知道 `app.db` 在 `{app_data}` 而非仓库内
- [ ] 看到 `send_message[ensure_role_loaded]` 知道查角色加载

---

## 深度链接

- [ERROR_CODES](../creator-docs/getting-started/ERROR_CODES.md)
- [USER_MANUAL §排错](../creator-docs/getting-started/USER_MANUAL.md)
