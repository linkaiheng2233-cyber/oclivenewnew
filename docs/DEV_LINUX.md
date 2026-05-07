# Linux 开发与验收

面向在 Linux 桌面（Tauri 1.x + WebKitGTK）上构建、运行与分发 oclive 的说明。通用跨平台说明见 [DEV_CROSS_PLATFORM.md](./DEV_CROSS_PLATFORM.md)。**内核** `oclive_kernel_runtime` 保持跨平台，适配集中在壳层与前端。

## L1 运行验证（须在 Linux 实体机执行）

推荐：**Ubuntu 22.04 / 24.04** 或 **Fedora**（含 GNOME 或 KDE，分别试 **Wayland** 与 **X11** 会话更佳）。

在仓库根目录：

1. **依赖**（与 `.github/workflows/ci.yml` 中 `apt-get install` 一致）：`libwebkit2gtk-4.0-dev`、`libgtk-3-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`、`patchelf`、`libssl-dev` 等。
2. **Rust**：`cargo check --workspace`、`cargo test --workspace`
3. **前端**：`npm ci`、`npm run build`
4. **Tauri 开发**：`npm run tauri:dev`（或 `package.json` 中等价脚本）
5. **核心流程**：启动 → 加载角色包 → 发消息收回复
6. **目录插件**：扫描、启用、RPC、子进程、停用/卸载

将 **阻塞性问题**（无法启动、Wayland 下白屏/崩溃、文件选择器不可用、核心对话不可用）记入下方「问题清单」。

### 问题清单（运行发现）

（在此追加发行版 / 桌面会话特有的 bug、Compositor、与复现步骤。）

---

## L2 快捷键

- **修饰键**：Linux 与 Windows 一致，主修饰键为 **Ctrl**（界面文案为 `Ctrl`，键盘事件为 `ctrlKey`）；与 macOS 的 ⌘ / `metaKey` 区分见 `src/lib/shortcutDisplay.ts`。
- **全局快捷键**：当前由 Tauri `global-shortcut` + `hotkey_bindings.json` 注册。在 **Wayland** 下，合成器可能不向应用暴露传统全局快捷键能力；若注册失败或不稳定，请在问题清单中记录会话类型（`echo $XDG_SESSION_TYPE`）。后续可考虑 **`tauri-plugin-global-shortcut`** 或 **`muda`** 等与 compositor 协作的方案，并在此文档更新结论。

---

## L3 原生窗口与菜单

- **`enable_macos_default_menu`** 仅作用于 macOS，**不影响** Linux。
- `tauri.conf.json` 中的 **`bundle.macOS` / `bundle.dmg`** 仅在对应平台打包时使用，**不会**在 Linux 运行时生效。
- **装饰与阴影**：由 GTK 与窗口管理器决定；保持 `decorations: true`（默认），在 GNOME / KDE 下确认最小化 / 最大化 / 关闭正常。
- **透明 / 置顶**：`transparent` 在 Linux 上与 WebKitGTK 行为相关；未启用 `macOSPrivateApi` 的 mac 专用项。若某环境出现撕裂或置顶异常，请在问题清单注明 WM 与 Wayland/X11。

---

## L4 权限与文件访问

- **数据目录**：请使用 Tauri `path_resolver`（如 `app_data_dir`），在 Linux 上通常为 **`$XDG_DATA_HOME/<product>`** 或 **`~/.local/share/<product>`**，**不要**依赖 Windows 的 `%APPDATA%`（代码库中业务路径应经 Tauri API 解析）。
- **目录插件子进程**：可执行文件勿放在 **`noexec`** 挂载的临时目录；若发行版启用 **SELinux** / **AppArmor**，插件路径需符合策略或用户显式放行。
- **文件与目录选择**：`allowlist.dialog.open` / `dialog.save` 已开启；若某桌面下 Zenity/Portal 异常，记录桌面与 `xdg-desktop-portal` 版本，并考虑在文档中补充 fallback（例如手动输入路径）的产品需求。

---

## L5 打包产物（AppImage / deb）

- `tauri.conf.json` → **`bundle.appimage`**、**`bundle.deb`** 已配置；`targets` 保持 **`all`**，以便在 **Windows / macOS / Linux** 各自的 CI 或本机 `tauri build` 上仍生成对应平台安装包（避免仅写 `["deb","appimage"]` 导致在非 Linux 上构建失败）。

### 在 Debian / Ubuntu 系上生成 AppImage 与 `.deb`（完整命令示例）

在**仓库根目录**执行；需已安装 **Rust stable** 与 **Node.js**（建议 20+，与 CI 一致）。

```bash
# 1) 系统依赖（与 .github/workflows/ci.yml 中 Linux 步骤一致）
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf

# 2) 安装前端依赖并构建静态资源
npm ci
npm run build

# 3)（可选）与 CI 对齐的 Rust 快速检查
cargo check --workspace

# 4) 生成 Linux 安装包（AppImage + deb，及当前 targets 下其它平台在对应 OS 上的产物）
npm run tauri build
```

- **产物路径**：`src-tauri/target/release/bundle/`（其下按 `appimage`、`deb` 等子目录或文件名组织；具体文件名随 `package.json` 的 `productName` 与版本号变化）。
- **Fedora / 其它发行版**：请用发行版包管理器安装与 **WebKitGTK 4.0**、**GTK3** 等价的 `-devel` / 运行时包后再执行步骤 2–4；包名与 CI 的 `apt` 列表不完全相同属预期。
- **AppImage**：可在无额外网络的情况下试跑；若需音视频且可接受体积增大，可将 `bundle.appimage.bundleMediaFramework` 设为 `true`（当前为 `false`）。
- **deb**：已配置 `section` 等字段；`deb.depends` 请按目标发行版维护（例如 Ubuntu 22.04 与 24.04 的 WebKitGTK 包名可能不同）。

### rpm、AUR 及商店形态（长期可选项 · 非当前发布主线）

以下内容**不影响**当前以 **AppImage、deb、GitHub Releases 资产** 为主的发布与 PR 门禁；按需排期即可。

- **RPM**：可在 `tauri.conf.json` → `bundle` 增加 **`rpm`** 及依赖列表，在 **Fedora / openSUSE** 等环境执行 `npm run tauri build` 验证；默认不纳入阻塞式 CI，以免拉长三端矩阵时间。
- **AUR（Arch User Repository）**：通常由社区维护独立 `PKGBUILD` 仓库（从源码构建或引用上游二进制）；**非**本仓库发版必选项，可在有维护者时再链入文档索引。
- **Snap / Flathub**：属商店与运行时封装层扩展，与内核无关；若产品化上架，需单独工作项（签名、沙箱接口、更新通道等）。可与路线图中的商店类 backlog 对齐。
