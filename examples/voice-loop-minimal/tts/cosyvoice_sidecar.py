"""CosyVoice2 HTTP sidecar — persistent TTS process for voice expansion.

Endpoints:
  GET  /health   — { ok, engine, model_dir, warmed, message }
  POST /warm     — preload model (optional body: { model_dir })
  POST /synthesize — { text, emo_text?, ref_audio?, ref_text?, speed? }

Set env:
  OCLIVE_COSYVOICE_MODEL_DIR — path to imported CosyVoice2-0.5B weights
  OCLIVE_COSYVOICE_PORT      — listen port (default 50000)

Run: python -m tts.cosyvoice_sidecar
Stdout: OCLIVE_SIDECAR_READY http://127.0.0.1:<port>
"""

from __future__ import annotations

import json
import os
import sys
import threading
import time
import wave
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

_ENGINE = "cosyvoice2"
_model_dir: Path | None = None
_model = None
_warmed = False
_lock = threading.Lock()


def _resolve_model_dir() -> Path:
    env = os.environ.get("OCLIVE_COSYVOICE_MODEL_DIR", "").strip()
    if env:
        return Path(env)
    return Path("models/tts/cosyvoice2-0.5b")


def _default_prompt_wav_path() -> Path:
    env = os.environ.get("OCLIVE_COSYVOICE_DEFAULT_PROMPT_WAV", "").strip()
    if env:
        return Path(env)
    try:
        import cosyvoice  # type: ignore[import-untyped]

        repo_root = Path(cosyvoice.__file__).resolve().parent.parent
        return repo_root / "asset" / "zero_shot_prompt.wav"
    except ImportError:
        return Path()


def _resolve_prompt_wav(ref_audio: str) -> Path | None:
    if ref_audio.strip() and Path(ref_audio).is_file():
        return Path(ref_audio)
    fallback = _default_prompt_wav_path()
    return fallback if fallback.is_file() else None


def _format_instruct_text(emo_text: str) -> str:
    text = emo_text.strip()
    if not text:
        return ""
    if "<|endofprompt|>" in text:
        return text
    return f"{text}<|endofprompt|>"


def _model_ready(path: Path) -> tuple[bool, str]:
    if not path.is_dir():
        return False, "model_dir_missing"
    manifest = path / "MANIFEST.json"
    if not manifest.is_file():
        return False, "manifest_missing"
    return True, ""


def _load_cosyvoice_model(path: Path) -> tuple[Any | None, str]:
    global _model, _model_dir, _warmed
    ready, reason = _model_ready(path)
    if not ready:
        return None, reason
    try:
        from cosyvoice.cli.cosyvoice import CosyVoice2  # type: ignore[import-untyped]
    except ImportError:
        return None, "cosyvoice_not_installed"
    try:
        with _lock:
            if _model is not None and _model_dir == path:
                _warmed = True
                return _model, ""
            model = CosyVoice2(str(path))
            _model = model
            _model_dir = path
            _warmed = True
            return model, ""
    except Exception as exc:  # noqa: BLE001
        return None, f"cosyvoice_load_failed:{exc}"


def _synthesize_with_model(
    model: Any,
    *,
    text: str,
    emo_text: str,
    ref_audio: str,
    ref_text: str,
    speed: float,
) -> dict[str, Any]:
    import base64
    import io
    import struct

    cleaned = text.strip()
    if not cleaned:
        return {"ok": False, "reason": "empty_text", "audio_base64": ""}

    try:
        import torchaudio  # type: ignore[import-untyped]
    except ImportError:
        return {
            "ok": False,
            "reason": "torch_not_installed",
            "message": "pip install -r requirements-cosyvoice.txt",
            "audio_base64": "",
        }

    speed = max(0.5, min(2.0, float(speed or 1.0)))
    prompt_text = ref_text.strip() or cleaned[:32]
    iterator = None
    started = time.perf_counter()

    if emo_text.strip():
        prompt_path = _resolve_prompt_wav(ref_audio)
        if prompt_path is None:
            return {
                "ok": False,
                "reason": "prompt_wav_missing",
                "message": "Provide ref_audio or install CosyVoice asset/zero_shot_prompt.wav",
                "audio_base64": "",
            }
        iterator = model.inference_instruct2(
            cleaned,
            _format_instruct_text(emo_text),
            str(prompt_path),
            stream=False,
            speed=speed,
        )
    elif ref_audio and Path(ref_audio).is_file():
        prompt_wav, _rate = torchaudio.load(ref_audio)
        iterator = model.inference_zero_shot(
            cleaned,
            prompt_text,
            prompt_wav,
            stream=False,
            speed=speed,
        )
    else:
        return {
            "ok": False,
            "reason": "ref_audio_missing",
            "message": "Place role-pack ref wav or set emo_text",
            "audio_base64": "",
        }

    audio_tensor = None
    sample_rate = 22050
    for _idx, chunk in enumerate(iterator):
        if isinstance(chunk, dict) and "tts_speech" in chunk:
            audio_tensor = chunk["tts_speech"]
        elif hasattr(chunk, "shape"):
            audio_tensor = chunk
    if audio_tensor is None:
        return {"ok": False, "reason": "cosyvoice_empty", "audio_base64": ""}

    elapsed_ms = int((time.perf_counter() - started) * 1000)
    sys.stderr.write(f"cosyvoice2 synthesize ok elapsed_ms={elapsed_ms} text_len={len(cleaned)}\n")
    sys.stderr.flush()

    if hasattr(audio_tensor, "detach"):
        audio_tensor = audio_tensor.detach().cpu()
    if audio_tensor.ndim > 1:
        audio_tensor = audio_tensor.squeeze()
    samples = audio_tensor.numpy().tolist()
    sample_rate = int(getattr(model, "sample_rate", 22050))

    pcm = b"".join(
        struct.pack("<h", max(-32768, min(32767, int(float(s) * 32767))))
        for s in samples
    )
    buf = io.BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm)
    return {
        "ok": True,
        "audio_base64": base64.b64encode(buf.getvalue()).decode("ascii"),
        "sample_rate": sample_rate,
        "engine": _ENGINE,
        "elapsed_ms": elapsed_ms,
    }


def health_payload() -> dict[str, Any]:
    path = _model_dir or _resolve_model_dir()
    ready, reason = _model_ready(path)
    cosy = _model is not None
    return {
        "ok": ready and cosy,
        "engine": _ENGINE,
        "model_dir": str(path),
        "warmed": _warmed,
        "reason": "" if ready and cosy else (reason or "not_warmed"),
        "message": "CosyVoice2 sidecar ready"
        if ready and cosy
        else "Import CosyVoice2 model pack or install cosyvoice package",
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:
        return

    def _json(self, code: int, payload: dict[str, Any]) -> None:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:
        if self.path.rstrip("/") == "/health":
            self._json(200, health_payload())
            return
        self._json(404, {"ok": False, "reason": "not_found"})

    def do_POST(self) -> None:
        length = int(self.headers.get("Content-Length", "0") or "0")
        raw = self.rfile.read(length) if length else b"{}"
        try:
            body = json.loads(raw.decode("utf-8") or "{}")
        except json.JSONDecodeError:
            self._json(400, {"ok": False, "reason": "bad_json"})
            return

        if self.path.rstrip("/") == "/warm":
            global _model_dir
            md = str(body.get("model_dir") or "").strip()
            if md:
                _model_dir = Path(md)
            path = _model_dir or _resolve_model_dir()
            load_started = time.perf_counter()
            model, err = _load_cosyvoice_model(path)
            load_ms = int((time.perf_counter() - load_started) * 1000)
            if model is None:
                self._json(200, {"ok": False, "reason": err, "engine": _ENGINE, "load_ms": load_ms})
                return
            sys.stderr.write(f"cosyvoice2 warm ok load_ms={load_ms}\n")
            sys.stderr.flush()
            self._json(
                200,
                {
                    "ok": True,
                    "engine": _ENGINE,
                    "warmed": True,
                    "model_dir": str(path),
                    "load_ms": load_ms,
                },
            )
            return

        if self.path.rstrip("/") == "/synthesize":
            path = _model_dir or _resolve_model_dir()
            model, err = _load_cosyvoice_model(path)
            if model is None:
                self._json(
                    200,
                    {"ok": False, "reason": err, "audio_base64": "", "engine": _ENGINE},
                )
                return
            result = _synthesize_with_model(
                model,
                text=str(body.get("text") or ""),
                emo_text=str(body.get("emo_text") or ""),
                ref_audio=str(body.get("ref_audio") or ""),
                ref_text=str(body.get("ref_text") or ""),
                speed=float(body.get("speed") or 1.0),
            )
            self._json(200, result)
            return

        self._json(404, {"ok": False, "reason": "not_found"})


def main() -> None:
    port = int(os.environ.get("OCLIVE_COSYVOICE_PORT", "50000") or "50000")
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    url = f"http://127.0.0.1:{port}"
    sys.stdout.write(f"OCLIVE_SIDECAR_READY {url}\n")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == "__main__":
    main()
