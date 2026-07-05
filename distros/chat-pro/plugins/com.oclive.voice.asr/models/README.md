# ASR / TTS models (not bundled in git)

Place user-imported or first-run copied models here or under user app data.

## Layout

```
models/
  asr/
    sherpa-paraformer-zh-small/
      MANIFEST.json
      model.int8.onnx
      tokens.txt
  tts/
    sherpa-piper-zh/
      MANIFEST.json
      model.onnx
      tokens.txt
      espeak-ng-data/   (if required by model)
```

## Windows default path (recommended)

```
%APPDATA%/OCLive/models/asr/<profile>/
%APPDATA%/OCLive/models/tts/<profile>/
```

Import via plugin settings **导入模型目录** or set `OCLIVE_VOICE_MODELS_DIR`.

## Sherpa Paraformer zh small/int8

Download sherpa-onnx Paraformer Chinese small int8 assets (see [sherpa-onnx ASR docs](https://k2-fsa.github.io/sherpa/onnx/pretrained_models/offline-paraformer/paraformer-models.html#csukuangfj-sherpa-onnx-paraformer-chinese-small-2024-03-09-int8)) and place under `sherpa-paraformer-zh-small/`.

## Sherpa Paraformer zh medium (optional · higher accuracy)

Profile id: **`sherpa-paraformer-zh-medium`**. Download the **medium** Paraformer ONNX + `tokens.txt` from the same sherpa-onnx docs page; place under `%APPDATA%/OCLive/models/asr/sherpa-paraformer-zh-medium/` with a `MANIFEST.json` that sets `"model_file": "model.onnx"` (or your filename). Select the profile in plugin settings.

## UI capture note

Chat toolbar records WebM/Opus in the browser; **VoiceToolbar.vue** converts to **16 kHz mono WAV** before RPC (reference copy: `slots/audioCapture.ts`). Do not send raw WebM to the Python engine unless **ffmpeg** is on `PATH` (fallback in `asr/audio_decode.py`).

Phase 6: optional installer may seed `%APPDATA%/OCLive/models/` on first launch (manual import until CDN milestone).
