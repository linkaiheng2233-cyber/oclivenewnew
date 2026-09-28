# Windows 开发环境附录

> 通用步骤见 [CONTRIBUTING.md §开发环境](../CONTRIBUTING.md#开发环境) · [02 三十分钟跑通](02_THIRTY_MINUTE_START.md)

## 必备组件

| 组件 | 说明 |
|------|------|
| **Node.js ≥ 22** | 见根 `package.json` `engines`；可选 `.nvmrc` |
| **Rust stable** | `rustup` 默认 toolchain |
| **Visual Studio Build Tools** | 勾选 **「使用 C++ 的桌面开发」**（MSVC 链接器） |
| **Windows SDK** | 含 **rc.exe**（资源编译器）；`npm run tauri:dev` 会自动把 SDK `bin/.../x64` 加入 PATH。未安装时：`winget install Microsoft.WindowsSDK.10.0.26100` |
| **WebView2** | Win10/11 通常已带；**Tauri 2** 桌面壳依赖 |

## Cargo 产物目录（外部 target-dir）

根 [`.cargo/config.toml`](../.cargo/config.toml) 将编译产物放到：

`../oclive-dev-artifacts/oclivenewnew-cargo-target/`

与源码分离。清理前先核实际 `target-dir`、正在运行的构建/调试消费者和冻结证据引用；旧仓内 `target/` 也不能只因目录名或被 Git 忽略就整夹删除。缓存用途、硬链接与维护准入见 [构建债的保留准则](../handoff/debt-marathon/waves/WAVE-20260928-BUILD-OBSERVATION.md#缓存用途与保留准则2026-09-28)。

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

1. 先看错误中无法打开的具体文件，核路径、文件是否存在、权限和磁盘空间；区分输入库缺失与输出 exe/PDB 被占用。
2. 输出被占用时，确认占用者属于自己的构建/调试会话，再从该会话正常退出；不要按进程名结束所有同名程序。
3. 若安全软件确实拦截该文件，查其事件记录，并按本机策略处理具体事件；不把整个产物目录加入排除项作为默认修复。
4. 不先运行全量 `cargo clean`。只有确认是缓存问题，且已核冻结引用、保全/回退、重建预算和绝对路径范围后，才选择有界维护；保留原错误与重建结果，避免清掉诊断证据。

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
