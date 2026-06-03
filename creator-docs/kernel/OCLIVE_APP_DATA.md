# OCLIVE_APP_DATA — 跨宿主数据目录

**读者**：VS Code 扩展、启动器、`oclive-kernel-server`、CI 维护者。

---

## 品牌路径（canonical）

| 平台 | 默认 `OCLive/data` |
|------|---------------------|
| Windows | `%LOCALAPPDATA%/OCLive/data` |
| macOS | `~/Library/Application Support/OCLive/data` |
| Linux | `$XDG_DATA_HOME/OCLive/data` 或 `~/.local/share/OCLive/data` |

与 `%LOCALAPPDATA%/OCLive/runtime`（共享内核二进制）**并列**；数据与 runtime 分离。

SQLite 真源：`{OCLIVE_APP_DATA}/app.db`（或通过 `resolve_db_path`）。

---

## 环境变量

| 变量 | 语义 |
|------|------|
| `OCLIVE_APP_DATA` | **显式** app data 根；spawn / 桌面优先 |
| `OCLIVE_USE_CANONICAL_APP_DATA=1` | headless `--api` 使用品牌目录（未设 `OCLIVE_APP_DATA` 时） |
| `OCLIVE_API_USE_TEMP_APP_DATA=1` | 强制 temp 库（**CI / OOCP 默认**） |
| `OCLIVE_SKIP_APP_DATA_MIGRATION=1` | 跳过 Tauri 旧目录 → canonical 一次性复制（测试） |
| `OCLIVE_ATTACH_REMOTE_KERNEL=1` | 桌面强制 attach `:8420`（不打开本地写库） |
| `OCLIVE_FORCE_LOCAL_KERNEL=1` | 桌面忽略已有 `:8420`，始终本地 canonical |

---

## headless `--api` 解析顺序

1. `OCLIVE_APP_DATA` 非空 → 持久化  
2. `OCLIVE_API_USE_TEMP_APP_DATA=1` → temp（退出时删除）  
3. `OCLIVE_USE_CANONICAL_APP_DATA=1` → 品牌目录  
4. 否则 → temp（与历史 CI 行为一致）

---

## 一次性迁移

当 canonical `app.db` **不存在**，且 Tauri 旧路径存在 `app.db` 时：

- **复制**（非移动）整棵旧 `app_data` 到 `OCLive/data`  
- 写入 `.migrated_from_tauri` 标记  
- 失败则 **不** 以写者打开库  

旧路径（`identifier: com.oclivenewnew.app`）：

- Windows: `%APPDATA%/com.oclivenewnew.app`  
- macOS: `~/Library/Application Support/com.oclivenewnew.app`  
- Linux: `~/.local/share/com.oclivenewnew.app`

CLI：`cargo run -p oclive-cli -- migrate-app-data [--target PATH] [--dry-run]`

---

## 单写者

同一 `app.db` 仅一个进程以写者打开；其它发行版 **attach** `GET http://127.0.0.1:8420/health`。

桌面 Phase 2：同进程绑定 `:8420` + canonical 数据；若端口已被占用则 attach HTTP 模式（`send_message` 走 `POST /chat`）。

---

## 相关文档

- [`CROSS_HOST_MEMORY.md`](../role-pack/CROSS_HOST_MEMORY.md)  
- [`VSCODE_DISTRIBUTION.md`](../role-pack/VSCODE_DISTRIBUTION.md)  
- [`handoff/CHAT_STORAGE_ARCHITECTURE.md`](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)
