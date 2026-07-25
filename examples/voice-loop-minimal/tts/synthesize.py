"""stdin JSON CLI for TTS synthesis."""

from __future__ import annotations

import json
import sys

from tts.engine import probe_engine, synthesize_text, warm_engine


def main() -> int:
    raw = sys.stdin.read()
    payload = json.loads(raw) if raw.strip() else {}
    model_dir = payload.get("model_dir")
    engine = payload.get("engine")
    _NO_MODEL_DIR_ENGINES = {
        "edge-tts",
        "cloud-tts-openai",
        "gpt-sovits-http",
        "qwen3-tts-http",
        "fish-speech-http",
        "indextts-http",
        "generic-http-adapter",
    }
    if not model_dir and engine not in _NO_MODEL_DIR_ENGINES:
        print(json.dumps({"ok": False, "reason": "model_dir_required"}, ensure_ascii=False))
        return 1
    if payload.get("probe"):
        print(
            json.dumps(
                probe_engine(
                    model_dir or ".",
                    engine=engine,
                    sidecar_endpoint=payload.get("sidecar_endpoint"),
                ),
                ensure_ascii=False,
            )
        )
        return 0
    if payload.get("warm"):
        directive = payload.get("directive") if isinstance(payload.get("directive"), dict) else {}
        print(
            json.dumps(
                warm_engine(
                    model_dir=model_dir or ".",
                    engine=engine,
                    sidecar_endpoint=payload.get("sidecar_endpoint"),
                    prime=bool(payload.get("prime", True)),
                    emo_text=str(directive.get("emo_text") or ""),
                    ref_audio=str(directive.get("ref_audio") or ""),
                    ref_text=str(directive.get("ref_text") or ""),
                ),
                ensure_ascii=False,
            )
        )
        return 0
    directive = payload.get("directive") if isinstance(payload.get("directive"), dict) else None
    result = synthesize_text(
        model_dir=model_dir or ".",
        text=str(payload.get("text", "")),
        speed=payload.get("speed"),
        directive=directive,
        engine=engine,
        voice=payload.get("voice"),
        sidecar_endpoint=payload.get("sidecar_endpoint"),
        cloud_url=payload.get("cloud_url"),
        cloud_token=payload.get("cloud_token"),
        cloud_voice_id=payload.get("cloud_voice_id"),
        cloud_model=payload.get("cloud_model"),
    )
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
