# Windows 开发环境附录

> 通用步骤见 [CONTRIBUTING.md §开发环境](../CONTRIBUTING.md#开发环境) · [02 三十分钟跑通](02_THIRTY_MINUTE_START.md)

## 必备组件

| 组件 | 说明 |
|------|------|
| **Node.js ≥ 20** | 见根 `package.json` `engines`；可选 `.nvmrc` |
| **Rust stable** | `rustup` 默认 toolchain |
| **Visual Studio Build Tools** | 勾选 **「使用 C++ 的桌面开发」**（MSVC 链接器） |
| **WebView2** | Win10/11 通常已带；Tauri 1.x 依赖 |

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
