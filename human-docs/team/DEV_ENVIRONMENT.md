# Chat Pro 垂直 sprint · 开发环境配置

> **读者**：视觉线 / 语音线组员（Windows 为主；macOS/Linux 见 §8）。  
> **工作区边界**：[SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md) — **语音线可不装 Node、不必全量编译 Chat Pro。**  
> **上级文档**：[CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md)

---

## 1. 硬件与磁盘（开工前）

| 项 | 建议 |
|----|------|
| **系统** | Windows 10/11 64 位（团队当前主环境） |
| **内存** | ≥ 16 GB（首次全量 Rust 编译 + Ollama 3B 同时跑） |
| **磁盘** | ≥ **30 GB 空闲**（见下表） |
| **网络** | 首次需拉 npm、Rust crates、Ollama 模型 |

**磁盘占用（参考）**

| 路径 | 约占用 | 说明 |
|------|--------|------|
| 仓库 `oclivenewnew/` | ~1–2 GB | 不含编译产物 |
| `../oclive-dev-artifacts/oclivenewnew-cargo-target/` | **10–25 GB** | Cargo 外部 target，[`.cargo/config.toml`](../../.cargo/config.toml) 指定 |
| `node_modules/` | ~0.5 GB | `npm install` |
| Ollama 模型 `hermes3:3b` 等 | ~2–5 GB | 语音/记忆测试需要 |
| Python venv（语音线） | ~0.2 GB | 可选 ASR 模型另算 |

---

## 2. 按角色：最少装什么

> 解耦原则：**语音 ≠ 前端开发者；视觉 ≠ 内核开发者。** 详见 [SCOPE_AND_BOUNDARIES.md §2–§3](./SCOPE_AND_BOUNDARIES.md)。

| 组件 | 视觉线 | 语音线 |
|------|:------:|:------:|
| Git | ✅ | ✅ |
| Node.js ≥ 20 | ✅ | ❌ 默认不需要 |
| Rust + MSVC | ✅（`tauri:dev`） | ⭕ 仅 `cargo build -p oclive-kernel-server` 一次 |
| `npm install` / `tauri:dev` | ✅ | ❌ 可用无头内核 + 他人编译的 exe |
| Python 3.10+ | ❌ | ✅ |
| Ollama | ✅ | ✅（测记忆时） |
| curl | ⭕ | ✅ |

**语音线最小路径（不编 Rust）：**

1. 他人或 CI 已构建的 `oclive-kernel-server.exe`，或本机只 `cargo build -p oclive-kernel-server` 一次  
2. Python + `examples/voice-loop-minimal`  
3. Ollama + 小模型  

**视觉线最小路径：**

1. 完整 §3 Windows 套件  
2. `npm run tauri:dev` 日常热重载  

---

## 3. Windows 全套安装（视觉线必做 · 语音线建议做）

### 3.1 安装清单

| # | 软件 | 版本 / 选项 | 验证命令 |
|---|------|-------------|----------|
| 1 | **Git** | 最新 stable | `git --version` |
| 2 | **Node.js** | **≥ 20**（仓库 [`.nvmrc`](../../.nvmrc) 为 `20`） | `node -v` |
| 3 | **Rust** | stable（rustup 默认） | `rustc --version` · `cargo --version` |
| 4 | **VS Build Tools** | 工作负载：**使用 C++ 的桌面开发** | 能成功 `cargo build` |
| 5 | **WebView2** | Win10/11 通常已有 | Tauri 窗口能打开 |
| 6 | **Ollama** | [https://ollama.com](https://ollama.com) Windows 版 | `ollama --version` |
| 7 | **Python** | **3.10+**（语音线必装） | `py -3 --version` 或 `python --version` |
| 8 | **curl** | Win10+ 自带 `curl.exe` | `curl.exe --version` |

### 3.2 一键检查（仓库内）

```powershell
cd D:\oclivenewnew
.\scripts\setup-dev.ps1
```

通过后会提示 `npm install` → `npm run tauri:dev` → `npm run check`。

### 3.3 Clone 与依赖

```powershell
git clone <仓库 URL> D:\oclivenewnew
cd D:\oclivenewnew
npm install
```

**首次编译**（会写外部 target-dir，约 **60–120 分钟**）：

```powershell
npm run tauri:dev
```

后续增量通常数分钟。旧版 PowerShell 请用 **`;` 分隔命令**，不要用 `&&`（见 [10_SETUP_WINDOWS.md](../10_SETUP_WINDOWS.md)）。

### 3.4 Ollama 与对话模型

```powershell
# 安装 Ollama 后
ollama pull hermes3:3b
# 或
ollama pull qwen2.5:3b

ollama list
ollama run hermes3:3b "hello"
```

Chat Pro 内对话失败但编译成功 → 多半是 Ollama 未启动或模型未 pull。语音 **识别正常、仅回复失败**（`LLM_ERROR` + `localhost:11434`）同理——语音 `send` 与键盘发送走同一 `send_message` 链，见 [TRACK_VOICE_RECOGNITION §10](./TRACK_VOICE_RECOGNITION.md)。

**延迟/记忆测试时不要开 mock：**

```powershell
# 不要设置，或显式清除：
Remove-Item Env:OCLIVE_HTTP_API_MOCK_LLM -ErrorAction SilentlyContinue
```

---

## 4. 语音线 Python 环境

```powershell
cd D:\oclivenewnew\examples\voice-loop-minimal
py -3 -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt

# 可选离线 TTS（Week 1 后）
pip install pyttsx3
```

**若 `python` 找不到**：Windows 安装 Python 时勾选 **Add to PATH**，或用 `py -3` 启动器。

**Week 2 ASR（按需，写进 requirements 时再装）：**

```powershell
pip install vosk
# 另需下载 Vosk 语言模型（见 TRACK_VOICE_RECOGNITION.md 任务 B4）
```

---

## 5. 两种运行模式（都要会）

### 模式 A · Chat Pro 桌面（视觉线主用）

```powershell
cd D:\oclivenewnew
npm run tauri:dev
```

- 自动 spawn/attach 内核 `127.0.0.1:8420`  
- Vue 热重载；改 `distros/` 前端 保存即刷新  

### 模式 B · 仅无头内核（语音线主用）

**终端 1 — 内核：**

```powershell
cd D:\oclivenewnew
cargo build -p oclive-kernel-server

$env:OCLIVE_ROLES_DIR = "D:\oclivenewnew\roles"
$env:OCLIVE_USE_CANONICAL_APP_DATA = "1"
$env:RUST_LOG = "info"
# Week 1 快速联调可开 mock（无 Ollama 也能回 reply）：
# $env:OCLIVE_HTTP_API_MOCK_LLM = "1"

..\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclive-kernel-server.exe --api
```

> 可执行文件在仓库**同级**目录 `oclive-dev-artifacts/oclivenewnew-cargo-target/debug/`（由 [`.cargo/config.toml`](../../.cargo/config.toml) 指定）。若路径不同，在资源管理器中搜索 `oclive-kernel-server.exe`。

**终端 2 — 语音 loop：**

```powershell
cd D:\oclivenewnew\examples\voice-loop-minimal
.\.venv\Scripts\Activate.ps1
$env:OCLIVE_ROLE_PATH = "D:\oclivenewnew\roles\mumu"
python loop.py
```

**终端 3 — 健康检查：**

```powershell
curl.exe -s http://127.0.0.1:8420/health
```

也可用 `npm run tauri:dev` 代替模式 B 终端 1（内核同样监听 8420）。

---

## 6. 环境变量速查（本 sprint 常用）

| 变量 | 典型值 | 谁需要 | 作用 |
|------|--------|--------|------|
| `OCLIVE_ROLES_DIR` | `D:\oclivenewnew\roles` | 语音 / 无头 | 角色包根目录 |
| `OCLIVE_USE_CANONICAL_APP_DATA` | `1` | 无头 API | 数据写到 `%LOCALAPPDATA%\OClive\data` |
| `OCLIVE_APP_DATA` | 自定义路径 | 可选 | 覆盖数据目录 |
| `OCLIVE_HTTP_API_MOCK_LLM` | `1` | 仅快速联调 | 假 LLM；**测记忆/延迟须关闭** |
| `OCLIVE_API_PORT` | `8420` | 极少 | 改 HTTP 端口 |
| `OCLIVE_PORTRAIT_EMOTION_LLM` | `0` | 延迟 SKU | 关闭立绘第二次 LLM |
| `OCLIVE_DISTRO_PROFILE` | `...\desktop.oclive.toml` | 可选 | 发行版能力 profile |
| `OLLAMA_MODEL` / 应用内设置 | `hermes3:3b` | 对话 | 与 `ollama pull` 一致 |
| `OCLIVE_API_BASE` | `http://127.0.0.1:8420` | voice-loop | Python 脚本用 |
| `OCLIVE_ROLE_PATH` | `...\roles\mumu` | voice-loop | HTTP `role_path` |
| `OCLIVE_SESSION_ID` | 固定 UUID | voice-loop | **勿每轮随机** |

数据目录详解：[OCLIVE_APP_DATA.md](../../creator-docs/kernel/OCLIVE_APP_DATA.md)

---

## 7. IDE 与推荐扩展

| 工具 | 用途 |
|------|------|
| **VS Code / Cursor** | 仓库根打开 `oclivenewnew` |
| Vue - Official | `.vue` 高亮 |
| rust-analyzer | `distros/desktop-tauri/`、`kernel/crates/`（视觉线改 Rust 时） |
| ESLint | 与仓库配置一致 |

**调试：**

- 前端：`npm run tauri:dev` + 浏览器 DevTools（Tauri 内嵌 WebView）  
- Rust：`RUST_LOG=debug` 或 `info`  
- 内核 HTTP：另开终端看 `oclive-kernel-server` 日志  

---

## 8. macOS / Linux（简要）

| 项 | 说明 |
|----|------|
| Node ≥ 20、Rust stable | 同 Windows |
| macOS | Xcode CLT；Linux 需 `build-essential` 等 |
| Tauri 依赖 | 见 [Tauri 官方 prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) |
| 文档 | [02 三十分钟跑通](../02_THIRTY_MINUTE_START.md) · [CONTRIBUTING.md](../../CONTRIBUTING.md) |

路径分隔符在 HTTP `role_path` 中建议用 **`/`**（Python 脚本已做转换）。

---

## 9. 环境验收清单（Day 0 打勾）

### 共用

- [ ] `.\scripts\setup-dev.ps1` 通过（或手动等价检查 node/rust/cargo）
- [ ] `curl.exe -s http://127.0.0.1:8420/health` 在内核启动后返回 JSON
- [ ] `POST /chat` 响应含 **`reply`** 字段（见 [CHAT_PRO_VERTICAL_HANDOFF.md §3.3](./CHAT_PRO_VERTICAL_HANDOFF.md)）

### 视觉线额外

- [ ] `npm run tauri:dev` 打开 Chat Pro 并能文字对话一轮
- [ ] `npm run test:unit` 通过
- [ ] 知道改 `distros/` 前端 后如何刷新、改 `distros/desktop-tauri/` 后需重新编译

### 语音线额外

- [ ] Python venv 激活后 `python loop.py` 能打印 `reply`
- [ ] `ollama list` 有所需模型；关闭 mock 后能真对话
- [ ] 理解 **固定 `session_id`** 对记忆测试的意义

---

## 10. 常见问题

| 症状 | 处理 |
|------|------|
| `LNK1104` / 无法链接 exe | 关掉正在运行的 `oclivenewnew-tauri.exe` / 旧 `tauri dev` |
| 首次 `cargo build` 极慢 | 正常；产物在外部 target-dir，勿删源码仓内旧 `target/` 误以为没编译 |
| `8420` 连接拒绝 | 先起 `tauri:dev` 或 `oclive-kernel-server --api` |
| 切剧情模式 **DB_ERROR** / `no such table: role_runtime` | 桌面会 **attach** 已有 `:8420` 进程；勿与 `cargo run -p oclive_kernel_server` 测试实例并存。`Get-Process oclive-kernel-server \| Stop-Process -Force` 后重启 `tauri:dev` |
| 插件报 `unsupported bridge command: get_plugin_settings_ui` | manifest 已声明 `bridge.invoke` 时，检查 `distros/desktop-tauri/src/api/plugin_bridge.rs` 是否已分发该命令（见 [DIRECTORY_PLUGINS.md §4.1](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)） |
| Chat 400 empty message | 语音 loop 不要 POST 空字符串 |
| Python 找不到 | 用 `py -3` 或安装时勾选 PATH |
| 对话总是同一句 mock | 清除 `OCLIVE_HTTP_API_MOCK_LLM` |
| Ollama 连接失败 | 系统托盘启动 Ollama；设置里核对 `OLLAMA_BASE_URL` |
| 杀毒拦截编译 | 排除 `oclive-dev-artifacts/` |

更多 Windows 项：[10_SETUP_WINDOWS.md](../10_SETUP_WINDOWS.md)

---

## 11. 相关文档

| 文档 | 内容 |
|------|------|
| [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md) | 团队总览 |
| [TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md) | 视觉任务 |
| [TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md) | 语音任务 |
| [examples/voice-loop-minimal/README.md](../../examples/voice-loop-minimal/README.md) | 语音 loop 运行 |
| [examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md) | 无头 API |
