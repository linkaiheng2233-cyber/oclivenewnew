"""IndexTTS HTTP infer adapter."""

from __future__ import annotations

from typing import Any

from tts.engines._http import http_audio, http_json, sidecar_base
from tts.engines.base import skipped_warm


class IndexTtsHttpEngine:
    engine_id = "indextts-http"
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
            default_port=7860,
        )
        probe_path = manifest.get("probe_path", "/health")
        health = http_json(f"{base}{probe_path}", timeout=3.0)
        if health.get("ok") is False:
            try:
                from urllib import request as urlrequest

                with urlrequest.urlopen(f"{base}{probe_path}", timeout=3.0) as resp:
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
            "message": "IndexTTS HTTP ready (user-local)",
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
            default_port=7860,
        )
        d = directive or {}
        path = manifest.get("synthesize_path", "/infer")
        payload = {
            "text": text,
            "speed": speed,
            "emo_text": str(d.get("emo_text") or ""),
            "ref_audio": str(d.get("ref_audio") or ""),
            "ref_text": str(d.get("ref_text") or ""),
        }
        result = http_audio(f"{base}{path}", payload=payload, timeout=300.0)
        if result.get("ok"):
            result["engine"] = self.engine_id
            result["profile"] = manifest.get("id", "indextts-http")
        return result
