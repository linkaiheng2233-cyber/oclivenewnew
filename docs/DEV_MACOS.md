# macOS 开发与验收

面向在 macOS 上构建、运行与分发 oclive（Tauri 1.x）的说明。通用跨平台说明见 [DEV_CROSS_PLATFORM.md](./DEV_CROSS_PLATFORM.md)。

## M1 运行验证（须在 macOS 实体机执行）

在仓库根目录：

1. **Rust**：`cargo check --workspace`、`cargo test --workspace`
2. **前端**：`npm ci`（或已有 `node_modules` 时跳过）、`npm run build`
3. **Tauri 开发**：`npm run tauri:dev`（或项目 `package.json` 中等价脚本）
4. **核心流程**：启动 → 加载角色包 → 发消息收回复
5. **插件**：目录插件扫描、启用、RPC、停用/卸载

将阻塞性问题（无法启动、崩溃、核心对话不可用）记在下方「问题清单」。

### 问题清单（运行发现）

（在此追加 macOS 特有 bug 与复现步骤。）

---

## M2 快捷键

- 应用内全局快捷键使用 **⌘（Meta）** 作为主修饰键；Windows / Linux 仍为 **Ctrl**。
- 文案通过 i18n 中的 `{m}` 占位符在加载时替换为 `⌘` 或 `Ctrl`。
- `app_data/hotkey_bindings.json` 里若使用 `Ctrl+…` 字符串，在 macOS 上注册全局快捷键时会规范为 `Command+…`。

---

## M3 原生窗口与菜单

- 主窗口默认 **启动时居中**（`tauri.conf.json` → `windows[].center`）。
- **macOS 默认菜单**（关于、服务、窗口等）通过 `tauri::Builder::enable_macos_default_menu(true)` 显式保持开启。
- 当前壳层未启用 **系统托盘**；若后续需要托盘，需在 `Cargo.toml` / `tauri.conf.json` 中启用 `system-tray` 并接入 `SystemTray` API。
- **`.ocpak` 文件关联**：Tauri 1.x 的 `bundle` 配置未内置通用「文件类型关联」字段时，可在分发说明中引导用户「右键 → 打开方式」或后续升级 Tauri 2 / 自定义 `Info.plist` 片段实现。

---

## M5 权限与沙箱

- 本应用默认 **非 Mac App Store 沙箱** 形态；数据目录使用 Tauri `app_data_dir` 等 API，与 Windows 一致。
- **网络**：对话、插件市场、远程模型等需出站连接；`entitlements/macos.plist` 声明了 client/server 网络能力，便于在开启 **Hardened Runtime** 且正式签名时通过校验。
- **目录插件子进程**：由宿主启动外部可执行文件；若签名后子进程加载失败，需在 Apple 文档下检查 **hardened runtime** 与 **Library Validation**，必要时在 entitlements 中按需增加（发布前须安全评审）。
- **本地网络 / 防火墙**：若使用局域网 HTTP API 或插件探活，请在「系统设置 → 网络 / 防火墙」中按需放行。

---

## M4 DMG 与代码签名

- `tauri.conf.json` → `bundle.dmg` 配置了 DMG 卷窗尺寸与图标位置（可按品牌替换 `background` 图）。
- **本地 / CI 无证书**：可使用 **adhoc** 签名以便本地安装测试；终端用户首次打开可能需在「隐私与安全性」中允许。
- **正式发布**：使用 Apple Developer **Application** 证书；`bundle.macOS.signingIdentity` 填钥匙串中的签名身份（如 `Developer ID Application: … (…)`）；配合 **notarytool** 公证。勿将私钥或 `p12` 提交进仓库。
- 完整 `tauri build` 可在 CI 的 `tauri-build-optional` 工作流中试跑（见 `.github/workflows/`）。
