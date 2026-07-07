"""TTS engines: CosyVoice2 sidecar, edge-tts, cloud OpenAI-compatible, sherpa Piper (dev/CI)."""

from __future__ import annotations

import base64
import json
import wave
from io import BytesIO
from pathlib import Path
from typing import Any
from urllib import error as urlerror
from urllib import request as urlrequest

_DEFAULT_TTS_PROFILE = "bundled-cosyvoice2-zh"


def _model_ready_sherpa(model_dir: Path) -> tuple[bool, str]:
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


def _load_manifest(model_dir: Path) -> dict[str, Any]:
    manifest_path = model_dir / "MANIFEST.json"
    if manifest_path.is_file():
        return json.loads(manifest_path.read_text(encoding="utf-8"))
    return {}


def _http_json(
    url: str,
    payload: dict[str, Any] | None = None,
    *,
    method: str = "GET",
    headers: dict[str, str] | None = None,
    timeout: float = 120.0,
) -> dict[str, Any]:
    data = None
    req_headers = {"Content-Type": "application/json; charset=utf-8", **(headers or {})}
    if payload is not None:
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    req = urlrequest.Request(url, data=data, headers=req_headers, method=method)
    try:
        with urlrequest.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urlerror.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        return {"ok": False, "reason": "http_error", "message": body[:300]}
    except Exception as exc:  # noqa: BLE001
        return {"ok": False, "reason": "http_unreachable", "message": str(exc)}


def _sidecar_base(
    *,
    manifest: dict[str, Any],
    sidecar_endpoint: str | None = None,
) -> str:
    if sidecar_endpoint and sidecar_endpoint.strip():
        return sidecar_endpoint.strip().rstrip("/")
    port = int(manifest.get("sidecar_port", 50000) or 50000)
    return f"http://127.0.0.1:{port}"


def probe_engine(
    model_dir: str | Path,
    *,
    engine: str | None = None,
    sidecar_endpoint: str | None = None,
) -> dict[str, Any]:
    path = Path(model_dir)
    manifest = _load_manifest(path)
    engine_name = engine or manifest.get("engine") or "cosyvoice2"

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

    if engine_name == "cloud-tts-openai":
        return {
            "ok": True,
            "engine": engine_name,
            "profile": manifest.get("id", "cloud-tts-openai"),
            "model_dir": str(path),
            "message": "cloud TTS ready when URL/token configured in settings",
        }

    if engine_name == "sherpa-onnx-tts":
        ready, reason = _model_ready_sherpa(path)
        try:
            import sherpa_onnx  # noqa: F401
        except ImportError:
            return {
                "ok": False,
                "engine": "sherpa-onnx-tts",
                "reason": "engine_not_installed",
                "message": "pip install -r requirements-tts.txt (dev/CI only)",
                "model_dir": str(path),
            }
        if not ready:
            return {
                "ok": False,
                "engine": "sherpa-onnx-tts",
                "reason": reason,
                "message": "Place Piper model under model_dir (dev/CI only)",
                "model_dir": str(path),
            }
        return {
            "ok": True,
            "engine": "sherpa-onnx-tts",
            "profile": manifest.get("id", path.name),
            "model_dir": str(path),
            "message": "sherpa Piper ready (dev/CI only — not Chat Pro product path)",
        }

    if engine_name == "cosyvoice2":
        if not path.is_dir():
            return {
                "ok": False,
                "engine": "cosyvoice2",
                "reason": "model_dir_missing",
                "message": "Import voice expansion model pack (CosyVoice2-0.5B)",
                "model_dir": str(path),
            }
        if not (path / "MANIFEST.json").is_file():
            return {
                "ok": False,
                "engine": "cosyvoice2",
                "reason": "manifest_missing",
                "message": "Import oclive-tts-cosyvoice2-0.5b-zh model pack",
                "model_dir": str(path),
            }
        base = _sidecar_base(manifest=manifest, sidecar_endpoint=sidecar_endpoint)
        health = _http_json(f"{base}/health", timeout=3.0)
        if health.get("ok"):
            return {
                "ok": True,
                "engine": "cosyvoice2",
                "profile": manifest.get("id", path.name),
                "model_dir": str(path),
                "sidecar_endpoint": base,
                "warmed": health.get("warmed", False),
                "message": health.get("message", "CosyVoice2 sidecar ready"),
            }
        return {
            "ok": False,
            "engine": "cosyvoice2",
            "reason": health.get("reason") or "sidecar_not_ready",
            "message": health.get("message")
            or "Start voice expansion sidecar and import CosyVoice2 weights",
            "model_dir": str(path),
            "sidecar_endpoint": base,
        }

    return {
        "ok": False,
        "engine": engine_name,
        "reason": "unsupported_engine",
        "message": f"Unknown TTS engine: {engine_name}",
        "model_dir": str(path),
    }


def warm_engine(
    *,
    model_dir: str | Path,
    sidecar_endpoint: str | None = None,
    engine: str | None = None,
    prime: bool = True,
) -> dict[str, Any]:
    path = Path(model_dir)
    manifest = _load_manifest(path)
    engine_name = engine or manifest.get("engine") or "cosyvoice2"
    if engine_name != "cosyvoice2":
        return {"ok": False, "reason": "unsupported_engine", "engine": engine_name}
    base = _sidecar_base(manifest=manifest, sidecar_endpoint=sidecar_endpoint)
    result = _http_json(
        f"{base}/warm",
        {"model_dir": str(path)},
        method="POST",
        timeout=300.0,
    )
    if not result.get("ok") or not prime:
        return result
    prime_result = _http_json(
        f"{base}/synthesize",
        {"text": "好", "emo_text": "平静"},
        method="POST",
        timeout=600.0,
    )
    result["primed"] = bool(prime_result.get("ok"))
    result["prime_reason"] = prime_result.get("reason", "")
    result["prime_elapsed_ms"] = prime_result.get("elapsed_ms")
    return result


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
    """Dev/CI Piper path — not used by Chat Pro product profiles."""
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
        "engine": "sherpa-onnx-tts",
    }


def _synthesize_cosyvoice2(
    *,
    path: Path,
    cleaned: str,
    speed: float,
    manifest: dict[str, Any],
    directive: dict[str, Any] | None,
    sidecar_endpoint: str | None,
) -> dict[str, Any]:
    base = _sidecar_base(manifest=manifest, sidecar_endpoint=sidecar_endpoint)
    d = directive or {}
    payload = {
        "text": cleaned,
        "emo_text": str(d.get("emo_text") or ""),
        "ref_audio": str(d.get("ref_audio") or ""),
        "ref_text": str(d.get("ref_text") or ""),
        "speed": speed,
    }
    result = _http_json(f"{base}/synthesize", payload, method="POST", timeout=600.0)
    if not result.get("ok"):
        return {"ok": False, "audio_base64": "", **result}
    return {
        "ok": True,
        "audio_base64": result.get("audio_base64", ""),
        "sample_rate": result.get("sample_rate", 22050),
        "profile": manifest.get("id", path.name),
        "engine": "cosyvoice2",
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


def _synthesize_cloud_openai(
    *,
    cleaned: str,
    speed: float,
    manifest: dict[str, Any],
    cloud_url: str | None,
    cloud_token: str | None,
    cloud_voice_id: str | None,
    cloud_model: str | None,
) -> dict[str, Any]:
    base = (cloud_url or "").strip().rstrip("/")
    if not base:
        return {
            "ok": False,
            "reason": "cloud_url_missing",
            "message": "Configure cloud TTS URL in voice expansion settings",
            "audio_base64": "",
            "engine": "cloud-tts-openai",
        }
    url = base if base.endswith("/audio/speech") else f"{base}/v1/audio/speech"
    headers: dict[str, str] = {"Content-Type": "application/json"}
    token = (cloud_token or "").strip()
    if token:
        headers["Authorization"] = f"Bearer {token}"
    body = {
        "model": (cloud_model or manifest.get("cloud_model") or "tts-1").strip(),
        "input": cleaned,
        "voice": (cloud_voice_id or manifest.get("voice") or "alloy").strip(),
        "speed": speed,
    }
    data = json.dumps(body).encode("utf-8")
    req = urlrequest.Request(url, data=data, headers=headers, method="POST")
    try:
        with urlrequest.urlopen(req, timeout=120.0) as resp:
            audio = resp.read()
    except Exception as exc:  # noqa: BLE001
        return {
            "ok": False,
            "reason": "cloud_tts_failed",
            "message": str(exc),
            "audio_base64": "",
            "engine": "cloud-tts-openai",
        }
    if not audio:
        return {"ok": False, "reason": "cloud_tts_empty", "audio_base64": ""}
    mime = "audio/mpeg"
    return {
        "ok": True,
        "audio_base64": base64.b64encode(audio).decode("ascii"),
        "sample_rate": 24000,
        "profile": manifest.get("id", "cloud-tts-openai"),
        "engine": "cloud-tts-openai",
        "audio_mime": mime,
    }


def synthesize_text(
    *,
    model_dir: str | Path,
    text: str,
    speed: float | None = None,
    directive: dict[str, Any] | None = None,
    engine: str | None = None,
    voice: str | None = None,
    sidecar_endpoint: str | None = None,
    cloud_url: str | None = None,
    cloud_token: str | None = None,
    cloud_voice_id: str | None = None,
    cloud_model: str | None = None,
) -> dict[str, Any]:
    cleaned = (text or "").strip()
    if not cleaned:
        return {"ok": False, "reason": "empty_text", "audio_base64": ""}

    path = Path(model_dir)
    manifest = _load_manifest(path)
    engine_name = engine or manifest.get("engine") or "cosyvoice2"
    effective_speed = float(speed if speed is not None else (directive or {}).get("speed", 1.0))
    effective_speed = max(0.5, min(2.0, effective_speed))

    if engine_name == "edge-tts":
        return _synthesize_edge_tts(
            cleaned=cleaned,
            speed=effective_speed,
            voice=voice,
            manifest=manifest,
        )
    if engine_name == "cloud-tts-openai":
        return _synthesize_cloud_openai(
            cleaned=cleaned,
            speed=effective_speed,
            manifest=manifest,
            cloud_url=cloud_url,
            cloud_token=cloud_token,
            cloud_voice_id=cloud_voice_id,
            cloud_model=cloud_model,
        )
    if engine_name == "cosyvoice2":
        probe = probe_engine(path, engine=engine_name, sidecar_endpoint=sidecar_endpoint)
        if not probe.get("ok"):
            return {"ok": False, "audio_base64": "", **probe}
        return _synthesize_cosyvoice2(
            path=path,
            cleaned=cleaned,
            speed=effective_speed,
            manifest=manifest,
            directive=directive,
            sidecar_endpoint=sidecar_endpoint or probe.get("sidecar_endpoint"),
        )
    if engine_name == "sherpa-onnx-tts":
        probe = probe_engine(path, engine=engine_name)
        if not probe.get("ok"):
            return {"ok": False, "audio_base64": "", **probe}
        return _synthesize_sherpa(
            path=path,
            cleaned=cleaned,
            speed=effective_speed,
            manifest=manifest,
        )

    probe = probe_engine(path, engine=engine_name, sidecar_endpoint=sidecar_endpoint)
    return {"ok": False, "audio_base64": "", **probe}
