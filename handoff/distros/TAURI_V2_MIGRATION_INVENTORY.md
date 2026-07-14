# Tauri v1 → v2 迁移清单（K-PLATFORM-01a Partial）

**状态：** 立项资产（对照表 + 版本快照 + CI 纪律）· **零运行时实装**  
**债项：** [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) · 父 **K-PLATFORM-01 OPEN** · 子 **01a Partial**（本文件）→ **01a Full** / 01b / 01c 另波  
**权威配置：** [`distros/desktop-tauri/tauri.conf.json`](../../distros/desktop-tauri/tauri.conf.json) · [`Cargo.toml`](../../distros/desktop-tauri/Cargo.toml) · [`src/lib.rs`](../../distros/desktop-tauri/src/lib.rs)  
**上游迁移指南：** [Upgrade from Tauri 1.0](https://v2.tauri.app/start/migrate/from-tauri-1/)（v2 capability / plugin 权限）

> **禁止（本波）**：bump `tauri` / `@tauri-apps/*`、改 `tauri.conf` ACL 实装、前端 API 迁移、一次改 ACL+JS+CI。  
> **下一动作（非本文件）**：bump 分支上 **01a Full**（capability 实装 + 单命令 invoke smoke）。

---

## 1. 版本清单（Cargo.lock / package 快照 · 2026-07-14）

| 组件 | 约束（manifest） | 锁 / 解析版本 |
|------|------------------|---------------|
| `tauri` | `1.5`（features 见下） | **1.8.3** |
| `tauri-build` | `=1.5.6` | **1.5.6** |
| `tauri-plugin-deep-link` | `0.1.2`（feature `desktop`） | **0.1.2** |
| `tao` | （经 tauri） | **0.16.11** |
| `wry` | （经 tauri） | **0.24.12** |
| `@tauri-apps/api` | `^1.5.6`（根 · shared · chat-pro · theater） | 锁随 `package-lock` |
| `@tauri-apps/cli` | `1.5.14`（根） | **1.5.14** |

**Cargo features（`oclivenewnew-tauri` → `tauri`）：**  
`window-minimize` · `window-unmaximize` · `window-set-decorations` · `window-close` · `window-maximize` · `window-start-dragging` · `protocol-asset` · `shell-open` · `dialog-open` · `dialog-save` · `global-shortcut`

**与供应链债的接点：** [`KNOWN_VULNERABILITIES.md`](../../creator-docs/security/KNOWN_VULNERABILITIES.md) 警告级中 **「需 Tauri 2」** 相关项：

- gtk-rs GTK3 簇（Linux WebView；audit ignore）— 需 Tauri 2 / wry 升级后方可移除 ignore  
- **RUSTSEC-2025-0057** `fxhash` — 经 Tauri HTML 解析传递  
- **RUSTSEC-2026-0097** `rand` 0.7 — 经 `phf` / Tauri 宏  

本波 **不** bump；升 v2 时须重跑 `cargo audit` 并回写该文件。

---

## 2. allowlist / features → 拟定 v2 capability / permission

路径约定（bump 时）：`distros/desktop-tauri/capabilities/`（或官方 migrate CLI 生成）+ `tauri.conf` `app.security`；权限标识以 bump 当日 `gen/schemas` 为准，下表为 **拟定**，非已落地 ACL。

| v1 来源 | 现状 | 拟定 v2 | 备注 |
|---------|------|---------|------|
| `allowlist.protocol.asset` + `assetScope: ["$RESOURCE/**"]` | conf + feature `protocol-asset` | `app.security.assetProtocol.enable` + `scope`（含 `$RESOURCE/**`） | 角色/插件资源经 asset protocol |
| `allowlist.shell.open` | conf + `shell-open` | `tauri-plugin-shell` + capability `shell:allow-open` | 外链 / 打开路径 |
| `allowlist.dialog.open` / `save` | conf + `dialog-open` / `dialog-save` | `tauri-plugin-dialog` + `dialog:allow-open` · `dialog:allow-save` | 导入导出、选目录 |
| `allowlist.window.minimize` | conf + `window-minimize` | `core:window:allow-minimize` | Win98 合成标题栏；见 BOUNDARIES §6 |
| `allowlist.window.maximize` | + `window-maximize` | `core:window:allow-maximize` | 同上 |
| `allowlist.window.unmaximize` | + `window-unmaximize` | `core:window:allow-unmaximize` | 同上 |
| `allowlist.window.close` | + `window-close` | `core:window:allow-close` | 同上 |
| `allowlist.window.startDragging` | + `window-start-dragging` | `core:window:allow-start-dragging` | 同上 |
| `allowlist.window.setDecorations` | + `window-set-decorations` | `core:window:allow-set-decorations` | 同上；勿扩无关 window API |
| Cargo `global-shortcut`（**无** conf allowlist 键） | `api/hotkeys.rs` → `GlobalShortcutManager` | `tauri-plugin-global-shortcut` + `global-shortcut:allow-register` / `unregister` / `is-registered` | v1 API 已移除；热键 SSOT 仍为 shared `keybindings` |
| `tauri-plugin-deep-link` 0.1.x | `prepare` + `register("oclive", …)` in `lib.rs` | 官方 **deep-link v2** 插件 + 对应 capability | scheme `oclive://`；与 `consume_pending_protocol_installs` 衔接 |
| 自定义协议 `ocliveplugin` | `register_uri_scheme_protocol("ocliveplugin", …)` · CSP `https://ocliveplugin.localhost` | v2 自定义 protocol / asset 服务 API + CSP 对齐 | 插件 shell HTML；见 `serve_ocliveplugin_asset` |
| `security.dangerousRemoteDomainIpcAccess` | domain `https://ocliveplugin.localhost` · window `main` · `enableTauriAPI` · `plugins: ["*"]` | v2 **remote** capability（`remote` URLs + 窄权限集）替代危险远程 IPC 白名单 | **禁止**在 bump 时写成 `*` 全开；收敛为 plugin shell 所需 IPC |
| 应用命令 allowlist（隐式：任意注册命令可 invoke） | `generate_handler!` **122** 条 | 自有 commands 默认对绑定 capability 的窗口可见；按窗/域收紧时用 ACL | 热路径候选见 [`INVOKE_HOTPATH_MATRIX.md`](../INVOKE_HOTPATH_MATRIX.md)（**13** 条）；**非本波实装 smoke** |

**CSP（迁配置时原样对齐，勿弱化）：**  
`default-src 'self' tauri: https://ocliveplugin.localhost; connect-src 'self' tauri: https://ocliveplugin.localhost http://127.0.0.1:50000 ws://127.0.0.1:50000`

---

## 3. 改动面 / blast radius（计量）

| 面 | 数量 / 路径 | 说明 |
|----|-------------|------|
| Tauri 命令 | **122** | `distros/desktop-tauri/src/lib.rs` `generate_handler!` |
| 前端 `@tauri-apps` 消费文件 | **~20** `.ts`/`.vue` | 主要在 `distros/shared/`；另有 chat-pro bootstrap、theater 立绘；01b 波迁移 |
| package 钉版 | 根 + shared + chat-pro + theater | `@tauri-apps/api` `^1.5.6`；CLI 仅根 |
| CI / 构建 | `.github/workflows/ci.yml` · `tauri:dev` / `tauri:build` | 升档前 **留 v1**（见 §4） |
| Dependabot | `.github/dependabot.yml` 已 ignore `tauri` / `tauri-build` / `tauri-*` | 避免自动噪声 PR 混入债波 |

---

## 4. CI 约束（升档前留 v1）

| 规则 | 现状 | 升档后（01a Full / 01c） |
|------|------|-------------------------|
| Workflow 构建 | `cargo build -p oclivenewnew-tauri`（v1 锁） | bump 分支再切 v2；**main 在 Full 前勿升** |
| `tauri_api_smoke` | [`tests/tauri_api_smoke.rs`](../../distros/desktop-tauri/tests/tauri_api_smoke.rs)：**进程内** `AppState` + `process_message`，**不经 Tauri IPC** | 全量 01a Done 另需 **单命令 invoke smoke**（后续 PR；热路径候选 `send_message` / `list_roles`） |
| `e2e-tauri` / dimension5 | 基于现 v1 二进制与路径 | **01c**：workflow + dimension5 口径切 v2（本波不动） |
| 门禁纪律 | 本 inventory 合入后文档门禁绿即可 | bump PR 须含 audit 回写 + 能力对照复核 |

**停止条件（任一即停本债波语义、改开 bump 计划）：**

1. 任何 `tauri` / `@tauri-apps/*` 版本 bump  
2. 试图一次改 ACL + JS + CI  

---

## 5. 子项与后续波（本文件不实施）

| 子项 | 范围 | 状态 |
|------|------|------|
| **01a Partial** | 本 inventory（对照 + 版本 + CI 纪律） | **本波** |
| **01a Full** | capability / ACL 实装于 bump 分支 + 单命令 invoke smoke | 后续 |
| **01b** | `@tauri-apps/api` v2 + chat-pro 发消息 E2E | 后续 |
| **01c** | workflow + dimension5 切 v2 | 后续 |

父 **K-PLATFORM-01** 在 01a–01c 未齐前 **保持 OPEN**；**勿**标父 Done。
