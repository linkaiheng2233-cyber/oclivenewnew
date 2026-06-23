# Voice loop minimal · ASR/TTS 闭环起点

> **读者**：语音识别轨道工程师（轨道 B）。  
> **上级文档**：[human-docs/team/TRACK_VOICE_RECOGNITION.md](../../human-docs/team/TRACK_VOICE_RECOGNITION.md)  
> **开发环境**：[human-docs/team/DEV_ENVIRONMENT.md](../../human-docs/team/DEV_ENVIRONMENT.md)  
> **工作区**：本目录 **仅语音同事修改**；视觉同事 **不要改这里**。边界见 [SCOPE_AND_BOUNDARIES.md](../../human-docs/team/SCOPE_AND_BOUNDARIES.md) §3。

本示例 **不在 OClive 内核内**；仅通过 HTTP 调用与 Chat Pro 相同的后端。

---

## 前置

| 项 | 要求 |
|----|------|
| Python | 3.10+ |
| 内核 | `http://127.0.0.1:8420` 已启动 |
| 角色包 | 默认 `roles/mumu`；联调立绘建议 `roles/demo-doll`（视觉线 A2 创建） |
| 路径 | `<REPO_ROOT>` = 仓库根（例 `D:\oclivenewnew`） |

### 启动内核（二选一）

**A. Chat Pro 开发模式（会 spawn 内核）**

```powershell
cd <REPO_ROOT>
npm run tauri:dev
```

**B. 仅无头 API（语音线推荐）**

```powershell
cd <REPO_ROOT>
cargo build -p oclive-kernel-server
$env:OCLIVE_USE_CANONICAL_APP_DATA = "1"
$env:OCLIVE_ROLES_DIR = "<REPO_ROOT>/roles"
$env:RUST_LOG = "info"
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"   # Week 1 联调可开；测记忆须去掉
..\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclive-kernel-server.exe --api
```

> 可执行文件在仓库外 `oclive-dev-artifacts/oclivenewnew-cargo-target/debug/`（见根目录 `.cargo/config.toml`）。

验证：

```powershell
curl.exe -s http://127.0.0.1:8420/health
```

---

## 安装

完整环境说明见 [human-docs/team/DEV_ENVIRONMENT.md](../../human-docs/team/DEV_ENVIRONMENT.md) §4。

```powershell
cd D:\oclivenewnew\examples\voice-loop-minimal
py -3 -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

> Windows 若 `python` 不可用，统一用 **`py -3`**。

---

## 运行（v0 · 键盘模拟 ASR）

```powershell
python loop.py
```

交互：输入一行文本 → 脚本 `POST /chat` → 打印 `reply`。

> HTTP 响应为 **`{ "data": { "reply": "..." }, ... }`**；`loop.py` 已解析。自建客户端请读 `data.reply`。

可选 TTS（Windows 离线）：

```powershell
pip install pyttsx3
python loop.py --tts
```

---

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `OCLIVE_API_BASE` | `http://127.0.0.1:8420` | 内核地址 |
| `OCLIVE_ROLE_PATH` | 仓库内 `roles/mumu` 绝对路径 | 与 HTTP `role_path` 一致 |
| `OCLIVE_SCENE_ID` | `default` | 场景 id |
| `OCLIVE_SESSION_ID` | 脚本内固定 UUID | **勿每轮随机**，否则无记忆 |

---

## 验收（Week 1）

- [ ] `python loop.py` 输入 `hi` 得到 `reply` 行  
- [ ] 同一 session 连聊两轮，第二轮能引用第一轮（需真 LLM，关 mock）  
- [ ] `python loop.py --tts` 能朗读（可选）

---

## 开发板部署（Week 3 由 B 填写）

模板：

1. AP 安装 Ollama + 拉取 `hermes3:3b`  
2. 启动 `oclive-kernel-server --api`，设置 `OCLIVE_ROLES_DIR`  
3. 本目录 `loop.py` 或等价 C++ 中控在同一机器运行  
4. MCU 仅负责舵机；文本链路不经过 MCU  

延迟建议见根目录 [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md) §4、`§12 M5`。

---

## 相关文档

- [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md)  
- [headless-kernel-minimal](../headless-kernel-minimal/README.md)  
- [human-docs/team/TRACK_VOICE_RECOGNITION.md](../../human-docs/team/TRACK_VOICE_RECOGNITION.md)
