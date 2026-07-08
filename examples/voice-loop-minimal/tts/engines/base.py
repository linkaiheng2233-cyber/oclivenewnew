"""TtsEngine protocol — engine-agnostic probe / synthesize / warm contract."""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class TtsEngine(Protocol):
    """Official or user-pack TTS adapter contract (VX-7)."""

    engine_id: str
    supports_stream: bool
    supports_warm: bool

    def probe(
        self,
        model_dir: str,
        *,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        """Return { ok, engine, message?, reason?, ... }."""

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
        """Return { ok, audio_base64?, sample_rate?, engine, ... }."""

    def warm(
        self,
        *,
        model_dir: str,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        prime: bool = True,
        **kwargs: Any,
    ) -> dict[str, Any]:
        """Warm model weights when supported; else { ok: true, skipped: true }."""


def skipped_warm(engine_id: str) -> dict[str, Any]:
    return {
        "ok": True,
        "skipped": True,
        "engine": engine_id,
        "message": f"{engine_id} does not require warm",
    }
