# Plugin engine path (optional)

Production builds may copy [`examples/voice-loop-minimal/asr/`](../../../../examples/voice-loop-minimal/asr/) and [`tts/`](../../../../examples/voice-loop-minimal/tts/) here as `engine/asr/` + `engine/tts/` so `rpc_server.mjs` resolves without the monorepo checkout.

Development uses repo `examples/voice-loop-minimal/` automatically, or set `OCLIVE_VOICE_ENGINE_ROOT`.
