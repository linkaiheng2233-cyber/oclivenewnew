"""GPT-SoVITS HTTP API adapter (user-local · port 9880 default)."""

from __future__ import annotations

from typing import Any
from urllib.parse import urlencode

from tts.engines._http import http_audio, http_json, sidecar_base
from tts.engines.base import skipped_warm


class GptSovitsHttpEngine:
    engine_id = "gpt-sovits-http"
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
            default_port=9880,
        )
        health = http_json(f"{base}/", timeout=3.0)
        if health.get("ok") is False and health.get("reason") == "http_error":
            # GSVI root may not be JSON — try a lightweight GET
            try:
                from urllib import request as urlrequest

                with urlrequest.urlopen(f"{base}/", timeout=3.0) as resp:
                    if resp.status < 500:
                        return {
                            "ok": True,
                            "engine": self.engine_id,
                            "sidecar_endpoint": base,
                            "supports_stream": False,
                            "supports_warm": False,
                            "message": "GPT-SoVITS HTTP reachable (user-local)",
                            "model_dir": model_dir,
                        }
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
            "message": "GPT-SoVITS HTTP ready (user-local · voice source is your responsibility)",
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
        **kwargs: Any,
    ) -> dict[str, Any]:
        base = sidecar_base(
            manifest=manifest,
            sidecar_endpoint=sidecar_endpoint,
            default_port=9880,
        )
        d = directive or {}
        ref_audio = str(d.get("ref_audio") or manifest.get("default_ref_audio") or "").strip()
        params: dict[str, Any] = {
            "text": text,
            "text_language": manifest.get("text_language", "zh"),
            "speed": speed,
        }
        if ref_audio:
            params["refer_wav_path"] = ref_audio
            ref_text = str(d.get("ref_text") or manifest.get("ref_text") or "").strip()
            if ref_text:
                params["prompt_text"] = ref_text
                params["prompt_language"] = manifest.get("prompt_language", "zh")
        synthesize_path = manifest.get("synthesize_path", "/tts")
        # GSVI v2 HTTP: GET /tts or POST depending on deployment; try GET first
        query = urlencode({k: v for k, v in params.items() if v is not None})
        result = http_audio(f"{base}{synthesize_path}?{query}", method="GET", timeout=300.0)
        if not result.get("ok"):
            result = http_audio(f"{base}{synthesize_path}", payload=params, method="POST", timeout=300.0)
        if not result.get("ok") and synthesize_path == "/tts":
            # Some GSVI builds expose synthesis at root with query params
            result = http_audio(f"{base}/?{query}", method="GET", timeout=300.0)
        if result.get("ok"):
            result["engine"] = self.engine_id
            result["profile"] = manifest.get("id", "gpt-sovits-http")
        return result
