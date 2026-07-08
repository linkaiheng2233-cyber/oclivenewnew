"""TtsEngineRegistry — register official adapters and dispatch probe/synth/warm."""

from __future__ import annotations

from typing import Any

from tts.engines.base import TtsEngine
from tts.engines.cloud_openai import CloudOpenaiEngine
from tts.engines.cosyvoice2 import Cosyvoice2Engine, ensure_cosyvoice2_warmed
from tts.engines.edge_tts import EdgeTtsEngine
from tts.engines.fish_speech import FishSpeechHttpEngine
from tts.engines.generic_http import GenericHttpAdapterEngine
from tts.engines.gpt_sovits import GptSovitsHttpEngine
from tts.engines.indextts import IndexTtsHttpEngine
from tts.engines.qwen3_tts import Qwen3TtsHttpEngine
from tts.engines.sherpa import SherpaOnnxEngine
from tts.engines._http import load_manifest


class TtsEngineRegistry:
    def __init__(self) -> None:
        self._engines: dict[str, TtsEngine] = {}

    def register(self, engine: TtsEngine) -> None:
        self._engines[engine.engine_id] = engine

    def get(self, engine_id: str) -> TtsEngine | None:
        return self._engines.get(engine_id)

    def list_engine_ids(self) -> list[str]:
        return sorted(self._engines.keys())

    def probe(
        self,
        model_dir: str,
        *,
        engine: str | None = None,
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        manifest = load_manifest(model_dir)
        engine_name = engine or manifest.get("engine") or "cosyvoice2"
        adapter = self.get(engine_name)
        if not adapter:
            return {
                "ok": False,
                "engine": engine_name,
                "reason": "unsupported_engine",
                "message": f"Unknown TTS engine: {engine_name}",
                "model_dir": model_dir,
            }
        result = adapter.probe(
            model_dir,
            manifest=manifest,
            sidecar_endpoint=sidecar_endpoint,
            **kwargs,
        )
        result.setdefault("supports_stream", adapter.supports_stream)
        result.setdefault("supports_warm", adapter.supports_warm)
        return result

    def warm(
        self,
        *,
        model_dir: str,
        engine: str | None = None,
        sidecar_endpoint: str | None = None,
        prime: bool = True,
        **kwargs: Any,
    ) -> dict[str, Any]:
        manifest = load_manifest(model_dir)
        engine_name = engine or manifest.get("engine") or "cosyvoice2"
        adapter = self.get(engine_name)
        if not adapter:
            return {"ok": False, "reason": "unsupported_engine", "engine": engine_name}
        if not adapter.supports_warm:
            from tts.engines.base import skipped_warm

            return skipped_warm(engine_name)
        return adapter.warm(
            model_dir=model_dir,
            manifest=manifest,
            sidecar_endpoint=sidecar_endpoint,
            prime=prime,
            **kwargs,
        )

    def synthesize(
        self,
        *,
        model_dir: str,
        text: str,
        speed: float,
        directive: dict[str, Any] | None = None,
        engine: str | None = None,
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        cleaned = (text or "").strip()
        if not cleaned:
            return {"ok": False, "reason": "empty_text", "audio_base64": ""}

        manifest = load_manifest(model_dir)
        engine_name = engine or manifest.get("engine") or "cosyvoice2"
        adapter = self.get(engine_name)
        if not adapter:
            probe = self.probe(model_dir, engine=engine_name, sidecar_endpoint=sidecar_endpoint)
            return {"ok": False, "audio_base64": "", **probe}

        if engine_name == "cosyvoice2":
            ready = ensure_cosyvoice2_warmed(
                model_dir=model_dir,
                manifest=manifest,
                sidecar_endpoint=sidecar_endpoint,
            )
            if not ready.get("ok"):
                return {"ok": False, "audio_base64": "", **ready}
            sidecar_endpoint = sidecar_endpoint or ready.get("sidecar_endpoint")

        if engine_name == "sherpa-onnx-tts":
            probe = self.probe(model_dir, engine=engine_name)
            if not probe.get("ok"):
                return {"ok": False, "audio_base64": "", **probe}

        return adapter.synthesize(
            model_dir=model_dir,
            manifest=manifest,
            text=cleaned,
            speed=speed,
            directive=directive,
            sidecar_endpoint=sidecar_endpoint,
            **kwargs,
        )


_REGISTRY: TtsEngineRegistry | None = None


def get_registry() -> TtsEngineRegistry:
    global _REGISTRY  # noqa: PLW0603
    if _REGISTRY is None:
        reg = TtsEngineRegistry()
        reg.register(Cosyvoice2Engine())
        reg.register(EdgeTtsEngine())
        reg.register(CloudOpenaiEngine())
        reg.register(SherpaOnnxEngine())
        reg.register(GptSovitsHttpEngine())
        reg.register(Qwen3TtsHttpEngine())
        reg.register(FishSpeechHttpEngine())
        reg.register(IndexTtsHttpEngine())
        reg.register(GenericHttpAdapterEngine())
        _REGISTRY = reg
    return _REGISTRY


def probe_engine(
    model_dir: str,
    *,
    engine: str | None = None,
    sidecar_endpoint: str | None = None,
    **kwargs: Any,
) -> dict[str, Any]:
    return get_registry().probe(
        model_dir,
        engine=engine,
        sidecar_endpoint=sidecar_endpoint,
        **kwargs,
    )


def warm_engine(
    *,
    model_dir: str,
    sidecar_endpoint: str | None = None,
    engine: str | None = None,
    prime: bool = True,
    **kwargs: Any,
) -> dict[str, Any]:
    return get_registry().warm(
        model_dir=model_dir,
        engine=engine,
        sidecar_endpoint=sidecar_endpoint,
        prime=prime,
        **kwargs,
    )


def synthesize_text(
    *,
    model_dir: str,
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
    effective_speed = float(speed if speed is not None else (directive or {}).get("speed", 1.0))
    effective_speed = max(0.5, min(2.0, effective_speed))
    return get_registry().synthesize(
        model_dir=model_dir,
        text=text,
        speed=effective_speed,
        directive=directive,
        engine=engine,
        sidecar_endpoint=sidecar_endpoint,
        voice=voice,
        cloud_url=cloud_url,
        cloud_token=cloud_token,
        cloud_voice_id=cloud_voice_id,
        cloud_model=cloud_model,
    )
