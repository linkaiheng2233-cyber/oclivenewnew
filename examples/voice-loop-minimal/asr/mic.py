"""Microphone capture helpers for voice-loop-minimal."""

from __future__ import annotations

import struct
import wave
from io import BytesIO
from pathlib import Path


def record_seconds(seconds: float = 3.0, sample_rate: int = 16000) -> bytes:
    import numpy as np
    import sounddevice as sd

    frames = int(seconds * sample_rate)
    audio = sd.rec(frames, samplerate=sample_rate, channels=1, dtype="float32")
    sd.wait()
    pcm = (np.clip(audio[:, 0], -1.0, 1.0) * 32767).astype("int16")
    buf = BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm.tobytes())
    return buf.getvalue()


def save_wav(path: Path, wav_bytes: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(wav_bytes)
