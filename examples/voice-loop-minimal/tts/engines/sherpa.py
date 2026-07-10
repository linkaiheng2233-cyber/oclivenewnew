"""sherpa-onnx Piper dev/CI adapter — not Chat Pro product path."""

from __future__ import annotations

import base64
import wave
from io import BytesIO
from pathlib import Path
from typing import Any

from tts.engines.base import skipped_warm


def _pcm_to_wav_base64(samples: list[float], sample_rate: int) -> str:
    import struct

    pcm = b"".join(struct.pack("<h", max(-32768, min(32767, int(s * 32767)))) for s in samples)
    buf = BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm)
    return base64.b64encode(buf.getvalue()).decode("ascii")


class SherpaOnnxEngine:
    engine_id = "sherpa-onnx-tts"
    supports_stream = False
    supports_warm = False

    def _model_ready(self, model_dir: Path) -> tuple[bool, str]:
        if not model_dir.is_dir():
            return False, "model_dir_missing"
        manifest = {}
        manifest_path = model_dir / "MANIFEST.json"
        if manifest_path.is_file():
            import json

            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        model_file = manifest.get("model_file", "model.onnx")
        tokens_file = manifest.get("tokens_file", "tokens.txt")
        missing = [
            name
            for name, fname in (("model", model_file), ("tokens", tokens_file))
            if not (model_dir / fname).is_file()
        ]
        if missing:
            return False, f"model_files_missing:{','.join(missing)}"
        return True, ""

    def probe(
        self,
        model_dir: str,
        *,
        manifest: dict[str, Any],
        sidecar_endpoint: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        path = Path(model_dir)
        ready, reason = self._model_ready(path)
        try:
            import sherpa_onnx  # noqa: F401
        except ImportError:
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": "engine_not_installed",
                "message": "pip install -r requirements-tts.txt (dev/CI only)",
                "model_dir": str(path),
            }
        if not ready:
            return {
                "ok": False,
                "engine": self.engine_id,
                "reason": reason,
                "message": "Place Piper model under model_dir (dev/CI only)",
                "model_dir": str(path),
            }
        return {
            "ok": True,
            "engine": self.engine_id,
            "profile": manifest.get("id", path.name),
            "model_dir": str(path),
            "supports_stream": False,
            "supports_warm": False,
            "message": "sherpa Piper ready (dev/CI only — not Chat Pro product path)",
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
        import sherpa_onnx

        path = Path(model_dir)
        tts_config = sherpa_onnx.OfflineTtsConfig(
            model=sherpa_onnx.OfflineTtsModelConfig(
                vits=sherpa_onnx.OfflineTtsVitsModelConfig(
                    model=str(path / manifest.get("model_file", "model.onnx")),
                    tokens=str(path / manifest.get("tokens_file", "tokens.txt")),
                    data_dir=str(path / manifest.get("data_dir", "espeak-ng-data"))
                    if (path / manifest.get("data_dir", "espeak-ng-data")).is_dir()
                    else "",
                    noise_scale=float(manifest.get("noise_scale", 0.667)),
                    noise_scale_w=float(manifest.get("noise_scale_w", 0.8)),
                    length_scale=float(manifest.get("length_scale", 1.0)),
                ),
                num_threads=int(manifest.get("num_threads", 2)),
            ),
            max_num_sentences=int(manifest.get("max_num_sentences", 2)),
        )
        if not tts_config.validate():
            return {
                "ok": False,
                "reason": "tts_config_invalid",
                "message": "Check model.onnx, tokens.txt, espeak-ng-data",
                "audio_base64": "",
            }
        tts = sherpa_onnx.OfflineTts(tts_config)
        base_speed = float(manifest.get("speed", 1.0))
        audio = tts.generate(
            text,
            sid=int(manifest.get("speaker_id", 0)),
            speed=max(0.5, min(2.0, base_speed * speed)),
        )
        audio_b64 = _pcm_to_wav_base64(list(audio.samples), audio.sample_rate)
        return {
            "ok": True,
            "audio_base64": audio_b64,
            "sample_rate": audio.sample_rate,
            "profile": manifest.get("id", path.name),
            "engine": self.engine_id,
        }
