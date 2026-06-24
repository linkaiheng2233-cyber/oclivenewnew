#!/usr/bin/env python3
"""Minimal voice/chat loop: text in → POST /chat → reply out (optional TTS).

ASR is not included in v0 — type at the prompt to simulate speech-to-text.
See README.md and human-docs/team/TRACK_VOICE_RECOGNITION.md.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
import uuid
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
    # HTTP wrapper: { "data": { "reply": "..." }, ... } or flat SendMessageResponse.
    if "data" in body and isinstance(body["data"], dict):
        inner = body["data"]
        if "reply" in inner:
            return str(inner["reply"])
    if "reply" in body:
        return str(body["reply"])
    raise KeyError(f"no reply field in response keys={list(body.keys())}")


def speak(text: str) -> None:
    try:
        import pyttsx3  # type: ignore
    except ImportError:
        print("[tts] pyttsx3 not installed; pip install pyttsx3", file=sys.stderr)
        return
    engine = pyttsx3.init()
    engine.say(text)
    engine.runAndWait()


def main() -> int:
    parser = argparse.ArgumentParser(description="OClive minimal chat loop (HTTP)")
    parser.add_argument("--tts", action="store_true", help="Speak reply via pyttsx3")
    parser.add_argument("--role-path", type=Path, default=None, help="Role pack directory")
    args = parser.parse_args()

    role_path = args.role_path or default_role_path()
    if not role_path.is_dir():
        print(f"role_path not found: {role_path}", file=sys.stderr)
        return 1

    if not health_ok():
        print(
            f"kernel not reachable at {api_base()}/health — start tauri:dev or oclive-kernel-server --api",
            file=sys.stderr,
        )
        return 1

    print(f"OClive voice-loop v0 | base={api_base()} | role={role_path}")
    print(f"session_id={session_id()} (fixed for memory)")
    print("Type a message and Enter (empty line or Ctrl+C to quit).\n")

    while True:
        try:
            line = input("you> ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break
        if not line:
            break
        try:
            body = post_chat(line, role_path)
            reply = extract_reply(body)
        except urllib.error.HTTPError as e:
            err = e.read().decode("utf-8", errors="replace")
            print(f"[error] HTTP {e.code}: {err}", file=sys.stderr)
            continue
        except (urllib.error.URLError, TimeoutError, KeyError, json.JSONDecodeError) as e:
            print(f"[error] {e}", file=sys.stderr)
            continue

        print(f"bot> {reply}\n")
        if args.tts:
            speak(reply)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
