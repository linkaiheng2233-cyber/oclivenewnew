# 跨平台开发说明（Linux / macOS / Windows）

## 链接

- [`AGENTS.md`](../AGENTS.md)
- [`handoff/RUST_RELEASE_AND_DEPENDENCIES.md`](../handoff/RUST_RELEASE_AND_DEPENDENCIES.md)
- [`handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md`](../handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md)

## Linux

与 `.github/workflows/ci.yml` 中 `apt-get install` 一致（WebKitGTK 等）。验收清单、打包与权限说明见 **[DEV_LINUX.md](./DEV_LINUX.md)**。

## macOS

Xcode Command Line Tools；整包验证见 `.github/workflows/tauri-build-optional.yml`。详细清单、签名与权限说明见 **[DEV_MACOS.md](./DEV_MACOS.md)**。

## 调试

`windows_subsystem` 仅影响 Windows 控制台；macOS/Linux 从终端运行 `tauri dev` 通常可直接看日志。

## 快捷键与 Deep Link

全局快捷键、`tauri-plugin-deep-link`：三系统各测注册、冲突与外部链接触发。

## CI

`ci.yml`：`ubuntu` / `windows` / `macos` 上 Rust 与 `npm run build`。完整 `tauri build` 见 `tauri-build-optional.yml`（不阻塞 PR）。
