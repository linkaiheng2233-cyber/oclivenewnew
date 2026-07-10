"""Optional compressed-audio decode (ffmpeg) when browser sends WebM without WAV prep."""

from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path


def _ffmpeg_path() -> str | None:
    return shutil.which("ffmpeg")


def is_compressed_container(raw: bytes) -> bool:
    if len(raw) < 4:
        return False
    if raw[:4] == b"RIFF":
        return False
    # EBML (WebM/Matroska) or Ogg
    if raw[:4] == b"\x1aE\xdf\xa3" or raw[:4] == b"OggS":
        return True
    # ISO BMFF / MP4 audio
    if len(raw) >= 12 and raw[4:8] == b"ftyp":
        return True
    return False


def decode_to_wav_bytes(raw: bytes, sample_rate: int = 16000) -> bytes | None:
    """Return mono PCM WAV bytes via ffmpeg, or None if ffmpeg unavailable."""
    ffmpeg = _ffmpeg_path()
    if not ffmpeg:
        return None
    with tempfile.TemporaryDirectory(prefix="oclive-asr-") as tmp:
        src = Path(tmp) / "input.bin"
        dst = Path(tmp) / "out.wav"
        src.write_bytes(raw)
        proc = subprocess.run(
            [
                ffmpeg,
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-i",
                str(src),
                "-ac",
                "1",
                "-ar",
                str(sample_rate),
                "-f",
                "wav",
                str(dst),
            ],
            capture_output=True,
            check=False,
        )
        if proc.returncode != 0 or not dst.is_file():
            return None
        return dst.read_bytes()
