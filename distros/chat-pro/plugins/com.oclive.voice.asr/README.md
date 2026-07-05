# com.oclive.voice.asr

Official **side-channel** `voice.asr` + `voice.speak` + `voice.build_directive` plugin for Chat Pro (Windows first, **v0.3.0**).

- **Registry**: [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../../../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) §4.1
- **Does not** enter six slots or `process_message`
- **Flow**: `chat_toolbar` hold-to-talk → `plugin_rpc_invoke(voice.transcribe)` → `com.oclive.voice.asr:submit` → host `send_message`; optional `voice.speak` TTS on `reply`
- **Reply path** (ASR 与 LLM **解耦**): `mode=send` 时宿主 `onSend` → Tauri `send_message` → kernel HTTP `:8420` → `process_message` → Ollama `POST /api/generate`。ASR 成功但回复 `LLM_ERROR` 表示 **Ollama 未就绪**，不是 `voice.transcribe` 故障；见 [DEV_ENVIRONMENT §3.4](../../../../human-docs/team/DEV_ENVIRONMENT.md)。
- **Plugin config**: `oclive.invoke("get_plugin_settings_ui")` / `set_plugin_settings_config` (manifest `bridge.invoke`) — must be routed in desktop `plugin_bridge.rs` (see [DIRECTORY_PLUGINS.md](../../../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) §4.1)

## RPC contract (frozen)

All methods return JSON-RPC `result` objects shaped as `{ ok, … }` unless noted.

### `voice.probe`

**params**

```json
{ "profile": "sherpa-paraformer-zh-small" }
```

**result**

```json
{
  "ok": true,
  "profile": "sherpa-paraformer-zh-small",
  "engine": "sherpa-onnx",
  "platform": "win32",
  "model_dir": "C:\\Users\\…\\AppData\\Roaming\\OCLive\\models\\asr\\sherpa-paraformer-zh-small",
  "message": "ASR ready"
}
```

Failure reasons include `unsupported_platform`, `engine_not_installed`, `model_dir_missing`, `model_files_missing`.

### `voice.list_profiles`

**result**

```json
{
  "default_profile": "sherpa-paraformer-zh-small",
  "platform": "win32",
  "profiles": [{ "id": "…", "label": "…", "engine": "…", "platform_ready": true }]
}
```

### `voice.import_model`

**params**

```json
{ "src_path": "D:\\models\\sherpa-paraformer-zh-small", "profile": "sherpa-paraformer-zh-small", "kind": "asr" }
```

**result**

```json
{ "ok": true, "profile": "sherpa-paraformer-zh-small", "dest": "…", "kind": "asr" }
```

### `voice.transcribe`

**params**

```json
{
  "profile": "sherpa-paraformer-zh-small",
  "audio_base64": "<wav or pcm>",
  "sample_rate": 16000
}
```

**result**

```json
{ "ok": true, "text": "你好", "profile": "sherpa-paraformer-zh-small", "engine": "sherpa-onnx" }
```

Empty/failed recognition: `{ "ok": false, "text": "", "reason": "…" }` — **host must not send chat**.

### `voice.speak`

**params**

```json
{ "text": "回复文本", "profile": "sherpa-piper-zh" }
```

**result**

```json
{ "ok": true, "audio_base64": "<wav>", "sample_rate": 22050, "profile": "sherpa-piper-zh" }
```

Optional **`directive`** (from `voice.build_directive`) — Piper uses **`speed`** only; `emotion_tag` ignored.

### `voice.build_directive`

**params**

```json
{
  "profile": "rules-v1",
  "bot_emotion": "shy",
  "role_path": "D:\\…\\roles\\mumu"
}
```

**result**

```json
{
  "ok": true,
  "director_profile": "rules-v1",
  "directive": {
    "schema_version": 1,
    "emotion_tag": "shy",
    "speed": 0.81,
    "energy": "soft",
    "emo_text": "",
    "synth_profile": "sherpa-piper-zh"
  }
}
```

Schema: [`voice_directive.schema.json`](voice_directive.schema.json). Director **`none`** or omit → synth-only (Phase 1 behaviour).

### Synth engines (Phase 4 adapters)

| profile | engine | 说明 |
|---------|--------|------|
| `sherpa-piper-zh` | `sherpa-onnx-tts` | 默认离线 Piper |
| `edge-tts-zh` | `edge-tts` | 在线 · `pip install edge-tts` |
| `pilot-tts-zh` | `pilot-tts` | 实验 adapter · 需自备模型 |
| `cosyvoice-zh` | `cosyvoice` | 实验 adapter · 需自备模型 |

## Engine layout

- **SSOT Python**: [`examples/voice-loop-minimal/asr/`](../../../../examples/voice-loop-minimal/asr/) · [`tts/`](../../../../examples/voice-loop-minimal/tts/)
- **Gateway**: `rpc_server.mjs` spawns `python -m asr.transcribe` / `python -m tts.synthesize` (stdin JSON)
- **Models**: not in git — import via settings or copy to `%APPDATA%/OCLive/models/{asr,tts}/<profile>/`
- **Env**: `OCLIVE_VOICE_ENGINE_ROOT`, `OCLIVE_VOICE_PYTHON`, `OCLIVE_VOICE_MODELS_DIR`

## UX

- **Immersive** mode only shows `chat_toolbar` (unchanged)
- Hold pointer on 🎤 → `MediaRecorder` → transcribe on release
- Settings: `submit_mode` (`send`|`fill`), `auto_tts`, `asr_profile`, `tts_profile`, `director_profile`, profile import/probe
- Win98 press/recording styles: `distros/shared/src/styles/win98/component-plugin-toolbar.css`

## HTTP dev loop

[`examples/voice-loop-minimal/`](../../../../examples/voice-loop-minimal/) — `python loop.py --mic` without plugin UI.
