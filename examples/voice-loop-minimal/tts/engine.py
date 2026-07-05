"""TTS engines: sherpa-onnx Piper, edge-tts, PilotTTS/CosyVoice adapters."""

from __future__ import annotations

import base64
import json
import wave
from io import BytesIO
from pathlib import Path
from typing import Any

_REQUIRED_FILES = ("model.onnx", "tokens.txt")


def _load_manifest(model_dir: Path) -> dict[str, Any]:
    manifest_path = model_dir / "MANIFEST.json"
    if manifest_path.is_file():
        return json.loads(manifest_path.read_text(encoding="utf-8"))
    return {}


def _model_ready(model_dir: Path) -> tuple[bool, str]:
    if not model_dir.is_dir():
        return False, "model_dir_missing"
    manifest = _load_manifest(model_dir)
    model_file = manifest.get("model_file", "model.onnx")
    tokens_file = manifest.get("tokens_file", "tokens.txt")
    missing = [
        name
        for name, fname in (("model", model_file), ("tokens", tokens_file))
        if not (model_dir / fname).is_file()
    ]
    if missing:
        return False, f"model_files_missing:{','.join(missing)}"
    return True, ""


def probe_engine(model_dir: str | Path, *, engine: str | None = None) -> dict[str, Any]:
    path = Path(model_dir)
    manifest = _load_manifest(path)
    engine_name = engine or manifest.get("engine") or "sherpa-onnx-tts"
    if engine_name == "edge-tts":
        try:
            import edge_tts  # noqa: F401
        except ImportError:
            return {
                "ok": False,
                "engine": "edge-tts",
                "reason": "engine_not_installed",
                "message": "pip install edge-tts",
                "model_dir": str(path),
            }
        return {
            "ok": True,
            "engine": "edge-tts",
            "profile": manifest.get("id", path.name or "edge-tts"),
            "model_dir": str(path),
            "message": "edge-tts ready (online)",
        }
    if engine_name in {"pilot-tts", "cosyvoice"}:
        ready, reason = _model_ready(path) if path.is_dir() else (False, "model_dir_missing")
        if not ready:
            return {
                "ok": False,
                "engine": engine_name,
                "reason": reason or "adapter_not_configured",
                "message": f"Place {engine_name} model under model_dir or use sherpa-piper-zh",
                "model_dir": str(path),
            }
        return {
            "ok": False,
            "engine": engine_name,
            "reason": "adapter_not_installed",
            "message": f"{engine_name} adapter reserved; install engine package separately",
            "model_dir": str(path),
        }

    ready, reason = _model_ready(path)
    try:
        import sherpa_onnx  # noqa: F401
    except ImportError:
        return {
            "ok": False,
            "engine": "sherpa-onnx-tts",
            "reason": "engine_not_installed",
            "message": "pip install -r requirements-tts.txt",
            "model_dir": str(path),
        }
    if not ready:
        return {
            "ok": False,
            "engine": "sherpa-onnx-tts",
            "reason": reason,
            "message": "Place Piper/sherpa TTS model under model_dir",
            "model_dir": str(path),
        }
    return {
        "ok": True,
        "engine": manifest.get("engine", "sherpa-onnx-tts"),
        "profile": manifest.get("id", path.name),
        "model_dir": str(path),
        "message": "TTS ready",
    }


def _pcm_to_wav_base64(samples: list[float], sample_rate: int) -> str:
    import struct

    pcm = b"".join(struct.pack("<h", max(-32768, min(32767, int(s * 32767)))) for s in samples)
    buf = BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm)
    return base64.b64encode(buf.getvalue()).decode("ascii")


def _synthesize_sherpa(
    *,
    path: Path,
    cleaned: str,
    speed: float,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    import sherpa_onnx

    tts_config = sherpa_onnx.OfflineTtsConfig(
        model=sherpa_onnx.OfflineTtsModelConfig(
            vits=sherpa_onnx.OfflineTtsVitsModelConfig(
                model=str(path / manifest.get("model_file", "model.onnx")),
                tokens=str(path / manifest.get("tokens_file", "tokens.txt")),
                data_dir=str(path / manifest.get("data_dir", "espeak-ng-data"))
                if (path / manifest.get("data_dir", "espeak-ng-data")).is_dir()
                else "",
                noise_scale=float(manifest.get("noise_scale", 0.667)),
                noise_scale_w=float(manifest.get("noise_scale_w", 0.8)),
                length_scale=float(manifest.get("length_scale", 1.0)),
            ),
            num_threads=int(manifest.get("num_threads", 2)),
        ),
        max_num_sentences=int(manifest.get("max_num_sentences", 2)),
    )
    if not tts_config.validate():
        return {
            "ok": False,
            "reason": "tts_config_invalid",
            "message": "Check model.onnx, tokens.txt, espeak-ng-data",
            "audio_base64": "",
        }
    tts = sherpa_onnx.OfflineTts(tts_config)
    base_speed = float(manifest.get("speed", 1.0))
    audio = tts.generate(
        cleaned,
        sid=int(manifest.get("speaker_id", 0)),
        speed=max(0.5, min(2.0, base_speed * speed)),
    )
    audio_b64 = _pcm_to_wav_base64(list(audio.samples), audio.sample_rate)
    return {
        "ok": True,
        "audio_base64": audio_b64,
        "sample_rate": audio.sample_rate,
        "profile": manifest.get("id", path.name),
        "engine": manifest.get("engine", "sherpa-onnx-tts"),
    }


def _synthesize_edge_tts(
    *,
    cleaned: str,
    speed: float,
    voice: str | None,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    try:
        import asyncio

        import edge_tts
    except ImportError:
        return {
            "ok": False,
            "reason": "engine_not_installed",
            "message": "pip install edge-tts",
            "audio_base64": "",
            "engine": "edge-tts",
        }

    voice_name = voice or manifest.get("voice") or "zh-CN-XiaoxiaoNeural"
    rate_pct = int((speed - 1.0) * 100)
    rate = f"{rate_pct:+d}%"

    async def _run() -> bytes:
        communicate = edge_tts.Communicate(cleaned, voice_name, rate=rate)
        chunks: list[bytes] = []
        async for chunk in communicate.stream():
            if chunk["type"] == "audio":
                chunks.append(chunk["data"])
        return b"".join(chunks)

    try:
        mp3 = asyncio.run(_run())
    except Exception as exc:  # noqa: BLE001
        return {
            "ok": False,
            "reason": "edge_tts_failed",
            "message": str(exc),
            "audio_base64": "",
            "engine": "edge-tts",
        }
    if not mp3:
        return {
            "ok": False,
            "reason": "edge_tts_empty",
            "audio_base64": "",
            "engine": "edge-tts",
        }
    return {
        "ok": True,
        "audio_base64": base64.b64encode(mp3).decode("ascii"),
        "sample_rate": 24000,
        "profile": manifest.get("id", voice_name),
        "engine": "edge-tts",
        "audio_mime": "audio/mpeg",
    }


def _synthesize_experimental(*, engine: str, path: Path) -> dict[str, Any]:
    probe = probe_engine(path, engine=engine)
    return {
        "ok": False,
        "audio_base64": "",
        **probe,
    }


def synthesize_text(
    *,
    model_dir: str | Path,
    text: str,
    speed: float | None = None,
    directive: dict[str, Any] | None = None,
    engine: str | None = None,
    voice: str | None = None,
) -> dict[str, Any]:
    cleaned = (text or "").strip()
    if not cleaned:
        return {"ok": False, "reason": "empty_text", "audio_base64": ""}

    path = Path(model_dir)
    manifest = _load_manifest(path)
    engine_name = engine or manifest.get("engine") or "sherpa-onnx-tts"
    effective_speed = float(speed if speed is not None else (directive or {}).get("speed", 1.0))
    effective_speed = max(0.5, min(2.0, effective_speed))

    if engine_name == "edge-tts":
        return _synthesize_edge_tts(
            cleaned=cleaned,
            speed=effective_speed,
            voice=voice,
            manifest=manifest,
        )
    if engine_name in {"pilot-tts", "cosyvoice"}:
        return _synthesize_experimental(engine=engine_name, path=path)

    probe = probe_engine(path, engine=engine_name)
    if not probe.get("ok"):
        return {"ok": False, "audio_base64": "", **probe}

    return _synthesize_sherpa(
        path=path,
        cleaned=cleaned,
        speed=effective_speed,
        manifest=manifest,
    )
