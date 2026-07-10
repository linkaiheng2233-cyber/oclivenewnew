"""sherpa-onnx Paraformer offline ASR."""

from __future__ import annotations

import base64
import json
import math
import struct
import wave
from functools import lru_cache
from io import BytesIO
from pathlib import Path
from typing import Any

from asr.audio_decode import decode_to_wav_bytes, is_compressed_container

_REQUIRED_FILES = ("model.int8.onnx", "tokens.txt")
_MIN_RMS = 0.002


def _load_manifest(model_dir: Path) -> dict[str, Any]:
    manifest_path = model_dir / "MANIFEST.json"
    if manifest_path.is_file():
        return json.loads(manifest_path.read_text(encoding="utf-8"))
    return {}


def _required_files(model_dir: Path) -> tuple[str, ...]:
    manifest = _load_manifest(model_dir)
    model_file = str(manifest.get("model_file", _REQUIRED_FILES[0]))
    tokens_file = str(manifest.get("tokens_file", _REQUIRED_FILES[1]))
    return (model_file, tokens_file)


def _model_ready(model_dir: Path) -> tuple[bool, str]:
    if not model_dir.is_dir():
        return False, "model_dir_missing"
    missing = [name for name in _required_files(model_dir) if not (model_dir / name).is_file()]
    if missing:
        return False, f"model_files_missing:{','.join(missing)}"
    return True, ""


def probe_engine(model_dir: str | Path) -> dict[str, Any]:
    path = Path(model_dir)
    ready, reason = _model_ready(path)
    try:
        import sherpa_onnx  # noqa: F401
    except ImportError:
        return {
            "ok": False,
            "engine": "sherpa-onnx",
            "reason": "engine_not_installed",
            "message": "pip install -r requirements-asr.txt",
            "model_dir": str(path),
        }
    if not ready:
        return {
            "ok": False,
            "engine": "sherpa-onnx",
            "reason": reason,
            "message": "Place Paraformer zh small/int8 under model_dir",
            "model_dir": str(path),
        }
    manifest = _load_manifest(path)
    return {
        "ok": True,
        "engine": manifest.get("engine", "sherpa-onnx"),
        "profile": manifest.get("id", path.name),
        "model_dir": str(path),
        "message": "ASR ready",
    }


def _decode_wav_bytes(raw: bytes) -> tuple[list[float], int]:
    with wave.open(BytesIO(raw), "rb") as wf:
        sample_rate = wf.getframerate()
        n_channels = wf.getnchannels()
        sample_width = wf.getsampwidth()
        n_frames = wf.getnframes()
        pcm = wf.readframes(n_frames)
    if sample_width != 2:
        raise ValueError(f"unsupported sample width: {sample_width}")
    count = len(pcm) // 2
    samples = struct.unpack(f"<{count}h", pcm)
    if n_channels > 1:
        samples = [samples[i] for i in range(0, count, n_channels)]
    floats = [s / 32768.0 for s in samples]
    return floats, sample_rate


def _decode_audio_payload(audio_base64: str, sample_rate: int | None) -> tuple[list[float], int]:
    raw = base64.b64decode(audio_base64)
    if raw[:4] == b"RIFF":
        return _decode_wav_bytes(raw)
    if is_compressed_container(raw):
        wav = decode_to_wav_bytes(raw, sample_rate or 16000)
        if wav:
            return _decode_wav_bytes(wav)
        raise ValueError(
            "compressed audio requires browser-side WAV conversion or ffmpeg in PATH"
        )
    if not sample_rate or sample_rate <= 0:
        raise ValueError("sample_rate required for raw PCM payload")
    count = len(raw) // 2
    samples = struct.unpack(f"<{count}h", raw[: count * 2])
    floats = [s / 32768.0 for s in samples]
    return floats, sample_rate


def _samples_rms(samples: list[float]) -> float:
    if not samples:
        return 0.0
    return math.sqrt(sum(s * s for s in samples) / len(samples))


@lru_cache(maxsize=4)
def _build_recognizer(model_dir: str):
    import sherpa_onnx

    path = Path(model_dir)
    manifest = _load_manifest(path)
    paraformer = str(path / manifest.get("model_file", "model.int8.onnx"))
    tokens = str(path / manifest.get("tokens_file", "tokens.txt"))
    return sherpa_onnx.OfflineRecognizer.from_paraformer(
        paraformer=paraformer,
        tokens=tokens,
        num_threads=int(manifest.get("num_threads", 2)),
        sample_rate=int(manifest.get("sample_rate", 16000)),
        feature_dim=int(manifest.get("feature_dim", 80)),
        decoding_method=str(manifest.get("decoding_method", "greedy_search")),
    )


def transcribe_audio(
    *,
    model_dir: str | Path,
    audio_base64: str = "",
    wav_path: str | Path | None = None,
    sample_rate: int | None = None,
) -> dict[str, Any]:
    path = Path(model_dir)
    probe = probe_engine(path)
    if not probe.get("ok"):
        return {"ok": False, "text": "", **probe}

    try:
        if wav_path:
            wav_bytes = Path(wav_path).read_bytes()
            samples, sr = _decode_wav_bytes(wav_bytes)
        elif audio_base64:
            samples, sr = _decode_audio_payload(audio_base64, sample_rate)
        else:
            return {"ok": False, "text": "", "reason": "no_audio", "message": "audio_base64 or wav_path required"}
    except ValueError as exc:
        return {
            "ok": False,
            "text": "",
            "reason": "bad_audio_format",
            "message": str(exc),
            "profile": probe.get("profile", path.name),
            "engine": probe.get("engine", "sherpa-onnx"),
        }

    rms = _samples_rms(samples)
    if rms < _MIN_RMS:
        return {
            "ok": False,
            "text": "",
            "reason": "audio_too_quiet",
            "message": "input too quiet; check mic level or hold longer",
            "profile": probe.get("profile", path.name),
            "engine": probe.get("engine", "sherpa-onnx"),
        }

    recognizer = _build_recognizer(str(path.resolve()))
    stream = recognizer.create_stream()
    stream.accept_waveform(sr, samples)
    recognizer.decode_stream(stream)
    text = (stream.result.text or "").strip()
    return {
        "ok": bool(text),
        "text": text,
        "profile": probe.get("profile", path.name),
        "engine": probe.get("engine", "sherpa-onnx"),
        "reason": "" if text else "empty_transcript",
    }
