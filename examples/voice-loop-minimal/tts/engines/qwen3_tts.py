"""Qwen3-TTS HTTP adapter (OpenAI-compatible /v1/audio/speech).

Spike (VX-8): community servers (qwen3-tts-server, Qwen3-TTS-API) expose
POST /v1/audio/speech with input, voice, language, instruct.
"""

from __future__ import annotations

from typing import Any

from tts.engines._http import http_audio, http_json, sidecar_base
from tts.engines.base import skipped_warm


class Qwen3TtsHttpEngine:
    engine_id = "qwen3-tts-http"
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
            default_port=8080,
        )
        health = http_json(f"{base}/health", timeout=3.0)
        if health.get("ok") is False:
            try:
                from urllib import request as urlrequest

                with urlrequest.urlopen(f"{base}/health", timeout=3.0) as resp:
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
            "message": "Qwen3-TTS HTTP ready (OpenAI-compatible /v1/audio/speech)",
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
            default_port=8080,
        )
        d = directive or {}
        emo_text = str(d.get("emo_text") or "").strip()
        ref_audio = str(d.get("ref_audio") or "").strip()
        voice_name = voice or manifest.get("voice") or "Vivian"
        language = manifest.get("language", "Chinese")
        speech_path = manifest.get("speech_path", "/v1/audio/speech")
        clone_path = manifest.get("clone_path", "/v1/audio/speech/clone")

        if ref_audio:
            # Voice clone endpoint (common in Qwen3-TTS-API)
            import json
            from urllib import error as urlerror
            from urllib import request as urlrequest

            boundary = "----ocliveqwen3"
            body_parts: list[bytes] = []
            for key, val in (
                ("input", text),
                ("language", language),
                ("ref_text", str(d.get("ref_text") or "")),
            ):
                if val:
                    body_parts.append(
                        f"--{boundary}\r\nContent-Disposition: form-data; name=\"{key}\"\r\n\r\n{val}\r\n".encode()
                    )
            with open(ref_audio, "rb") as f:
                audio_bytes = f.read()
            body_parts.append(
                (
                    f"--{boundary}\r\nContent-Disposition: form-data; name=\"voice_file\"; "
                    f'filename="ref.wav"\r\nContent-Type: audio/wav\r\n\r\n'
                ).encode()
                + audio_bytes
                + f"\r\n--{boundary}--\r\n".encode()
            )
            data = b"".join(body_parts)
            req = urlrequest.Request(
                f"{base}{clone_path}",
                data=data,
                headers={"Content-Type": f"multipart/form-data; boundary={boundary}"},
                method="POST",
            )
            try:
                with urlrequest.urlopen(req, timeout=300.0) as resp:
                    audio = resp.read()
                if audio:
                    import base64

                    return {
                        "ok": True,
                        "audio_base64": base64.b64encode(audio).decode("ascii"),
                        "sample_rate": 24000,
                        "engine": self.engine_id,
                        "profile": manifest.get("id", "qwen3-tts-http"),
                        "audio_mime": "audio/wav",
                    }
            except urlerror.HTTPError as exc:
                if exc.code not in (404, 405):
                    return {
                        "ok": False,
                        "reason": "qwen3_clone_failed",
                        "message": str(exc),
                        "audio_base64": "",
                        "engine": self.engine_id,
                    }
            except Exception as exc:  # noqa: BLE001
                return {
                    "ok": False,
                    "reason": "qwen3_clone_failed",
                    "message": str(exc),
                    "audio_base64": "",
                    "engine": self.engine_id,
                }

        payload: dict[str, Any] = {
            "input": text,
            "voice": voice_name,
            "language": language,
            "speed": speed,
            "response_format": manifest.get("response_format", "wav"),
        }
        if emo_text:
            payload["instruct"] = emo_text
        result = http_audio(f"{base}{speech_path}", payload=payload, timeout=300.0)
        if result.get("ok"):
            result["engine"] = self.engine_id
            result["profile"] = manifest.get("id", "qwen3-tts-http")
        return result
