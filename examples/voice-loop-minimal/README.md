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

## 运行

### 键盘（v0）

```powershell
python loop.py
```

交互：输入一行文本 → 脚本 `POST /chat` → 打印 `reply`。

### 麦克风 + ASR（Week 2+）

```powershell
pip install -r requirements-asr.txt
# 放置模型到 models/asr/sherpa-paraformer-zh-small/（见 models/asr/.../MANIFEST.json）
python loop.py --mic
```

`--mic-seconds 4` 调整录音时长；`OCLIVE_ASR_MODEL_DIR` 覆盖模型路径。

> HTTP 响应为 **`{ "data": { "reply": "..." }, ... }`**；`loop.py` 已解析。自建客户端请读 `data.reply`。

可选 TTS：

```powershell
pip install pyttsx3
python loop.py --tts

# 或 sherpa TTS（**dev/CI only** · `--tts-sherpa` · 需 models/tts/sherpa-piper-zh/）
# 或 CosyVoice2（`--tts-cosyvoice` · 需侧车 + models/tts/cosyvoice2-0.5b/）
pip install -r requirements-tts.txt
python loop.py --mic --tts-sherpa
```

### SSE stream（Week 3 · 可选）

```powershell
python loop.py --stream
```

同一 prompt 可对比 blocking `/chat` 与 `/chat/stream`；输出 `ttft_ms`（首 token）与 `total_ms`（`done`）。

---

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `OCLIVE_API_BASE` | `http://127.0.0.1:8420` | 内核地址 |
| `OCLIVE_ROLE_PATH` | 仓库内 `roles/mumu` 绝对路径 | 与 HTTP `role_path` 一致 |
| `OCLIVE_SCENE_ID` | `default` | 场景 id |
| `OCLIVE_SESSION_ID` | 脚本内固定 UUID | **勿每轮随机**，否则无记忆 |

---

## 验收（Week 1–2）

- [x] `python loop.py` 输入 `hi` 得到 `reply` 行  
- [x] 同一 session 连聊两轮，第二轮能引用第一轮（需真 LLM，关 mock）  
- [x] `python loop.py --tts` / `--tts-sherpa` 能朗读（可选）
- [x] `python loop.py --mic` 在已放置 sherpa 模型时识别并进 `/chat`（Windows）

---

## 开发板部署（Week 3 · AP + MCU 分工）

> 完整硬件叙事见根目录 [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md) §4–§5 · §12 M5。

| 组件 | 职责 | 说明 |
|------|------|------|
| **AP（应用处理器）** | Ollama + `oclive-kernel-server --api` + Python/C++ 中控 | 与 PC 相同 HTTP 契约：`POST /chat` · 读 `data.reply` |
| **loop / 中控** | 麦克风 · VAD · ASR · TTS · 播放 | 本目录 `loop.py` 或等价实现；**文本不经过 MCU** |
| **MCU** | 舵机 / GPIO | 仅执行 AP 下发的动作指令 |

### PC 模拟 AP（他人可复现）

1. 安装 Ollama 并 `ollama pull hermes3:3b`（或小模型，与蓝图 llm 槽一致）  
2. 构建并启动无头内核：

```powershell
cd <REPO_ROOT>
cargo build -p oclive-kernel-server
$env:OCLIVE_USE_CANONICAL_APP_DATA = "1"
$env:OCLIVE_ROLES_DIR = "<REPO_ROOT>/distros/chat-pro/roles"
$env:RUST_LOG = "info"
# 测记忆时勿设 OCLIVE_HTTP_API_MOCK_LLM
..\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclive-kernel-server.exe --api
```

3. 本目录 venv + `python loop.py --mic`（或 `--stream` 测延迟）  
4. 环境变量：`OCLIVE_API_BASE`（默认 `:8420`）· `OCLIVE_ROLE_PATH` · 固定 `OCLIVE_SESSION_ID`

### 延迟建议

- 立绘情感 LLM：`OCLIVE_PORTRAIT_EMOTION_LLM=0`（玩偶场景）  
- 优先小模型 + `POST /chat/stream`（`loop.py --stream` 可测 ttft）  
- ASR/TTS 模型放 AP 本地存储，避免云端依赖

---

## 相关文档（旧模板已合并至上节）

- [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md)  
- [headless-kernel-minimal](../headless-kernel-minimal/README.md)  
- [human-docs/team/TRACK_VOICE_RECOGNITION.md](../../human-docs/team/TRACK_VOICE_RECOGNITION.md)
