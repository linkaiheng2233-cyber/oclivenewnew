# ASR / TTS models (not bundled in git)



Place user-imported or DLC-copied models under user app data or plugin `models/`.



## Layout



```

models/

  asr/

    sherpa-paraformer-zh-small/

      MANIFEST.json

      model.int8.onnx

      tokens.txt

  tts/

    cosyvoice2-0.5b/

      MANIFEST.json

      voice_model_pack.json

      … CosyVoice2-0.5B weights …

```



## Windows default path (recommended)



```

%APPDATA%/OCLive/models/asr/<profile>/

%APPDATA%/OCLive/models/tts/cosyvoice2-0.5b/

```



Import via settings **导入模型** or set `OCLIVE_VOICE_MODELS_DIR`.



## ASR · Sherpa Paraformer



See [sherpa-onnx Paraformer docs](https://k2-fsa.github.io/sherpa/onnx/pretrained_models/offline-paraformer/paraformer-models.html).



## TTS · CosyVoice2 voice expansion (product)

- Profile id: **`bundled-cosyvoice2-zh`**
- Pack id: **`oclive-tts-cosyvoice2-0.5b-zh`**
- Runtime: `%APPDATA%/OCLive/models/tts/cosyvoice2-0.5b/`
- Text frontend: `%APPDATA%/OCLive/models/tts/cosyvoice-ttsfrd/` (CosyVoice-ttsfrd)

### ModelScope download (official)

```powershell
pip install modelscope
modelscope download --model iic/CosyVoice2-0.5B --local_dir "$env:APPDATA\OCLive\models\tts\cosyvoice2-0.5b"
modelscope download --model iic/CosyVoice-ttsfrd --local_dir "$env:APPDATA\OCLive\models\tts\cosyvoice-ttsfrd"
```

Copy `MANIFEST.json` + `voice_model_pack.json` from [`tts/cosyvoice2-0.5b/`](tts/cosyvoice2-0.5b/) if the download did not include them.

- Recommended: NVIDIA **6GB+** VRAM · **~5.5GB** disk (CosyVoice2) + **~0.3GB** (ttsfrd)
- App: Settings → **语音扩展** → enable → **预热 TTS 侧车** → `auto_tts`

### Python runtime (CosyVoice2 sidecar)

Inference deps SSOT: [`examples/voice-loop-minimal/requirements-cosyvoice-inference.txt`](../../../../examples/voice-loop-minimal/requirements-cosyvoice-inference.txt).

```powershell
cd examples/voice-loop-minimal
py -3.12 -m venv .venv-cosyvoice
.\.venv-cosyvoice\Scripts\pip.exe install "setuptools==69.5.1" wheel
# PyTorch: official pin is cu121; RTX 50-series (sm_120) needs cu128 >= 2.7.1
.\.venv-cosyvoice\Scripts\pip.exe install torch==2.7.1 torchaudio==2.7.1 --index-url https://download.pytorch.org/whl/cu128
.\.venv-cosyvoice\Scripts\pip.exe install -r requirements-cosyvoice-inference.txt
# If openai-whisper build fails: add --no-build-isolation to that package only
```

Clone upstream source (not PyPI `cosyvoice`):

```powershell
git clone --recursive https://github.com/FunAudioLLM/CosyVoice.git D:\oclive-dev-artifacts\CosyVoice
```

Write `.venv-cosyvoice/Lib/site-packages/oclive_cosyvoice_paths.pth` with two lines: CosyVoice root + `third_party/Matcha-TTS`. `findCosyvoicePython()` auto-detects `.venv-cosyvoice/Scripts/python.exe`; optional env:

- `OCLIVE_COSYVOICE_PYTHON` — venv python path
- `OCLIVE_COSYVOICE_MODEL_DIR` — `%APPDATA%\OCLive\models\tts\cosyvoice2-0.5b`

Windows uses **wetext** frontend by default; `CosyVoice-ttsfrd` is Linux-only (see upstream README).

### Manual test checklist (after env install)

1. **Sidecar + warm + prime** (first run may take several minutes; watch stderr `elapsed_ms`):

```powershell
cd examples/voice-loop-minimal
$env:OCLIVE_COSYVOICE_MODEL_DIR = "$env:APPDATA\OCLive\models\tts\cosyvoice2-0.5b"
.\.venv-cosyvoice\Scripts\python.exe -m tts.cosyvoice_sidecar
# Expect stdout: OCLIVE_SIDECAR_READY http://127.0.0.1:50000
```

Another terminal:

```powershell
curl.exe -s -X POST http://127.0.0.1:50000/warm -H "Content-Type: application/json" -d "{}"
cd examples/voice-loop-minimal
$warmJson = @{
  warm = $true
  engine = "cosyvoice2"
  model_dir = "$env:APPDATA\OCLive\models\tts\cosyvoice2-0.5b"
  sidecar_endpoint = "http://127.0.0.1:50000"
} | ConvertTo-Json -Compress
$warmJson | .\.venv-cosyvoice\Scripts\python.exe -m tts.synthesize
# Expect: ok true, primed true, prime_elapsed_ms present
```

2. **Chat Pro**: Settings → enable **语音扩展** → save (auto warm+prime) → send a chat message with **auto_tts** on.

3. **Second synthesis** should be faster than the first; compare `elapsed_ms` in sidecar stderr.

**Piper** is **not** a product fallback. Dev-only: [`examples/voice-loop-minimal/models/tts/sherpa-piper-zh/`](../../../../examples/voice-loop-minimal/models/tts/sherpa-piper-zh/).



## UI capture note



Chat toolbar records WebM/Opus; **VoiceToolbar.vue** converts to **16 kHz mono WAV** before RPC (`slots/audioCapture.ts`).


