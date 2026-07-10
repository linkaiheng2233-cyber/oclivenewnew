"""Fish Speech HTTP adapter (OpenAI-like or custom infer endpoint)."""

from __future__ import annotations

import base64
from pathlib import Path
from typing import Any

from tts.engines._http import http_audio, http_json, sidecar_base
from tts.engines.base import skipped_warm


class FishSpeechHttpEngine:
    engine_id = "fish-speech-http"
    supports_stream = False
    supports_warm = False

    @staticmethod
    def _probe_paths(manifest: dict[str, Any]) -> list[str]:
        custom = str(manifest.get("probe_path") or "").strip()
        paths = [custom] if custom else []
        for path in ("/v1/health", "/health", "/v1/models"):
            if path not in paths:
                paths.append(path)
        return paths

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
        last_error = ""
        for path in self._probe_paths(manifest):
            health = http_json(f"{base}{path}", timeout=3.0)
            if health.get("ok"):
                break
            last_error = str(health.get("message") or health.get("reason") or last_error)
            try:
                from urllib import request as urlrequest

                with urlrequest.urlopen(f"{base}{path}", timeout=3.0) as resp:
                    if resp.status < 500:
                        health = {"ok": True}
                        break
            except Exception as exc:  # noqa: BLE001
                last_error = str(exc)
        else:
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "endpoint_unreachable",
                "message": last_error or "Fish Speech HTTP probe failed",
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

    @staticmethod
    def _fish_references(directive: dict[str, Any]) -> list[dict[str, str]]:
        ref_audio = str(directive.get("ref_audio") or "").strip()
        if not ref_audio or not Path(ref_audio).is_file():
            return []
        ref_text = str(directive.get("ref_text") or "").strip() or "参考音频"
        audio_b64 = base64.b64encode(Path(ref_audio).read_bytes()).decode("ascii")
        return [{"audio": audio_b64, "text": ref_text}]

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
        api_style = manifest.get("api_style", "native-v1-tts")
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
            references = self._fish_references(d)
            payload = {
                "text": text,
                "format": "wav",
                "references": references,
            }
            if references:
                payload["reference_id"] = None
            path = manifest.get("synthesize_path", "/v1/tts")
            result = http_audio(f"{base}{path}", payload=payload, timeout=300.0)
        if result.get("ok"):
            result["engine"] = self.engine_id
            result["profile"] = manifest.get("id", "fish-speech-http")
        return result
