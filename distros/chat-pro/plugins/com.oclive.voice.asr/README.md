# com.oclive.voice.asr

Official **side-channel** `voice.asr` + `voice.speak` plugin for Chat Pro (Windows first).

- **Registry**: [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../../../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) §4.1
- **Does not** enter six slots or `process_message`
- **Flow**: `chat_toolbar` hold-to-talk → `plugin_rpc_invoke(voice.transcribe)` → `com.oclive.voice.asr:submit` → host `send_message`; optional `voice.speak` TTS on `reply`

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

## Engine layout

- **SSOT Python**: [`examples/voice-loop-minimal/asr/`](../../../../examples/voice-loop-minimal/asr/) · [`tts/`](../../../../examples/voice-loop-minimal/tts/)
- **Gateway**: `rpc_server.mjs` spawns `python -m asr.transcribe` / `python -m tts.synthesize` (stdin JSON)
- **Models**: not in git — import via settings or copy to `%APPDATA%/OCLive/models/{asr,tts}/<profile>/`
- **Env**: `OCLIVE_VOICE_ENGINE_ROOT`, `OCLIVE_VOICE_PYTHON`, `OCLIVE_VOICE_MODELS_DIR`

## UX

- **Immersive** mode only shows `chat_toolbar` (unchanged)
- Hold pointer on 🎤 → `MediaRecorder` → transcribe on release
- Settings: `submit_mode` (`send`|`fill`), `auto_tts`, profile import/probe
- Win98 press/recording styles: `distros/shared/src/styles/win98/component-plugin-toolbar.css`

## HTTP dev loop

[`examples/voice-loop-minimal/`](../../../../examples/voice-loop-minimal/) — `python loop.py --mic` without plugin UI.
