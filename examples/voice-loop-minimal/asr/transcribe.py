"""stdin JSON / --wav CLI for ASR transcription."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from asr.engine import probe_engine, transcribe_audio


def _read_stdin_json() -> dict:
    raw = sys.stdin.read()
    if not raw.strip():
        return {}
    return json.loads(raw)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="OCLive ASR transcribe (sherpa-onnx)")
    parser.add_argument("--wav", type=Path, help="WAV file path")
    parser.add_argument("--probe", action="store_true", help="Probe model_dir only")
    parser.add_argument("--model-dir", type=Path, dest="model_dir", help="Model directory")
    args = parser.parse_args(argv)

    payload = _read_stdin_json() if not sys.stdin.isatty() else {}
    model_dir = args.model_dir or payload.get("model_dir")
    if not model_dir:
        out = {"ok": False, "reason": "model_dir_required", "text": ""}
        print(json.dumps(out, ensure_ascii=False))
        return 1

    if args.probe or payload.get("probe"):
        print(json.dumps(probe_engine(model_dir), ensure_ascii=False))
        return 0

    result = transcribe_audio(
        model_dir=model_dir,
        audio_base64=str(payload.get("audio_base64", "")),
        wav_path=args.wav,
        sample_rate=payload.get("sample_rate"),
    )
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
