# 轨道 B · 语音识别与播报

> **读者**：负责 ASR → 对话 → TTS 闭环的工程师（可先不懂 Vue/Rust）。  
> **前置**：[DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md)（§4 Python · §5 模式 B · §9 语音线验收）→ [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md) → [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md) §4–§5  
> **架构归属**（**§1 核心术语** · 独立通道 · 不进六槽）：[ARCHITECTURE_DECOUPLING_PANORAMA.md §1 · §6–§7 · §11.2](./ARCHITECTURE_DECOUPLING_PANORAMA.md)  
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
| **W3** | 开发板迁移说明 + 可选 stream | README「开发板部署」节他人可复现；`--stream` 输出 ttft |

**Phase 0 Done（v0.2.1 基线）**：WebM→16kHz WAV 内联 · `plugin_bridge` 分发 `get_plugin_settings_ui` / `set_plugin_settings_config` · transport `.js`/`.ts` fallback · §10 故障排查。

**Phase 1 Done（TTS）**：`tts_profile` / `auto_tts` 设置 · `voice.speak`（历史 Piper 路径；**产品默认已迁移**，见下）。

**Phase 2 Done（voice_directive）**：`voice.build_directive` · `rules-v1` director · 角色包 `voice_profile.json` 可选覆盖 · 设置页 director 下拉。

### 语音扩展（v0.4 · 情感 TTS · 可选 DLC）

| 阶段 | 目标 | Done 定义 |
|------|------|-----------|
| **VX-0** | 去 Piper 产品路径 | 默认 `bundled-cosyvoice2-zh`；Piper 仅 `voice-loop-minimal` dev/CI |
| **VX-1** | 扩展开关 | `tts_expansion_enabled` 默认关；ASR 与 TTS 设置分区 |
| **VX-2** | CosyVoice2 侧车 | `voice.warm` · 常驻 `tts.cosyvoice_sidecar` · `tts/engine.py` adapter |
| **VX-3** | 模型 DLC | `voice_model_pack.json` · `voice.list_model_packs` · 手动导入 |
| **VX-4** | 角色 ref | `voice_profile` v2 · `ref_map` · `emo_text` |
| **VX-4b** | 人设 → 风格指令 | 无 `emo_text_template` 时从 `core_personality.txt` 规则派生 · 显式 `voice_profile.json` 优先 |
| **VX-5** | 云端并列 | `synth_provider: cloud` · OpenAI-compatible · `edge-tts-zh` |
| **VX-6** | 延迟 | 流式首句 `voice:stream-sentence`（旁白过滤 · 首块更早）· 侧车 `/warm` **prime** dummy 合成 · 角色切换预热 |
| **VX-7** | 引擎契约统一 | `TtsEngine` 协议 + `tts/engines/registry.py` · profile 驱动 `synth_provider` · 非 `cosyvoice2` 不启侧车 · `shouldUseDirectSidecarStream(engine, provider)` |
| **VX-8** | 主流 TTS adapter | Tier-1：`gpt-sovits-http` · `qwen3-tts-http` · `edge-tts` · `cloud-tts-openai` · `fish-speech-http` · `indextts-http` |
| **VX-9** | 通用 HTTP 适配包 | `voice_tts_adapter.schema.json` · `generic-http-adapter` · `voice.import_tts_adapter` · 设置页导入 UI |
| **VX-10** | 愿景与生态 | 社区 directory 插件 · 插件市场 adapter 分类 · `voice_directive` v2 · 全引擎流式 — 见 TECHNICAL_DEBT K-VOICE-02–08 |

**产品原则**：文字默认；情感 TTS 为扩展；不为发声订阅；probe 失败诚实提示，无 Piper 降级。

**声线来源合规（贴风格 ≠ 克隆）**：官方包 / 分发 / 商用的参考音色**只用**原创、明确授权或免版权（CC0）音源；用 `voice_profile.json` 的 `emo_text_template`（CosyVoice2 instruct 文字指令）贴近某类"风格 / 原型"，**不**克隆受版权保护的声优/角色音频，也**不**把第三方角色音色模型（GPT-SoVITS/RVC 等，本质仍是版权声音复制品且各有自身许可）放进官方包或分发。纯本地个人实验属使用者自担。

### 人设 → 风格指令（VX-4b · rules-v1）

`voice.build_directive`（[`rpc_server.mjs`](../../distros/chat-pro/plugins/com.oclive.voice.asr/rpc_server.mjs) · `deriveVoiceStyleFromPersonality`）在收到 `role_path` 时按下列优先级合成 CosyVoice2 instruct：

| 优先级 | 来源 | 覆盖字段 |
|--------|------|----------|
| 1 | 角色包 **`voice_profile.json`**（显式） | `emo_text_template` · `speed` · `energy` · `ref_*` · `synth_profile` |
| 2 | 角色包 **`core_personality.txt`**（规则派生） | 缺省时的 `emo_text_template` · baseline `speed` · `energy` |
| 3 | **`rules-v1` 情绪表** | `{tone}` 占位替换 · 无模板时的 `emo_text` 兜底 |

**规则派生做什么**：读取人设全文，按关键词计分推断声线原型（小女孩 / 少女 / 少年感男声 / 泛角色）与 3 条以内风格修饰（温柔、活泼、毒舌、害羞、关心等），拼成 `emo_text_template`（含 `{tone}`）。否定句（如「不会用语言撒娇」「不挖苦」）**不计分**，避免把禁止项误当特征。

**作者怎么控**：要精确贴某角色声线 → 写 `voice_profile.json` 的 `emo_text_template`（见 [ROLE_PACK_SPEC §10](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)）；只写好人设、不写 voice 文件 → 自动得「够用」的默认风格；社区后续可扩展 **ref 音色库**（VX-4），与 instruct 文字正交。

**手测派生结果**：

1. **产品路径**：切换角色 → 开启「语音扩展 + 自动朗读」→ 预热 TTS → 发消息听声线是否与 persona 大致一致。  
2. **插件 RPC 烟测**（改 `rpc_server.mjs` 后、不必开 Chat Pro UI）：

```powershell
cd <REPO_ROOT>
node distros/chat-pro/plugins/com.oclive.voice.asr/rpc_server.mjs
# 另开终端对 /rpc POST voice.build_directive，params 含 role_path + bot_emotion
```

期望示例（2026-07 · rules-v1）：`mumu` 用手写 `voice_profile`；`shimeng` 偏清冷毒舌少女；`枫侵月` 偏温和少年感；仅一句人设的 `polish-dev` 得泛角色 + 嘴硬修饰。

**自动化矩阵（2026-07-08 · L1）**：`node scripts/test-voice-build-directive.mjs` → 四角色 × neutral/happy/shy **PASS**；`node scripts/test-voice-speak-path.mjs --probe-only` → bundled CosyVoice **ok** · GPT-SoVITS **probe ok**（本地 :9880 在线时）· 离线 engine 诚实 `endpoint_unreachable` / `engine_not_installed`。

**不在本轨道：** Live2D、Chat Pro Vue、`kernel/crates/` 内核、**Chat Pro UI 流式打字机**（见 [CHAT_PRO §2 延迟/stream](./CHAT_PRO_VERTICAL_HANDOFF.md) · 组长或视觉线）。

### VX-10 愿景（OPEN · 台账 K-VOICE-02–08）

| 项 | 说明 | 台账 |
|----|------|------|
| 社区 directory 插件 `com.user.tts.*` | 自带 sidecar/RPC，经 `plugin_rpc_invoke` | K-VOICE-06 |
| 插件市场 adapter 分类 | 姊妹仓 `oclive-plugin-market` | 跨仓 OPEN |
| `voice_directive` v2 + `engine_extras` | RFC 小节后再实现 | K-VOICE-07 |
| 全引擎流式 playback | 统一 chunked audio contract | K-VOICE-08 Deferred |
| Tier-2 引擎（ChatTTS · XTTS · Piper 产品化等） | 靠 VX-9 generic pack 或社区 | K-VOICE-02–05 |

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
- 可选 TTS（**dev loop / Piper 回归**）：`requirements-tts.txt` + `models/tts/sherpa-piper-zh/`。产品路径见 **语音扩展** · CosyVoice2 · [`models/README.md`](../../distros/chat-pro/plugins/com.oclive.voice.asr/models/README.md)

**Done：** `python loop.py --mic` 或 Chat Pro 按住 🎤 → 5 次非空 `reply`。

- [x] W2 · ASR 全链路（Windows sherpa-onnx + Chat Pro 插件）
- [x] W2 · session 固定 UUID（`loop.py` / 手测记录）

### 任务 B5 · 按键说话 / 简单 VAD（1 天）

v1 可用 **按住空格录音、松开识别**（`loop.py --mic`），或 Chat Pro 工具栏 **按住 🎤**（`VoiceToolbar.vue` · `MediaRecorder`）。

**Done：** 与 B4 相同 5 次全链路；可选 `--tts` / `--tts-sherpa` 或插件设置 `auto_tts`。

- [x] W2 · 按住 🎤 / `loop.py --mic` 按键路径

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

- [x] W3 · `loop.py --stream` 打印 `ttft_ms` / `total_ms`

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

- [x] W3 · README「开发板部署」节已填写

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
| **ASR 正常、回复 `LLM_ERROR`（Ollama `localhost:11434`）** | **非语音路径**；语音 `send` 与键盘发送走同一 `send_message` → `process_message` → Ollama。见 [DEV_ENVIRONMENT §3.4](./DEV_ENVIRONMENT.md)（启动 Ollama、`ollama pull`、模型与设置一致） |
| UI 识别胡话 / 空结果 | 确认已用 v0.2.1+（WebM→16kHz WAV）；可换 **medium** profile 或 `loop.py --mic` 对比 |
| `audio_too_quiet` | 靠近麦克风、延长按住；检查系统输入音量 |
| `role_path not found` | `OCLIVE_ROLE_PATH` 或 `--role-path` 指向真实目录 |
| `unsupported bridge command: get_plugin_settings_ui` | 见 [DEV_ENVIRONMENT §10](./DEV_ENVIRONMENT.md)；桌面 `plugin_bridge.rs` 须分发插件设置命令 |
| `vueCompileFailed` / 插槽加载失败（`audioCapture` 等） | **`ui_slots` 的 `.vue` 勿 `import` 同级 `.ts`**（`vue3-sfc-loader` 经 `read_plugin_asset_text` 常请求 `.js` 而磁盘仅有 `.ts`）。逻辑内联进 `.vue` 或仅 `import "vue"`；见 [DIRECTORY_PLUGINS §4.3.1](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Chat Pro 切模式 DB_ERROR | 勿留测试用 `oclive-kernel-server` 占 `:8420`；见 [DEV_ENVIRONMENT §10](./DEV_ENVIRONMENT.md) |
| directive 一直 null | 正常（mumu 无 catalog）；联调用 `distros/chat-pro/roles/demo-doll` |
| 预热成功但发消息无声 / 首句卡住数分钟 | CosyVoice2 `stream=True` 在 Windows 会死锁；侧车 `_collect_synthesis_tensors` 默认已改**非流式**（`OCLIVE_COSYVOICE_STREAM=1` 才尝试流式）。整句合成 ~3s 出声属正常 |
| `/health` 返回 `not_warmed` 或 `model_dir` 不对 | ① 模型包 `iic/CosyVoice2-0.5B` 是否已导入 `%APPDATA%/OCLive/models/tts/cosyvoice2-0.5b`（且含 `MANIFEST.json`）；② 是否有**残留/重复的 cosyvoice_sidecar 进程**占用 50000（全部结束后由 app 重新拉起，避免带旧 env/旧代码） |
| webview 直连侧车流式 CORS / preflight 失败 | 侧车须回 CORS 头 + 处理 `OPTIONS`（`cosyvoice_sidecar.py` 已加）；`tauri.conf.json` CSP `connect-src` 须含侧车端口（默认 `http://127.0.0.1:50000`） |
| 合成成功却仍无声（无报错） | 浏览器自动播放限制：先用鼠标点一下聊天区域（产生用户手势）再发消息；发送时会解锁 Web Audio |
| 选对 TTS 档案仍指向旧模型 | 插件进程内存态残留：整体重启 app；`voice.asr` 配置在 `%LOCALAPPDATA%/OCLive/data/plugin-data/com.oclive.voice.asr/config.json` |
| 自动派生声线与人设不符 | rules-v1 为**通用**关键词映射；精确控制请写 `voice_profile.json` 的 `emo_text_template`（见 [ROLE_PACK_SPEC §10](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · TRACK §1 VX-4b） |
| 换 TTS profile 仍启动 CosyVoice 侧车 | VX-7：仅 `engine=cosyvoice2` 且 `synth_provider=bundled` 才 spawn/warm；GPT-SoVITS/Qwen3 等走 HTTP adapter，无侧车 |
| GPT-SoVITS / Qwen3 probe 失败 | 确认本地服务已启动且端口与 profile `sidecar_endpoint` 一致（Qwen3 默认 **8080** · Fish Speech 默认 **9881**，避免同机冲突）；音色来源合规自负（TRACK §1） |
| `edge-tts-zh` probe `engine_not_installed` | 在 voice-loop venv 或 RPC 使用的 Python 环境执行 `pip install edge-tts` |
| `voice.import_tts_adapter` 失败 | 目录须含 `tts_adapter_pack.json`；示例见 `examples/voice-loop-minimal/tts_adapter_packs/` |

---

## 11. 交付物与 PR

**交付物：**

- [x] `examples/voice-loop-minimal/loop.py` + README  
- [x] ASR（W2）· TTS `--tts` / `--tts-sherpa` 说明  
- [x] session 记忆手测记录  
- [x] 开发板部署节（W3）  
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
