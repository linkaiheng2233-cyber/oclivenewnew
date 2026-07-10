"""OpenAI-compatible cloud TTS adapter."""

from __future__ import annotations

import base64
import json
from typing import Any
from urllib import request as urlrequest

from tts.engines.base import skipped_warm


class CloudOpenaiEngine:
    engine_id = "cloud-tts-openai"
    supports_stream = False
    supports_warm = False

    def probe(
        self,
        model_dir: str,
        *,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        return {
            "ok": True,
            "engine": self.engine_id,
            "profile": manifest.get("id", "cloud-tts-openai"),
            "model_dir": model_dir,
            "supports_stream": False,
            "supports_warm": False,
            "message": "cloud TTS ready when URL/token configured in settings",
        }

    def warm(self, **kwargs: Any) -> dict[str, Any]:
        return skipped_warm(self.engine_id)

    def synthesize(
        self,
        *,
        model_dir: str,
        manifest: dict[str, Any],
        text: str,
        speed: float,
        directive: dict[str, Any] | None,
        sidecar_endpoint: str | None = None,
        cloud_url: str | None = None,
        cloud_token: str | None = None,
        cloud_voice_id: str | None = None,
        cloud_model: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        base = (cloud_url or "").strip().rstrip("/")
        if not base:
            return {
                "ok": False,
                "reason": "cloud_url_missing",
                "message": "Configure cloud TTS URL in voice expansion settings",
                "audio_base64": "",
                "engine": self.engine_id,
            }
        url = base if base.endswith("/audio/speech") else f"{base}/v1/audio/speech"
        headers: dict[str, str] = {"Content-Type": "application/json"}
        token = (cloud_token or "").strip()
        if token:
            headers["Authorization"] = f"Bearer {token}"
        body = {
            "model": (cloud_model or manifest.get("cloud_model") or "tts-1").strip(),
            "input": text,
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
                "engine": self.engine_id,
            }
        if not audio:
            return {"ok": False, "reason": "cloud_tts_empty", "audio_base64": ""}
        return {
            "ok": True,
            "audio_base64": base64.b64encode(audio).decode("ascii"),
            "sample_rate": 24000,
            "profile": manifest.get("id", "cloud-tts-openai"),
            "engine": self.engine_id,
            "audio_mime": "audio/mpeg",
        }
