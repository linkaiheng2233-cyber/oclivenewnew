# Tauri v1 → v2 迁移清单（K-PLATFORM-01a **Full**）

**状态：** **01a Full 完成**（Tauri 2 bump · capability ACL · Rust API · 单命令 invoke smoke）  
**债项：** [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) · 父 **K-PLATFORM-01 OPEN** · 子 **01a Done** · **01b** / **01c** 另波  
**权威配置：** [`distros/desktop-tauri/tauri.conf.json`](../../distros/desktop-tauri/tauri.conf.json) · [`capabilities/`](../../distros/desktop-tauri/capabilities/) · [`Cargo.toml`](../../distros/desktop-tauri/Cargo.toml) · [`src/lib.rs`](../../distros/desktop-tauri/src/lib.rs)  
**上游迁移指南：** [Upgrade from Tauri 1.0](https://v2.tauri.app/start/migrate/from-tauri-1/)

> **本波已做：** Rust `tauri` 2.x + plugins · ACL（窄权限 · 无 remote `*`）· invoke smoke · npm **最小齐步**（`@tauri-apps/api` v2 + dialog/shell/opener 插件 import）。  
> **npm 最小齐步 ≠ 01b Done**（chat-pro 发消息 E2E 迁移表收口另波）。  
> **01c：** workflow/dimension5 全量 v2 门禁叙事另波（本波硬门禁以 `cargo build -p oclivenewnew-tauri` + smoke + audit 为准）。

---

## 1. 版本清单（Cargo.lock / package · 2026-07-14 · Full）

| 组件 | 约束（manifest） | 锁 / 解析版本 |
|------|------------------|---------------|
| `tauri` | `2`（feature `protocol-asset`） | **2.11.x** |
| `tauri-build` | `2` | **2.6.x** |
| `tauri-plugin-deep-link` | `2`（feature `desktop`） | **2.4.x** |
| `tauri-plugin-shell` / `dialog` / `opener` / `global-shortcut` | `2` | 见 lock |
| `@tauri-apps/api` | `^2.11`（根 · shared · chat-pro · theater） | 锁随 `package-lock` |
| `@tauri-apps/cli` | `^2.11`（根） | **2.11.x** |
| `@tauri-apps/plugin-dialog` / `shell` / `opener` | `^2` | npm |

**与供应链债的接点：** [`KNOWN_VULNERABILITIES.md`](../../creator-docs/security/KNOWN_VULNERABILITIES.md)

- **RUSTSEC-2025-0057** `fxhash` · **RUSTSEC-2026-0097** `rand` 0.7 — **已随 Tauri 2 清出锁图**
- gtk-rs GTK3 / `glib`（Linux WebView via wry）— 仍警告级跟踪；audit ignore 簇保留

---

## 2. allowlist / features → 已落地 v2 capability

路径：`distros/desktop-tauri/capabilities/` + `tauri.conf` `app.security`。

| v1 来源 | v2 落地 | 备注 |
|---------|---------|------|
| `allowlist.protocol.asset` | `app.security.assetProtocol.enable` + `scope: ["$RESOURCE/**"]` | |
| `allowlist.shell.open` | `tauri-plugin-opener`（Rust path open）+ `opener:allow-open-*`；JS `@tauri-apps/plugin-opener`；`shell:allow-open` 保留给 shell 插件 | |
| `allowlist.dialog.*` | `tauri-plugin-dialog` + `dialog:allow-open` / `save` / `confirm` / `message` | |
| `allowlist.window.*` | `core:window:allow-*`（minimize/maximize/unmaximize/close/start-dragging/set-decorations） | BOUNDARIES §6 |
| `global-shortcut` | `tauri-plugin-global-shortcut` + register/unregister 权限 | |
| deep-link `oclive://` | `tauri-plugin-deep-link` v2 + `plugins.deep-link.desktop.schemes` + `deep-link:default` | |
| `ocliveplugin` 自定义协议 | `register_uri_scheme_protocol`（v2 `UriSchemeContext`） | |
| `dangerousRemoteDomainIpcAccess` `plugins: ["*"]` | **`capabilities/plugin-shell-remote.json`**：`remote.urls` = `https://ocliveplugin.localhost/**` + **窄** `core:default` / `core:event:default`（**禁止** `*`） | |

主窗 ACL：[`capabilities/main.json`](../../distros/desktop-tauri/capabilities/main.json)。

**CSP（未弱化）：**  
`default-src 'self' tauri: https://ocliveplugin.localhost; connect-src 'self' tauri: https://ocliveplugin.localhost http://127.0.0.1:50000 ws://127.0.0.1:50000`  
窗体 `useHttpsScheme: true` 对齐既有 `https`/`tauri:` CSP。

---

## 3. Smoke

| 项 | 路径 / 命令 | 说明 |
|----|-------------|------|
| 进程内编排烟测（非 IPC） | [`tests/tauri_api_smoke.rs`](../../distros/desktop-tauri/tests/tauri_api_smoke.rs) | 既有 `process_message` |
| **01a 单命令 invoke smoke** | [`tests/tauri_invoke_smoke.rs`](../../distros/desktop-tauri/tests/tauri_invoke_smoke.rs) · `cargo test -p oclivenewnew-tauri --test tauri_invoke_smoke` | `list_roles_impl` + Tauri 2 `CommandError` IPC 序列化契约；Windows 上 `tauri::test` mock webview 曾 `STATUS_ENTRYPOINT_NOT_FOUND`，故用 invoke-shaped host smoke |

---

## 4. 改动面 / blast radius

| 面 | 说明 |
|----|------|
| Tauri 命令 | `generate_handler!` 仍约 **122** 条；ACL 绑定 `main` |
| 前端 | 最小齐步：`api/core` · `webviewWindow` · plugin dialog/opener；**完整迁移表 + E2E = 01b** |
| CI | 本波 **不以 01c 改门禁叙事**；仍跑既有 `cargo build -p oclivenewnew-tauri` / dimension5 / audit |

---

## 5. 子项状态

| 子项 | 范围 | 状态 |
|------|------|------|
| **01a Partial** | inventory 对照（历史） | 被 Full 取代 |
| **01a Full** | bump + ACL + smoke + audit | **本波 Done**（见 TECHNICAL_DEBT 验证行） |
| **01b** | 前端 IPC 迁移表 + chat-pro E2E | **OPEN**（下波） |
| **01c** | workflow + dimension5 切 v2 口径 | **OPEN** |

父 **K-PLATFORM-01** 在 01b–01c 未齐前 **保持 OPEN**；**勿**标父 Done。
