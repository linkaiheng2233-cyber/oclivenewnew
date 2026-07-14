# Windows 开发环境附录

> 通用步骤见 [CONTRIBUTING.md §开发环境](../CONTRIBUTING.md#开发环境) · [02 三十分钟跑通](02_THIRTY_MINUTE_START.md)

## 必备组件

| 组件 | 说明 |
|------|------|
| **Node.js ≥ 20** | 见根 `package.json` `engines`；可选 `.nvmrc` |
| **Rust stable** | `rustup` 默认 toolchain |
| **Visual Studio Build Tools** | 勾选 **「使用 C++ 的桌面开发」**（MSVC 链接器） |
| **Windows SDK** | 含 **rc.exe**（资源编译器）；`npm run tauri:dev` 会自动把 SDK `bin/.../x64` 加入 PATH。未安装时：`winget install Microsoft.WindowsSDK.10.0.26100` |
| **WebView2** | Win10/11 通常已带；**Tauri 2** 桌面壳依赖 |

## Cargo 产物目录（外部 target-dir）

根 [`.cargo/config.toml`](../.cargo/config.toml) 将编译产物放到：

`../oclive-dev-artifacts/oclivenewnew-cargo-target/`

与源码分离；清理旧仓内 `target/` 可整夹删除。

## 首次编译预期

| 阶段 | 耗时（参考） |
|------|----------------|
| `npm install` | 1–3 分钟 |
| 首次 `cargo build`（全 workspace） | **60–120 分钟**（视磁盘与 MSVC 缓存） |
| 后续增量 `npm run tauri:dev` | 数分钟 |

## 常见问题

### RC.EXE / embed-resource panic

Tauri 编译若报 `Are you sure you have RC.EXE in your $PATH`：

1. 安装 Windows SDK（见上表 `winget install Microsoft.WindowsSDK.10.0.26100`）
2. 使用仓库脚本包装：`npm run tauri:dev`（已含 `scripts/with-windows-rc-path.mjs`）

### link.exe not found（MSVC 链接器）

Rust/Tauri 最终链接需要 **Visual Studio Build Tools** 里的 `link.exe`（不是 VS Code）：

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

安装完成后**新开终端**，再 `npm run tauri:dev`。脚本会自动把 MSVC `bin/Hostx64/x64` 加入 PATH。

### LNK1104 / 无法打开文件

- 关闭占用 `oclivenewnew-tauri.exe` 的进程（含上次 `tauri dev`）
- 杀毒软件排除 `oclive-dev-artifacts/` 目录
- `cargo clean` 后重编（仅清外部 target-dir）

### Playwright 超时（Windows `frontend` CI 不跑 E2E）

```powershell
npm run preview -- --host 127.0.0.1 --port 4180 --strictPort
# 另一终端
$env:PW_TEST_USE_EXTERNAL='1'
npm run test:e2e:preview
```

### PowerShell 与 `&&`

旧版 PowerShell 不支持 `&&`；用 `;` 分隔命令，或升级 PS 7+。

## 验证命令

```powershell
npm install
npm run check          # 日常
npm run check:release  # 发版 / 改引擎
```

English: [human-docs-en/10_SETUP_WINDOWS.md](../human-docs-en/10_SETUP_WINDOWS.md)
