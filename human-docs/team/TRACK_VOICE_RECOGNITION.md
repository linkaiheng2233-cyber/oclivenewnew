# 轨道 B · 语音识别与播报

> **读者**：负责 ASR → 对话 → TTS 闭环的工程师（可先不懂 Vue/Rust）。  
> **前置**：[DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md)（§4 Python · §5 模式 B · §9 语音线验收）→ [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md) → [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md) §4–§5  
> **预计周期**：2–3 周（开发板移植在 PC 闭环通过后）  
> **路径占位符**：`<REPO_ROOT>` = 本机 clone 路径（例：`D:\oclivenewnew`），下文命令请替换。

---

## 0. 开发环境（语音线）

| 必装 | 验证 |
|------|------|
| Python 3.10+ | `py -3 --version` |
| Git · curl | 手测 `GET /health` |
| 内核监听 `:8420` | 见 [DEV_ENVIRONMENT.md §5 模式 B](./DEV_ENVIRONMENT.md) |
| Ollama + 小模型（测记忆） | 关闭 `OCLIVE_HTTP_API_MOCK_LLM` |

**不必第一时间装齐**：Node / 全量 `npm install` / 日常 `tauri:dev`（除非联调日看 Chat Pro 界面）。

**工作区白名单 / 禁区** → [SCOPE_AND_BOUNDARIES.md §3](./SCOPE_AND_BOUNDARIES.md)（HTTP 烟测 **只改** `examples/voice-loop-minimal/`；**产品路径**为官方目录插件 [`distros/chat-pro/plugins/com.oclive.voice.asr/`](../../distros/chat-pro/plugins/com.oclive.voice.asr/) · 独立通道 `voice.asr`，见 [RFC §4.1](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md#41-voiceasr-插件通道windows-已交付--宿主侧)）。

**仅首次需要 Rust 时（编无头内核一次）：**

```powershell
cd <REPO_ROOT>
cargo build -p oclive-kernel-server
# 产物在 ../oclive-dev-artifacts/oclivenewnew-cargo-target/debug/（见 .cargo/config.toml）
```

**语音 loop 日常：**

```powershell
cd <REPO_ROOT>\examples\voice-loop-minimal
py -3 -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
python loop.py
```

环境变量表：[DEV_ENVIRONMENT.md §6](./DEV_ENVIRONMENT.md) · 示例 README：[examples/voice-loop-minimal/README.md](../../examples/voice-loop-minimal/README.md)

---

## 1. 你的目标（可验收）

| 阶段 | 目标 | 完成定义（Done） |
|------|------|------------------|
| **W1** | PC 上 HTTP 语音 loop v0 | `python loop.py` 输入文本 → 打印 `reply`；`--tts` 可选朗读 |
| **W2** | 真 ASR + 固定 session 记忆 | 麦克风/按键 → 全链路 5 次成功；连聊 3 轮能引用上下文 |
| **W3** | 开发板迁移说明 + 可选 stream | README「开发板部署」节他人可复现；可选 `--stream` 输出 ttft |

**不在本轨道：** Live2D、Chat Pro Vue、`kernel/crates/` 内核、**Chat Pro UI 流式打字机**（见 [CHAT_PRO §2 延迟/stream](./CHAT_PRO_VERTICAL_HANDOFF.md) · 组长或视觉线）。

---

## 2. 架构边界（必须遵守）

```text
┌─────────────────────────────────────┐
│  你们实现（本轨道）                    │
│  麦克风 · VAD · ASR · TTS · 播放      │
└──────────────┬──────────────────────┘
               │ 只传文本 / 只读 JSON
┌──────────────▼──────────────────────┐
│  OClive 内核（不要改）                 │
│  POST /chat 或 /chat/stream           │
│  127.0.0.1:8420                       │
└─────────────────────────────────────┘
```

| 规则 | 说明 |
|------|------|
| ASR 输出 | 纯文本 → HTTP 请求体 **`message`** 字段 |
| 回复字段 | 读 **`reply`**（在 `data.reply`，见 §4.2）；禁止自造 `response` |
| session | 固定 **`session_id`**（UUID）；`loop.py` 默认已常量化，勿每轮随机 |
| 不进内核 | Whisper / Vosk / Piper 等为 **sidecar 或本目录脚本** |

开发板最终形态与 [HARDWARE_INTEGRATION.md §4](../../HARDWARE_INTEGRATION.md) 伪代码 **相同**。

---

## 3. 必读文件（约 45 分钟）

| # | 文件 | 看什么 |
|---|------|--------|
| 1 | [SCOPE_AND_BOUNDARIES.md §3](./SCOPE_AND_BOUNDARIES.md) | 白名单 |
| 2 | [examples/voice-loop-minimal/README.md](../../examples/voice-loop-minimal/README.md) | 运行 |
| 3 | [examples/voice-loop-minimal/loop.py](../../examples/voice-loop-minimal/loop.py) | `post_chat` · `extract_reply` |
| 4 | [HARDWARE_INTEGRATION.md §4–§5](../../HARDWARE_INTEGRATION.md) | 主循环 + API |

**不必读**：整个 `distros/` 前端、`handoff/LIVE2D_*`、`PLUGIN_V1`、`human-docs/06_KERNEL`。

**Week 3 接 `/chat/stream` 时只读**（不改）：`kernel/crates/oclive_kernel_host/src/http_api/chat.rs` 中 SSE 事件名 `token` / `done` / `error`。

---

## 4. HTTP 契约速查

### 4.1 健康检查

```http
GET http://127.0.0.1:8420/health
```

### 4.2 一轮对话（Week 1–2 主用）

**请求：**

```http
POST http://127.0.0.1:8420/chat
Content-Type: application/json

{
  "role_path": "<REPO_ROOT>/distros/chat-pro/roles/mumu",
  "message": "用户说的话",
  "scene_id": "default",
  "session_id": "00000000-0000-4000-8000-000000000001"
}
```

> Windows 路径在 JSON 里用 **正斜杠** `D:/oclivenewnew/distros/chat-pro/roles/mumu`，或设环境变量 `OCLIVE_ROLE_PATH`（见 `loop.py`）。

**响应形状（重要）：**

HTTP 体是 **包装结构**，`reply` 在 **`data` 里**，不是顶层：

```json
{
  "data": {
    "reply": "……",
    "bot_emotion": "happy",
    "visual_state_id": "happy_default",
    "performance_directive": { "kind": "image", "path": "assets/images/happy.webp", ... }
  },
  "personality_source": "vector",
  "session_id": "00000000-0000-4000-8000-000000000001"
}
```

`loop.py` 的 `extract_reply()` 已处理上述包装。**自己写客户端时务必解析 `body["data"]["reply"]`。**

**联调时可打印给视觉同事 A：**

```python
inner = body.get("data") or body
directive = inner.get("performance_directive")
print("[directive]", directive)
```

| 字段（在 `data` 内） | 用途 |
|----------------------|------|
| `reply` | TTS 朗读 |
| `bot_emotion` | 可选：灯效 / 日志 |
| `performance_directive` | 可选：视觉线立绘（你只管打印，不改 UI） |

### 4.3 流式（Week 3 · 降延迟）

```http
POST http://127.0.0.1:8420/chat/stream
```

请求体与 §4.2 相同。SSE 事件：

| event | 含义 |
|-------|------|
| `token` | `{"token":"..."}` 增量文本 |
| `done` | 完整 `ChatApiResponse` JSON（`data` 内仍有 `reply`、directive） |
| `error` | 错误体 |

**Done 目标：** 首 `token` 起即可分段 TTS（与 Chat Pro UI 是否流式无关）。

---

## 5. 启动内核（任务 B1 前置 · 可复制）

**推荐：无头内核（不必装 Node）**

```powershell
cd <REPO_ROOT>
cargo build -p oclive-kernel-server   # 仅首次或内核升级后

$env:OCLIVE_USE_CANONICAL_APP_DATA = "1"
$env:OCLIVE_ROLES_DIR = "<REPO_ROOT>/roles"
$env:RUST_LOG = "info"
# Week 1 联调可开 mock；测记忆必须去掉下一行：
# $env:OCLIVE_HTTP_API_MOCK_LLM = "1"

..\oclive-dev-artifacts\oclivenewnew-cargo-target\debug\oclive-kernel-server.exe --api
```

另开终端验证：

```powershell
curl.exe -s http://127.0.0.1:8420/health
```

**备选：** 视觉同事已开 `npm run tauri:dev` → 内核通常已在 `:8420`，你 **不必** 再起一份（若端口冲突见 [DEV_ENVIRONMENT §10](./DEV_ENVIRONMENT.md)）。

---

## 6. Week 1 任务清单

### 任务 B1 · 跑通 voice-loop v0（1 天）

```powershell
cd <REPO_ROOT>\examples\voice-loop-minimal
python loop.py
```

**Done：**

- [ ] 输入 `hello` 终端出现 `bot> …`（非 `KeyError: reply`）  
- [ ] [README](../../examples/voice-loop-minimal/README.md) 每一步你本人走通  

### 任务 B2 · session 记忆测试（0.5 天）

`loop.py` 已用固定 `DEFAULT_SESSION_ID`；可通过 `OCLIVE_SESSION_ID` 覆盖。**无需**每轮 `uuid4()`。

连聊：

1. 「我叫小明」  
2. 「我叫什么名字？」  

**Done：** 第二轮 `reply` 提到「小明」（需 **真 LLM**，关闭 mock）。

```powershell
# 不要设 OCLIVE_HTTP_API_MOCK_LLM=1
ollama pull hermes3:3b
# 内核进程需能访问 Ollama；蓝图 llm 槽为 ollama
```

### 任务 B3 · TTS 验收（0.5 天）

仓库 **已实现** `python loop.py --tts`（pyttsx3）。

**Done：**

- [ ] `pip install pyttsx3` 后 `--tts` 能朗读  
- [ ] README 写明 TTS 依赖与 Windows 注意项  
- [ ] Week 2 再评估 Piper / edge-tts（可选升级，非阻塞 W1）  

---

## 7. Week 2 任务清单

### 任务 B4 · 真实 ASR（2–3 天）

**已交付（Windows）**：**sherpa-onnx Paraformer** · SSOT 在 [`examples/voice-loop-minimal/asr/`](../../examples/voice-loop-minimal/asr/) · 产品网关 [`com.oclive.voice.asr`](../../distros/chat-pro/plugins/com.oclive.voice.asr/) · RPC 契约见插件 [`README.md`](../../distros/chat-pro/plugins/com.oclive.voice.asr/README.md)。

**模型准备（不入 git）：**

1. 下载 [sherpa-onnx Paraformer zh small int8](https://k2-fsa.github.io/sherpa/onnx/pretrained_models/offline-paraformer/paraformer-models.html#csukuangfj-sherpa-onnx-paraformer-chinese-small-2024-03-09-int8)  
2. 放到 `examples/voice-loop-minimal/models/asr/sherpa-paraformer-zh-small/`（`MANIFEST.json` 所列文件）**或** `%APPDATA%/OCLive/models/asr/sherpa-paraformer-zh-small/`（Chat Pro 设置 → 导入模型目录）  
3. `pip install -r requirements-asr.txt`（HTTP 烟测）或 Chat Pro 内由 `rpc_server.mjs` spawn Python

**要求：**

- **ASR 空结果 → 不要 POST /chat**（空 `message` 会 400）  
- 可选 TTS：`requirements-tts.txt` + `models/tts/sherpa-piper-zh/`（见 [`models/README.md`](../../distros/chat-pro/plugins/com.oclive.voice.asr/models/README.md)）

**Done：** `python loop.py --mic` 或 Chat Pro 按住 🎤 → 5 次非空 `reply`。

### 任务 B5 · 按键说话 / 简单 VAD（1 天）

v1 可用 **按住空格录音、松开识别**（`loop.py --mic`），或 Chat Pro 工具栏 **按住 🎤**（`VoiceToolbar.vue` · `MediaRecorder`）。

**Done：** 与 B4 相同 5 次全链路；可选 `--tts` 或插件设置 `auto_tts`。

### 任务 B6 · 与视觉线联调（0.5 天）

> **重要：`session_id` 限制**  
> Chat Pro UI 的 `send_message` **默认不会**使用你在 `loop.py` 里配置的 `OCLIVE_SESSION_ID`。  
> 因此 **「同一 session 下 UI + loop 同时记忆」在本 sprint 不保证**。  
> 联调请用下列 **二选一**：

| 方式 | 谁做什么 |
|------|----------|
| **A（推荐）** | 你只跑 `loop.py`；视觉 A 在 Chat Pro **手动打字**同一角色，各自验证 JSON / UI |
| **B** | 你打印 `performance_directive` 到日志；A 对照 [TRACK_VISUAL §A6](./TRACK_VISUAL_UPGRADE.md) 看 UI 切图 |

**Done：** B 触发 `/chat` 后，日志里 `performance_directive` 非空（角色需 catalog，通常 `distros/chat-pro/roles/demo-doll`）；或 A 截图证明 UI 切图。

---

## 8. Week 3 任务清单

### 任务 B7 · `/chat/stream` 实验（1–2 天，可选）

在 `loop.py` 增加 `--stream`：

- 记录 **ttft_ms**（首 token）、**total_ms**（`done`）  
- 首 token 起分段 print 或分段 TTS  

若仓库已有 `scripts/bench-ttft.mjs`，输出格式与其对齐便于组长对比。

**Done：** 同一句 prompt，对比 `/chat` vs `/chat/stream` 的 ttft 数字（截图或日志）。

### 任务 B8 · 开发板迁移文档（1 天）

在 [examples/voice-loop-minimal/README.md](../../examples/voice-loop-minimal/README.md) 填写 **「开发板部署」**：

| 项 | 内容 |
|----|------|
| AP | `oclive-kernel-server --api` + Ollama |
| loop | AP 上 Python 或 C++ 中控，同样 HTTP |
| MCU | 仅舵机；文本 **不经过** MCU |
| 环境变量 | `OCLIVE_ROLES_DIR`、`OCLIVE_APP_DATA` |
| 延迟 | `OCLIVE_PORTRAIT_EMOTION_LLM=0`、小模型、stream |

**Done：** 另一成员按文档在 PC 模拟 AP 复现（不必真板子）。

### 任务 B9 · 目录插件形态（可选）

仅当组长要求：参考 `examples/directory-plugin-minimal/`，**不占六槽**。

---

## 9. 禁区

| 禁止 | 原因 |
|------|------|
| 改 `process_message` / `kernel/crates/` | 内核轨道 |
| 改 `distros/` 前端 Vue | 视觉轨道 |
| ASR 进 `slot_registry` | 架构边界 |
| 公网暴露 `:8420` | 仅 loopback |
| 量产唯一依赖云端 ASR | 玩偶要离线 |

---

## 10. 故障排查（先自查）

| 现象 | 先查 |
|------|------|
| `KeyError: reply` / 找不到 reply | 是否解析 **`data.reply`** |
| HTTP 400 | `message` 是否空；ASR 是否误 POST |
| `8420` 连接拒绝 | 内核是否启动；[DEV_ENVIRONMENT §10](./DEV_ENVIRONMENT.md) |
| 记忆像没有 | `OCLIVE_SESSION_ID` 是否固定；是否 mock LLM |
| `role_path not found` | `OCLIVE_ROLE_PATH` 或 `--role-path` 指向真实目录 |
| directive 一直 null | 正常（mumu 无 catalog）；联调用 `distros/chat-pro/roles/demo-doll` |

---

## 11. 交付物与 PR

**交付物：**

- [ ] `examples/voice-loop-minimal/loop.py` + README  
- [ ] ASR（W2）· TTS `--tts` 说明  
- [ ] session 记忆手测记录  
- [ ] 开发板部署节（W3）  
- [ ] 联调记录（directive 或 A 的截图）  

**PR 描述必勾选：**

- [ ] 仅改 `examples/voice-loop-minimal/`（及本 track 文档）  
- [ ] 未改 `distros/` 前端 · `kernel/crates/` · 对方视觉目录  
- [ ] README 验收步骤可复现  

审阅规则：[CHAT_PRO_VERTICAL_HANDOFF.md §5](./CHAT_PRO_VERTICAL_HANDOFF.md)

---

## 12. 找谁问什么

| 问题 | 找 |
|------|-----|
| `/chat` 4xx/5xx | 组长：贴响应 JSON + `/health` |
| 记忆 / 模型 | 是否 mock；Ollama 是否运行 |
| `role_path` | 组长：`OCLIVE_ROLES_DIR` |
| 立绘 / directive | 视觉 A · [TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md) |
| Chat Pro stream UI | 组长（本轨道只做 HTTP stream 实验） |
