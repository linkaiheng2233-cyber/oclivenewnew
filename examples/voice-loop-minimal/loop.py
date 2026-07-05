#!/usr/bin/env python3
"""Minimal voice/chat loop: text or mic → POST /chat → reply out (optional TTS).

See README.md and human-docs/team/TRACK_VOICE_RECOGNITION.md.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

# Fixed session for multi-turn memory (do not regenerate each request).
DEFAULT_SESSION_ID = "00000000-0000-4000-8000-000000000001"


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def default_role_path() -> Path:
    env = os.environ.get("OCLIVE_ROLE_PATH", "").strip()
    if env:
        return Path(env)
    return repo_root() / "distros" / "chat-pro" / "roles" / "mumu"


def default_asr_model_dir() -> Path:
    env = os.environ.get("OCLIVE_ASR_MODEL_DIR", "").strip()
    if env:
        return Path(env)
    return Path(__file__).resolve().parent / "models" / "asr" / "sherpa-paraformer-zh-small"


def api_base() -> str:
    return os.environ.get("OCLIVE_API_BASE", "http://127.0.0.1:8420").rstrip("/")


def scene_id() -> str:
    return os.environ.get("OCLIVE_SCENE_ID", "default")


def session_id() -> str:
    return os.environ.get("OCLIVE_SESSION_ID", DEFAULT_SESSION_ID)


def health_ok() -> bool:
    req = urllib.request.Request(f"{api_base()}/health", method="GET")
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            body = json.loads(resp.read().decode("utf-8"))
            return bool(body.get("ok", True))
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError):
        return False


def post_chat(message: str, role_path: Path) -> dict:
    payload = {
        "role_path": str(role_path).replace("\\", "/"),
        "message": message,
        "scene_id": scene_id(),
        "session_id": session_id(),
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{api_base()}/chat",
        data=data,
        method="POST",
        headers={"Content-Type": "application/json; charset=utf-8"},
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        raw = resp.read().decode("utf-8")
        return json.loads(raw)


def extract_reply(body: dict) -> str:
    if "data" in body and isinstance(body["data"], dict):
        inner = body["data"]
        if "reply" in inner:
            return str(inner["reply"])
    if "reply" in body:
        return str(body["reply"])
    raise KeyError(f"no reply field in response keys={list(body.keys())}")


def post_chat_stream(message: str, role_path: Path) -> tuple[str, int | None, int]:
    """POST /chat/stream; returns (reply, ttft_ms, total_ms)."""
    payload = {
        "role_path": str(role_path).replace("\\", "/"),
        "message": message,
        "scene_id": scene_id(),
        "session_id": session_id(),
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{api_base()}/chat/stream",
        data=data,
        method="POST",
        headers={
            "Content-Type": "application/json; charset=utf-8",
            "Accept": "text/event-stream",
        },
    )
    started = time.perf_counter()
    ttft_ms: int | None = None
    tokens: list[str] = []
    reply = ""
    event_name = ""
    with urllib.request.urlopen(req, timeout=300) as resp:
        while True:
            line = resp.readline()
            if not line:
                break
            text = line.decode("utf-8", errors="replace").strip()
            if not text:
                continue
            if text.startswith("event:"):
                event_name = text.split(":", 1)[1].strip()
                continue
            if not text.startswith("data:"):
                continue
            data_text = text.split(":", 1)[1].strip()
            if event_name == "token":
                try:
                    token = str(json.loads(data_text).get("token", ""))
                except json.JSONDecodeError:
                    token = data_text
                if token:
                    if ttft_ms is None:
                        ttft_ms = int((time.perf_counter() - started) * 1000)
                    tokens.append(token)
                    print(token, end="", flush=True)
            elif event_name == "done":
                try:
                    body = json.loads(data_text)
                    reply = extract_reply(body)
                except (json.JSONDecodeError, KeyError):
                    reply = "".join(tokens)
            elif event_name == "error":
                raise RuntimeError(data_text)
    total_ms = int((time.perf_counter() - started) * 1000)
    if not reply:
        reply = "".join(tokens)
    if tokens:
        print()
    return reply, ttft_ms, total_ms


def speak(text: str, use_plugin_tts: bool = False, tts_model_dir: Path | None = None) -> None:
    if use_plugin_tts and tts_model_dir:
        try:
            from tts.engine import synthesize_text

            result = synthesize_text(model_dir=tts_model_dir, text=text)
            if not result.get("ok"):
                print(f"[tts] {result.get('reason', 'failed')}", file=sys.stderr)
                return
            import base64
            import struct
            import wave
            from io import BytesIO

            import sounddevice as sd

            raw = base64.b64decode(result["audio_base64"])
            with wave.open(BytesIO(raw), "rb") as wf:
                sr = wf.getframerate()
                pcm = wf.readframes(wf.getnframes())
            count = len(pcm) // 2
            samples = [s / 32768.0 for s in struct.unpack(f"<{count}h", pcm)]
            sd.play(samples, sr)
            sd.wait()
            return
        except Exception as exc:  # noqa: BLE001
            print(f"[tts] sherpa failed: {exc}", file=sys.stderr)
    try:
        import pyttsx3  # type: ignore
    except ImportError:
        print("[tts] pyttsx3 not installed; pip install pyttsx3", file=sys.stderr)
        return
    engine = pyttsx3.init()
    engine.say(text)
    engine.runAndWait()


def transcribe_mic(seconds: float, model_dir: Path) -> str:
    from asr.engine import transcribe_audio
    from asr.mic import record_seconds

    wav_bytes = record_seconds(seconds=seconds)
    import base64

    result = transcribe_audio(
        model_dir=model_dir,
        audio_base64=base64.b64encode(wav_bytes).decode("ascii"),
    )
    if not result.get("ok"):
        raise RuntimeError(result.get("reason") or result.get("message") or "asr failed")
    return str(result.get("text", "")).strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="OClive minimal chat loop (HTTP)")
    parser.add_argument("--tts", action="store_true", help="Speak reply (pyttsx3 or sherpa TTS)")
    parser.add_argument("--tts-sherpa", action="store_true", help="Use sherpa TTS model for reply")
    parser.add_argument("--stream", action="store_true", help="Use POST /chat/stream and print ttft_ms")
    parser.add_argument("--mic", action="store_true", help="Record from microphone → ASR → chat")
    parser.add_argument("--mic-seconds", type=float, default=3.0, help="Mic capture length (default 3)")
    parser.add_argument("--asr-model-dir", type=Path, default=None, help="ASR model directory")
    parser.add_argument("--role-path", type=Path, default=None, help="Role pack directory")
    args = parser.parse_args()

    role_path = args.role_path or default_role_path()
    asr_model_dir = args.asr_model_dir or default_asr_model_dir()
    if not role_path.is_dir():
        print(f"role_path not found: {role_path}", file=sys.stderr)
        return 1

    if not health_ok():
        print(
            f"kernel not reachable at {api_base()}/health — start tauri:dev or oclive-kernel-server --api",
            file=sys.stderr,
        )
        return 1

    print(f"OClive voice-loop | base={api_base()} | role={role_path}")
    print(f"session_id={session_id()} (fixed for memory)")
    if args.mic:
        print(f"mic mode | asr_model={asr_model_dir} | seconds={args.mic_seconds}")
    else:
        print("Type a message and Enter (empty line or Ctrl+C to quit).\n")

    while True:
        try:
            if args.mic:
                input("Press Enter to record from mic (Ctrl+C to quit)... ")
                line = transcribe_mic(args.mic_seconds, asr_model_dir)
                print(f"you> {line}")
            else:
                line = input("you> ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break
        if not line:
            break
        try:
            if args.stream:
                reply, ttft_ms, total_ms = post_chat_stream(line, role_path)
                print(f"bot> {reply}")
                print(f"[stream] ttft_ms={ttft_ms} total_ms={total_ms}")
            else:
                body = post_chat(line, role_path)
                reply = extract_reply(body)
                print(f"bot> {reply}\n")
        except urllib.error.HTTPError as e:
            err = e.read().decode("utf-8", errors="replace")
            print(f"[error] HTTP {e.code}: {err}", file=sys.stderr)
            continue
        except (urllib.error.URLError, TimeoutError, KeyError, json.JSONDecodeError, RuntimeError) as e:
            print(f"[error] {e}", file=sys.stderr)
            continue

        if args.tts or args.tts_sherpa:
            tts_dir = Path(__file__).resolve().parent / "models" / "tts" / "sherpa-piper-zh"
            speak(reply, use_plugin_tts=args.tts_sherpa, tts_model_dir=tts_dir)
        if not args.stream:
            print()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
