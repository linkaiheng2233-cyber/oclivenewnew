"""TTS engines: CosyVoice2 sidecar, edge-tts, cloud OpenAI-compatible, sherpa Piper (dev/CI).

Facade over ``tts.engines.registry`` (VX-7 engine registry).
"""

from tts.engines.registry import probe_engine, synthesize_text, warm_engine

__all__ = ["probe_engine", "synthesize_text", "warm_engine"]
