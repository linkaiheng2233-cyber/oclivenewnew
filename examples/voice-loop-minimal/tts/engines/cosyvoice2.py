"""CosyVoice2 bundled / local HTTP synthesize adapter."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from tts.engines._http import http_json, sidecar_base


class Cosyvoice2Engine:
    engine_id = "cosyvoice2"
    supports_stream = True
    supports_warm = True

    def probe(
        self,
        model_dir: str,
        *,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        path = Path(model_dir)
        if not path.is_dir():
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "model_dir_missing",
                "message": "Import voice expansion model pack (CosyVoice2-0.5B)",
                "model_dir": str(path),
            }
        if not (path / "MANIFEST.json").is_file():
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "manifest_missing",
                "message": "Import oclive-tts-cosyvoice2-0.5b-zh model pack",
                "model_dir": str(path),
            }
        base = sidecar_base(manifest=manifest, sidecar_endpoint=sidecar_endpoint)
        health = http_json(f"{base}/health", timeout=3.0)
        if health.get("ok"):
            return {
                "ok": True,
                "engine": self.engine_id,
                "profile": manifest.get("id", path.name),
                "model_dir": str(path),
                "sidecar_endpoint": base,
                "warmed": health.get("warmed", False),
                "supports_stream": True,
                "supports_warm": True,
                "message": health.get("message", "CosyVoice2 sidecar ready"),
            }
        return {
            "ok": False,
            "engine": self.engine_id,
            "reason": health.get("reason") or "sidecar_not_ready",
            "message": health.get("message")
            or "Start voice expansion sidecar and import CosyVoice2 weights",
            "model_dir": str(path),
            "sidecar_endpoint": base,
            "supports_stream": True,
            "supports_warm": True,
        }

    def warm(
        self,
        *,
        model_dir: str,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        prime: bool = True,
        **kwargs: Any,
    ) -> dict[str, Any]:
        base = sidecar_base(manifest=manifest, sidecar_endpoint=sidecar_endpoint)
        return http_json(
            f"{base}/warm",
            {"model_dir": str(model_dir), "prime": prime},
            method="POST",
            timeout=900.0,
        )

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
        path = Path(model_dir)
        base = sidecar_base(manifest=manifest, sidecar_endpoint=sidecar_endpoint)
        d = directive or {}
        emo_text = str(d.get("emo_text") or "").strip()
        ref_audio = str(d.get("ref_audio") or "").strip()
        if not emo_text and not ref_audio:
            emo_text = "用自然平静的语气"
        payload = {
            "text": text,
            "emo_text": emo_text,
            "ref_audio": ref_audio,
            "ref_text": str(d.get("ref_text") or ""),
            "speed": speed,
        }
        result = http_json(f"{base}/synthesize", payload, method="POST", timeout=600.0)
        if not result.get("ok"):
            return {"ok": False, "audio_base64": "", **result}
        return {
            "ok": True,
            "audio_base64": result.get("audio_base64", ""),
            "sample_rate": result.get("sample_rate", 22050),
            "profile": manifest.get("id", path.name),
            "engine": self.engine_id,
        }


def ensure_cosyvoice2_warmed(
    *,
    model_dir: str,
    manifest: dict[str, Any],
    sidecar_endpoint: str | None,
) -> dict[str, Any]:
    engine = Cosyvoice2Engine()
    probe = engine.probe(model_dir, manifest=manifest, sidecar_endpoint=sidecar_endpoint)
    if probe.get("ok"):
        return probe
    reason = str(probe.get("reason") or "")
    endpoint = sidecar_endpoint or probe.get("sidecar_endpoint")
    if reason != "not_warmed":
        return probe
    warm = engine.warm(
        model_dir=model_dir,
        manifest=manifest,
        sidecar_endpoint=endpoint,
        prime=False,
    )
    if not warm.get("ok"):
        return {
            "ok": False,
            "engine": "cosyvoice2",
            "reason": warm.get("reason") or "warm_failed",
            "message": warm.get("message") or "CosyVoice2 warm failed",
            "model_dir": str(model_dir),
            "sidecar_endpoint": endpoint,
        }
    return engine.probe(model_dir, manifest=manifest, sidecar_endpoint=endpoint)
