"""Fish Speech HTTP adapter (OpenAI-like or custom infer endpoint)."""

from __future__ import annotations

from typing import Any

from tts.engines._http import http_audio, http_json, sidecar_base
from tts.engines.base import skipped_warm


class FishSpeechHttpEngine:
    engine_id = "fish-speech-http"
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
        base = sidecar_base(
            manifest=manifest,
            sidecar_endpoint=sidecar_endpoint,
            default_port=9881,
        )
        health = http_json(f"{base}/health", timeout=3.0)
        if health.get("ok") is False:
            try:
                from urllib import request as urlrequest

                with urlrequest.urlopen(f"{base}/v1/models", timeout=3.0) as resp:
                    if resp.status < 500:
                        health = {"ok": True}
            except Exception as exc:  # noqa: BLE001
                return {
                    "ok": False,
                    "engine": self.engine_id,
                    "reason": "endpoint_unreachable",
                    "message": str(exc),
                    "sidecar_endpoint": base,
                    "model_dir": model_dir,
                }
        return {
            "ok": True,
            "engine": self.engine_id,
            "sidecar_endpoint": base,
            "supports_stream": False,
            "supports_warm": False,
            "message": "Fish Speech HTTP ready (user-local)",
            "model_dir": model_dir,
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
        voice: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        base = sidecar_base(
            manifest=manifest,
            sidecar_endpoint=sidecar_endpoint,
            default_port=9881,
        )
        d = directive or {}
        api_style = manifest.get("api_style", "openai-speech-v1")
        if api_style == "openai-speech-v1":
            payload = {
                "input": text,
                "voice": voice or manifest.get("voice") or "default",
                "speed": speed,
            }
            ref_audio = str(d.get("ref_audio") or "").strip()
            if ref_audio:
                payload["reference_audio"] = ref_audio
            result = http_audio(f"{base}/v1/audio/speech", payload=payload, timeout=300.0)
        else:
            payload = {
                "text": text,
                "speed": speed,
                "reference_audio": str(d.get("ref_audio") or ""),
            }
            path = manifest.get("synthesize_path", "/v1/tts")
            result = http_audio(f"{base}{path}", payload=payload, timeout=300.0)
        if result.get("ok"):
            result["engine"] = self.engine_id
            result["profile"] = manifest.get("id", "fish-speech-http")
        return result
