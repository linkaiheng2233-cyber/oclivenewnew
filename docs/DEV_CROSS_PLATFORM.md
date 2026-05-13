# 跨平台开发说明（Linux / macOS / Windows）

面向 **Tauri + Vue + Rust** 壳层与 CI 的跨平台说明。**对话内核** `oclive_kernel_runtime` 与平台无关；窗口、快捷键、打包与权限等适配集中在 **`src-tauri`** 与前端。

## 平台适配状态

| 平台 | 当前支持 | 主要参考 |
|------|----------|----------|
| **Windows** | 开发、CI、安装包（NSIS/MSI 等，`tauri build`）；全局快捷键与路径以 `Ctrl` / 盘符路径为主 | 本文 **「Windows」** 与根目录 [README.md](../README.md)（环境要求、开发、打包） |
| **macOS** | 开发、CI；DMG、默认菜单、⌘ 快捷键、Hardened Runtime / entitlements | **[DEV_MACOS.md](./DEV_MACOS.md)**（验收、签名、权限） |
| **Linux** | 开发、CI（Ubuntu）；AppImage / deb、Ctrl 快捷键、XDG 数据目录、GTK / Wayland 注意点 | **[DEV_LINUX.md](./DEV_LINUX.md)**（验收、打包、权限） |

**CI 持续验证**：`.github/workflows/ci.yml` 中 **`rust`** 与 **`frontend`** job 使用矩阵 **`os: [ubuntu-latest, windows-latest, macos-latest]`**，在三端执行 `cargo fmt` / `clippy` / `cargo test --workspace`（及内核 doc、清单脚本等）与 **`npm ci` + `npm run build`**。  
**完整桌面安装包**（三端 `tauri build`）：**[tauri-build-optional.yml](../.github/workflows/tauri-build-optional.yml)**（定时 / 手动，不阻塞 PR）。

## 常用链接

- [`AGENTS.md`](../AGENTS.md)
- [`handoff/RUST_RELEASE_AND_DEPENDENCIES.md`](../handoff/RUST_RELEASE_AND_DEPENDENCIES.md)
- [`handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md`](../handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md)
- **按平台深入**：[DEV_LINUX.md](./DEV_LINUX.md) · [DEV_MACOS.md](./DEV_MACOS.md)

## Windows

- 开发需 **Visual Studio Build Tools**（MSVC 链接器）；详见根目录 [README.md](../README.md)「环境要求」。
- 控制台子系统、路径分隔符与 `npm run build` / `cargo test` 行为以 **Windows** job 为准排障。

## Linux

与 `ci.yml` 中 **`apt-get install`** 一致（WebKitGTK、GTK3、librsvg、patchelf 等）。**验收清单、AppImage/deb、Wayland 快捷键、XDG 与插件权限** 一律以 **[DEV_LINUX.md](./DEV_LINUX.md)** 为准。

## macOS

Xcode Command Line Tools；可选整包验证见 **`tauri-build-optional.yml`**。**DMG、代码签名、沙箱相关 entitlements、⌘ 快捷键** 以 **[DEV_MACOS.md](./DEV_MACOS.md)** 为准。

## 调试

`windows_subsystem` 仅影响 Windows 控制台；macOS / Linux 从终端运行 `tauri dev` 通常可直接看日志。

## 快捷键与 Deep Link

全局快捷键、`tauri-plugin-deep-link`：三系统各测注册、冲突与外部链接触发；修饰键与文案规则见 `src/lib/shortcutDisplay.ts` 及 **DEV_LINUX** / **DEV_MACOS** 对应小节。

## CI 摘要（与矩阵对齐）

| Job（节选） | 矩阵 | 说明 |
|-------------|------|------|
| `rust` | `ubuntu-latest`, `windows-latest`, `macos-latest` | fmt、clippy、全量 `cargo test`、`oclive_kernel_server` release（仅 Linux）、Tauri 能力校验等 |
| `frontend` | 同上 | `npm ci`、`npm run build` |
| `kernel-runtime-http-linux` | `ubuntu-latest` 单端 | 内核 HTTP/OOCP 特性测试，与桌面矩阵解耦 |

完整步骤以仓库内 **`.github/workflows/ci.yml`** 为准。

## 长期维护（Backlog，非承诺排期）

- **应用商店**：Microsoft Store、Mac App Store、Snap / Flathub 等，按需评估签名、审核与更新通道。
- **应用内更新**：Tauri updater 与各平台签名源配置。
- **ARM**：Apple Silicon 与 ARM64 Linux 原生构建与发布资产验证。
