"""stdin JSON CLI for TTS synthesis."""

from __future__ import annotations

import json
import sys

from tts.engine import probe_engine, synthesize_text


def main() -> int:
    raw = sys.stdin.read()
    payload = json.loads(raw) if raw.strip() else {}
    model_dir = payload.get("model_dir")
    if not model_dir and payload.get("engine") != "edge-tts":
        print(json.dumps({"ok": False, "reason": "model_dir_required"}, ensure_ascii=False))
        return 1
    if payload.get("probe"):
        print(
            json.dumps(
                probe_engine(model_dir or ".", engine=payload.get("engine")),
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
        engine=payload.get("engine"),
        voice=payload.get("voice"),
    )
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
