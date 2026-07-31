# com.oclive.voice.asr

Official **side-channel** `voice.asr` + optional **voice expansion** (CosyVoice2 TTS) + `voice.build_directive` for Chat Pro (Windows first, **v0.5.0**).

- **Registry**: [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../../../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) §4.1
- **Does not** enter six slots or `process_message`
- **Base**: trusted-host microphone capture → ASR → idempotent `com.oclive.voice.asr:submit` event → Chat Pro's canonical `onSend` / draft path
- **Expansion** (optional): `tts_expansion_enabled` + `auto_tts` + `role_tts_enabled[role_id]` + pack `voice_profile.json` → CosyVoice2 sidecar → `voice.speak` on `reply`
- **Optional expansion**: users bring GPU, model DLC, or their own cloud API keys; iframe host events remain manifest-allowlisted

## Product split

| Layer | Default | Optional |
|-------|---------|----------|
| Text chat | On | — |
| ASR (mic) | On (model import) | — |
| Emotional TTS | **Off** | Global enable + per-role enable + `voice_profile.json` + model pack |
| Cloud TTS | — | `synth_provider: cloud` or `edge-tts-zh` |

**Piper** removed from Chat Pro product path; retained in [`examples/voice-loop-minimal/`](../../../../examples/voice-loop-minimal/) for dev/CI only.

## RPC contract

All methods return JSON-RPC `result` objects shaped as `{ ok, … }` unless noted.

### `voice.probe` / `voice.probe_tts`

ASR probe uses `voice.probe`. TTS probe uses `voice.probe_tts` (respects expansion + `synth_provider`).

### `voice.list_model_packs`

Lists DLC metadata (`requires_pack`, `min_vram_gb_recommended`, `installed`).

### `voice.warm`

Start CosyVoice2 sidecar (if bundled) and preload model.

### `config_updated` resource transition

When the host leaves bundled CosyVoice, it may attach an internal `resource_transition` with `operation: "unload"`. The gateway waits for active synthesis, asks the loopback sidecar to unload the matching model (or stops its managed child), and returns a matching confirmation. The host retains its lease when release cannot be confirmed.

### `voice.speak`

Requires `tts_expansion_enabled`. Params:

```json
{
  "text": "回复文本",
  "profile": "bundled-cosyvoice2-zh",
  "directive": {
    "schema_version": 1,
    "emotion_tag": "shy",
    "speed": 0.81,
    "energy": "soft",
    "emo_text": "用害羞轻柔的语气",
    "synth_profile": "bundled-cosyvoice2-zh",
    "ref_audio": "D:/…/roles/mumu/assets/voice/ref_shy.wav",
    "ref_text": "参考文本"
  }
}
```

Failure `tts_expansion_disabled` when expansion off. **No Piper fallback.**

### `voice.build_directive`

Unchanged entry; `rules-v1` now fills `emo_text` + role-pack `ref_map`.

### Synth profiles (product)

| profile | engine | 说明 |
|---------|--------|------|
| `bundled-cosyvoice2-zh` | `cosyvoice2` | 默认 · 侧车 + 模型 DLC |
| `local-cosyvoice-http` | `cosyvoice2` | 用户自建 CosyVoice HTTP（`local-http-tts` 为弃用别名） |
| `local-gpt-sovits-http` | `gpt-sovits-http` | GPT-SoVITS :9880（用户本地 · 合规自负） |
| `local-qwen3-tts-http` | `qwen3-tts-http` | Qwen3 OpenAI-compatible :8080 |
| `local-fish-speech-http` | `fish-speech-http` | Fish Speech HTTP :9881（与 Qwen3 默认 :8080 分离） |
| `local-indextts-http` | `indextts-http` | IndexTTS HTTP |
| `edge-tts-zh` | `edge-tts` | 在线无 key · 须 `pip install edge-tts`（voice-loop venv 或 `OCLIVE_VOICE_PYTHON` 指向的 env） |
| `cloud-tts-openai` | `cloud-tts-openai` | 用户 URL/token |
| *(imported)* | `generic-http-adapter` | `voice.import_tts_adapter` + `tts_adapter_pack.json` |

## Engine layout

- **SSOT Python**: [`examples/voice-loop-minimal/asr/`](../../../../examples/voice-loop-minimal/asr/) · [`tts/`](../../../../examples/voice-loop-minimal/tts/)
- **Sidecar**: `python -m tts.cosyvoice_sidecar` (CosyVoice2; see `requirements-cosyvoice.txt`)
- **Gateway**: `rpc_server.mjs` spawns ASR/TTS Python + manages sidecar lifecycle
- **Models**: `%APPDATA%/OCLive/models/{asr,tts}/` or import via settings
- **Model pack**: [`voice_model_pack.schema.json`](voice_model_pack.schema.json)

## UX

- Settings → **插件扩展** → **语音识别** (ASR) + **语音扩展** (TTS, collapsed by default)
- `auto_tts` only when expansion is enabled; automatic playback additionally requires the role id in `role_tts_enabled` and that pack's `voice_profile.json`
- Legacy configs without `role_tts_enabled` migrate only roles that actually contain `voice_profile.json`; new installs start with no role enabled
- Win98 styles: `distros/shared/src/styles/win98/component-voice-settings.css`

## HTTP dev loop

[`examples/voice-loop-minimal/`](../../../../examples/voice-loop-minimal/) — Piper TTS path for loop regression only.
