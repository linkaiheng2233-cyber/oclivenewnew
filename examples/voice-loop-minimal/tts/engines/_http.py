"""Shared HTTP helpers for TTS adapters."""

from __future__ import annotations

import base64
import json
from typing import Any
from urllib import error as urlerror
from urllib import request as urlrequest


def http_json(
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
            body = resp.read()
            ctype = resp.headers.get("Content-Type", "")
            if "json" in ctype:
                return json.loads(body.decode("utf-8"))
            return {"ok": True, "audio_bytes": body, "audio_mime": ctype or "application/octet-stream"}
    except urlerror.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        try:
            parsed = json.loads(body)
            if isinstance(parsed, dict):
                return {"ok": False, "reason": "http_error", **parsed}
        except json.JSONDecodeError:
            pass
        return {"ok": False, "reason": "http_error", "message": body[:300]}
    except Exception as exc:  # noqa: BLE001
        return {"ok": False, "reason": "http_unreachable", "message": str(exc)}


def http_audio(
    url: str,
    payload: dict[str, Any] | None = None,
    *,
    method: str = "POST",
    headers: dict[str, str] | None = None,
    timeout: float = 120.0,
) -> dict[str, Any]:
    data = None
    req_headers = dict(headers or {})
    if payload is not None:
        if "Content-Type" not in req_headers:
            req_headers["Content-Type"] = "application/json; charset=utf-8"
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    req = urlrequest.Request(url, data=data, headers=req_headers, method=method)
    try:
        with urlrequest.urlopen(req, timeout=timeout) as resp:
            audio = resp.read()
            if not audio:
                return {"ok": False, "reason": "empty_audio", "audio_base64": ""}
            mime = resp.headers.get("Content-Type", "audio/wav").split(";")[0].strip()
            return {
                "ok": True,
                "audio_base64": base64.b64encode(audio).decode("ascii"),
                "audio_mime": mime,
                "sample_rate": 24000,
            }
    except Exception as exc:  # noqa: BLE001
        return {"ok": False, "reason": "http_failed", "message": str(exc), "audio_base64": ""}


def sidecar_base(
    *,
    manifest: dict[str, Any],
    sidecar_endpoint: str | None = None,
    default_port: int = 50000,
) -> str:
    if sidecar_endpoint and sidecar_endpoint.strip():
        return sidecar_endpoint.strip().rstrip("/")
    port = int(manifest.get("sidecar_port", default_port) or default_port)
    return f"http://127.0.0.1:{port}"


def load_manifest(model_dir: str) -> dict[str, Any]:
    from pathlib import Path

    manifest_path = Path(model_dir) / "MANIFEST.json"
    if manifest_path.is_file():
        return json.loads(manifest_path.read_text(encoding="utf-8"))
    return {}


def load_adapter_pack(model_dir: str) -> dict[str, Any] | None:
    from pathlib import Path

    for name in ("tts_adapter_pack.json", "voice_tts_adapter.json"):
        path = Path(model_dir) / name
        if path.is_file():
            return json.loads(path.read_text(encoding="utf-8"))
    return None
