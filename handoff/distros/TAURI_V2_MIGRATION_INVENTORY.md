# Tauri v1 → v2 迁移清单（K-PLATFORM-01a **Full** · **01b** 前端 IPC · **01c** 门禁叙事）

**状态：** **01a/01b/01c 完成** · 父 **K-PLATFORM-01 Done**  
**债项：** [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) · 父 **Done** · 子 **01a Done** · **01b Done** · **01c Done**  
**权威配置：** [`distros/desktop-tauri/tauri.conf.json`](../../distros/desktop-tauri/tauri.conf.json) · [`capabilities/`](../../distros/desktop-tauri/capabilities/) · [`Cargo.toml`](../../distros/desktop-tauri/Cargo.toml) · [`src/lib.rs`](../../distros/desktop-tauri/src/lib.rs)  
**上游迁移指南：** [Upgrade from Tauri 1.0](https://v2.tauri.app/start/migrate/from-tauri-1/)

> **01a 已做：** Rust `tauri` 2.x + plugins · ACL（窄权限 · 无 remote `*`）· invoke smoke · npm **最小齐步** · CI apt `webkit2gtk-4.1`。  
> **01b 已做：** §6 Frontend IPC 迁移表 · `helpers`/`role` 走 `@tauri-apps/api/core` · 生产 TS/Vue **零** `@tauri-apps/api/{tauri,dialog,shell,window,fs}` · preview E2E [`send-message.spec.ts`](../../distros/chat-pro/e2e/send-message.spec.ts)（证据面 = `frontend` job / `test:e2e:preview`，**非** soft `e2e-tauri`）。  
> **01c 已做：** CONTRIBUTING / Windows setup 叙事对齐 Tauri **2** · `libwebkit2gtk-4.1` · dimension5 轻断言 `tauri` major **2** + 活跃路径不得再写 `webkit2gtk-4.0`。

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
| 前端 | 生产已 v2（§6）；preview E2E 经 `e2e-mock` alias；原生壳发消息 E2E **非** 01b 硬门禁 |
| CI | apt 已是 `webkit2gtk-4.1`；01c = 人类/dimension5 叙事对齐；01b 证据面 = `frontend`（含 `test:e2e:preview` / send-message） |

---

## 5. 子项状态

| 子项 | 范围 | 状态 |
|------|------|------|
| **01a Partial** | inventory 对照（历史） | 被 Full 取代 |
| **01a Full** | bump + ACL + smoke + audit | **Done**（见 TECHNICAL_DEBT 验证行） |
| **01b** | 前端 IPC 迁移表 + chat-pro preview 发消息 E2E | **Done**（§6 · TECHNICAL_DEBT 验证行） |
| **01c** | 人类/门禁叙事 + dimension5 v2 口径 | **Done** |

父 **K-PLATFORM-01**：**Done**（01a+01b+01c）。

---

## 6. Frontend IPC 迁移表（01b）

生产路径（`distros/shared` · `distros/chat-pro` · `distros/theater` 的 `.ts`/`.vue`）grep：  
`@tauri-apps/api/(tauri|dialog|shell|window|fs)` → **生产残留 = 0**。

| v1 | v2 / 替代 | 落点 |
|----|-----------|------|
| `@tauri-apps/api/tauri` `invoke` | `@tauri-apps/api/core` | [`helpers.ts`](../../distros/shared/src/api/helpers.ts)（SSOT）· 少量 direct 如 [`role.ts`](../../distros/shared/src/api/role.ts) · [`directoryShellBootstrap.ts`](../../distros/shared/src/utils/directoryShellBootstrap.ts) |
| `@tauri-apps/api/dialog` | `@tauri-apps/plugin-dialog` | shared composables / components（`useRolePackImport` · `useChatStorageSettings` · `RolePackBar` · `ChatExportBar` · `ModelManagerBody` · `SimplePluginManager` · `AsyncPluginVue`） |
| shell open | `@tauri-apps/plugin-opener`（+ Rust shell/opener 插件） | [`openPackEditor.ts`](../../distros/shared/src/utils/openPackEditor.ts)（`openPath` / `openUrl`） |
| `@tauri-apps/api/window` / `appWindow` | `webviewWindow` `getCurrentWebviewWindow` | [`useEasterEggSkin.ts`](../../distros/shared/src/composables/useEasterEggSkin.ts) · [`Win98TitleBar.vue`](../../distros/shared/src/components/win98/Win98TitleBar.vue) |
| `@tauri-apps/api/fs` | **自定义** `desktop_fs` commands（非 npm `fs`） | 生产：[`desktop.ts`](../../distros/shared/src/api/desktop.ts) `write_user_text_file` · **零** `@tauri-apps/api/fs` import；preview alias → [`e2e-mock/fs.ts`](../../e2e-mock/fs.ts)（**E2E-only**，见 [`vite.base.config.ts`](../../vite.base.config.ts)） |
| event / listen | `@tauri-apps/api/event` | `roleStore` · `kernelConnectionStore` · `useAppBootstrap` · `HotkeyHost` · `useRolePackImport` |

### 残留清单（允许声明）

| 项 | 说明 |
|----|------|
| [`scripts/split-tauri-api.mjs`](../../scripts/split-tauri-api.mjs) | 模板已改为 `@tauri-apps/api/core`（避免再生成脏码） |
| [`helpers.test.ts`](../../distros/shared/src/api/helpers.test.ts) | 注释已改为「Tauri v2 IPC」 |
| `@tauri-apps/plugin-shell`（npm） | `distros/shared/package.json` 仍声明；**生产 TS/Vue 零 import** → **Observe**（卸载另波，非 01b） |
| `e2e-mock` / vite `--mode e2e` alias | preview 硬门禁所需；**非**生产路径 |
| archive / CHANGELOG 历史叙述 | **不改**（G3） |

**证据面：** `npm run build:e2e` + `npm run test:e2e:preview`（[`send-message.spec.ts`](../../distros/chat-pro/e2e/send-message.spec.ts)）· CI job **`frontend`**。不以 soft `e2e-tauri` 卡 01b Done。
