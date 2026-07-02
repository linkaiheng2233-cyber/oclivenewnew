"""stdin JSON CLI for TTS synthesis."""

from __future__ import annotations

import json
import sys

from tts.engine import probe_engine, synthesize_text


def main() -> int:
    raw = sys.stdin.read()
    payload = json.loads(raw) if raw.strip() else {}
    model_dir = payload.get("model_dir")
    if not model_dir:
        print(json.dumps({"ok": False, "reason": "model_dir_required"}, ensure_ascii=False))
        return 1
    if payload.get("probe"):
        print(json.dumps(probe_engine(model_dir), ensure_ascii=False))
        return 0
    result = synthesize_text(model_dir=model_dir, text=str(payload.get("text", "")))
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
