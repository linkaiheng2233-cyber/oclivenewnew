"""User-imported generic HTTP TTS adapter pack (VX-9)."""

from __future__ import annotations

import base64
import json
import re
from typing import Any

from tts.engines._http import http_audio, http_json, load_adapter_pack, sidecar_base
from tts.engines.base import skipped_warm


def _resolve_template(value: Any, ctx: dict[str, Any]) -> Any:
    if not isinstance(value, str):
        return value
    out = value
    for key, val in ctx.items():
        out = out.replace(f"{{{key}}}", str(val))
    return out


def _build_payload(template: dict[str, Any], ctx: dict[str, Any]) -> dict[str, Any]:
    payload: dict[str, Any] = {}
    for key, val in template.items():
        if isinstance(val, dict):
            payload[key] = _build_payload(val, ctx)
        elif isinstance(val, list):
            payload[key] = [_resolve_template(item, ctx) for item in val]
        else:
            payload[key] = _resolve_template(val, ctx)
    return payload


class GenericHttpAdapterEngine:
    engine_id = "generic-http-adapter"
    supports_stream = False
    supports_warm = False

    def _pack(self, model_dir: str, manifest: dict[str, Any]) -> dict[str, Any] | None:
        pack = load_adapter_pack(model_dir)
        if pack:
            return pack
        return manifest.get("adapter_pack") if isinstance(manifest.get("adapter_pack"), dict) else None

    def probe(
        self,
        model_dir: str,
        *,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        pack = self._pack(model_dir, manifest)
        if not pack:
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "adapter_pack_missing",
                "message": "Import tts_adapter_pack.json via voice.import_tts_adapter",
                "model_dir": model_dir,
            }
        base = (sidecar_endpoint or pack.get("base_url") or "").strip().rstrip("/")
        if not base:
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "base_url_missing",
                "message": "Set base_url in adapter pack or local HTTP endpoint",
                "model_dir": model_dir,
            }
        probe_path = pack.get("probe_path", "/health")
        health = http_json(f"{base}{probe_path}", timeout=3.0)
        if health.get("ok") is False and health.get("reason") not in {"http_unreachable"}:
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
                    "adapter_id": pack.get("adapter_id"),
                    "sidecar_endpoint": base,
                    "model_dir": model_dir,
                }
        return {
            "ok": True,
            "engine": self.engine_id,
            "adapter_id": pack.get("adapter_id"),
            "api_style": pack.get("api_style"),
            "sidecar_endpoint": base,
            "supports_stream": False,
            "supports_warm": False,
            "message": f"Generic HTTP adapter {pack.get('adapter_id', '')} ready",
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
        pack = self._pack(model_dir, manifest)
        if not pack:
            return {
                "ok": False,
                "reason": "adapter_pack_missing",
                "audio_base64": "",
                "engine": self.engine_id,
            }
        base = (sidecar_endpoint or pack.get("base_url") or "").strip().rstrip("/")
        if not base:
            return {
                "ok": False,
                "reason": "base_url_missing",
                "audio_base64": "",
                "engine": self.engine_id,
            }
        d = directive or {}
        field_map: dict[str, str] = pack.get("field_map") or {}
        ctx = {
            "text": text,
            "speed": speed,
            "emo_text": str(d.get("emo_text") or ""),
            "ref_audio": str(d.get("ref_audio") or ""),
            "ref_text": str(d.get("ref_text") or ""),
            "emotion_tag": str(d.get("emotion_tag") or ""),
        }
        for src, dst in field_map.items():
            if src in ctx and dst not in ctx:
                ctx[dst] = ctx[src]

        api_style = pack.get("api_style", "custom-json-template")
        synth_path = pack.get("synthesize_path", "/v1/audio/speech")
        method = str(pack.get("method", "POST")).upper()

        if api_style == "openai-speech-v1":
            payload = {
                "input": text,
                "voice": pack.get("default_voice", "alloy"),
                "speed": speed,
            }
            if ctx.get("emo_text"):
                payload["instruct"] = ctx["emo_text"]
            result = http_audio(f"{base}{synth_path}", payload=payload, method=method, timeout=300.0)
        elif api_style == "gpt-sovits-v2":
            params = {
                "text": text,
                "text_language": pack.get("text_language", "zh"),
                "speed": speed,
            }
            if ctx.get("ref_audio"):
                params["refer_wav_path"] = ctx["ref_audio"]
            if ctx.get("ref_text"):
                params["prompt_text"] = ctx["ref_text"]
            query = "&".join(f"{k}={v}" for k, v in params.items() if v)
            result = http_audio(f"{base}{synth_path}?{query}", method="GET", timeout=300.0)
        elif api_style == "cosyvoice2-synthesize":
            payload = {
                "text": text,
                "emo_text": ctx.get("emo_text") or "用自然平静的语气",
                "ref_audio": ctx.get("ref_audio", ""),
                "ref_text": ctx.get("ref_text", ""),
                "speed": speed,
            }
            result = http_audio(f"{base}{synth_path}", payload=payload, method=method, timeout=600.0)
        else:
            template = pack.get("request_template") or {"text": "{text}", "speed": "{speed}"}
            payload = _build_payload(template, ctx)
            result = http_audio(f"{base}{synth_path}", payload=payload, method=method, timeout=300.0)

        if not result.get("ok"):
            return {**result, "audio_base64": "", "engine": self.engine_id}

        response_field = pack.get("response_audio_field")
        if response_field and result.get("audio_base64") is None:
            # JSON response with nested base64 field
            json_result = http_json(
                f"{base}{synth_path}",
                payload if api_style == "custom-json-template" else None,
                method=method,
                timeout=300.0,
            )
            if not json_result.get("ok", True) and json_result.get("reason"):
                return {**json_result, "audio_base64": "", "engine": self.engine_id}
            parts = response_field.split(".")
            node: Any = json_result
            for part in parts:
                if isinstance(node, dict):
                    node = node.get(part)
                else:
                    node = None
                    break
            if isinstance(node, str) and node:
                if re.match(r"^[A-Za-z0-9+/=]+$", node):
                    result = {
                        "ok": True,
                        "audio_base64": node,
                        "sample_rate": json_result.get("sample_rate", 24000),
                    }
                else:
                    result = {
                        "ok": True,
                        "audio_base64": base64.b64encode(node.encode()).decode("ascii"),
                        "sample_rate": 24000,
                    }

        if result.get("ok"):
            result["engine"] = self.engine_id
            result["adapter_id"] = pack.get("adapter_id")
        return result
