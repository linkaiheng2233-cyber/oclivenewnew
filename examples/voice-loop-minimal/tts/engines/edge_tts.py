"""edge-tts online adapter."""

from __future__ import annotations

import asyncio
import base64
from typing import Any

from tts.engines.base import skipped_warm


class EdgeTtsEngine:
    engine_id = "edge-tts"
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
        try:
            import edge_tts  # noqa: F401
        except ImportError:
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "engine_not_installed",
                "message": "pip install edge-tts",
                "model_dir": model_dir,
            }
        return {
            "ok": True,
            "engine": self.engine_id,
            "profile": manifest.get("id", "edge-tts"),
            "model_dir": model_dir,
            "supports_stream": False,
            "supports_warm": False,
            "message": "edge-tts ready (online)",
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
        try:
            import edge_tts
        except ImportError:
            return {
                "ok": False,
                "reason": "engine_not_installed",
                "message": "pip install edge-tts",
                "audio_base64": "",
                "engine": self.engine_id,
            }

        voice_name = voice or manifest.get("voice") or "zh-CN-XiaoxiaoNeural"
        rate_pct = int((speed - 1.0) * 100)
        rate = f"{rate_pct:+d}%"

        async def _run() -> bytes:
            communicate = edge_tts.Communicate(text, voice_name, rate=rate)
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
                "engine": self.engine_id,
            }
        if not mp3:
            return {
                "ok": False,
                "reason": "edge_tts_empty",
                "audio_base64": "",
                "engine": self.engine_id,
            }
        return {
            "ok": True,
            "audio_base64": base64.b64encode(mp3).decode("ascii"),
            "sample_rate": 24000,
            "profile": manifest.get("id", voice_name),
            "engine": self.engine_id,
            "audio_mime": "audio/mpeg",
        }
