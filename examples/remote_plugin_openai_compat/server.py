#!/usr/bin/env python3
"""
JSON-RPC 侧车：llm.generate / llm.generate_tag 转发到 OpenAI 兼容 HTTPS API（chat/completions）。

其余方法与 remote_plugin_minimal 相同（共享 ../common/oclive_stub_handlers.py）。

协议：creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md
依赖：pip install -r requirements.txt
"""
from pathlib import Path
import json
import os
import sys

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

try:
    from dotenv import load_dotenv

    load_dotenv()
except ImportError:
    pass

try:
    import requests
except ImportError as e:
    print("Missing dependency: pip install -r requirements.txt", file=sys.stderr)
    raise e

from common.jsonrpc_http import rpc_error, run_server  # noqa: E402
from common.oclive_stub_handlers import handle_stub_plugin_method  # noqa: E402

HOST = "127.0.0.1"
PORT = int(os.environ.get("SIDECAR_PORT", "8765"))
PATH = "/rpc"


def _openai_chat(prompt: str, model: str, tag_mode: bool):
    api_key = os.environ.get("OPENAI_API_KEY", "").strip()
    if not api_key:
        return rpc_error(
            None,
            -32603,
            "OPENAI_API_KEY is not set",
            {"hint": "copy .env.example to .env or export the variable"},
        )

    base = os.environ.get("OPENAI_BASE_URL", "https://api.openai.com/v1").rstrip("/")
    model = (model or "").strip() or os.environ.get("OPENAI_MODEL", "gpt-4o-mini").strip()
    url = f"{base}/chat/completions"

    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
    }
    if tag_mode:
        payload["max_tokens"] = 64
        payload["temperature"] = 0.2
    else:
        payload["temperature"] = float(os.environ.get("OPENAI_TEMPERATURE", "0.7"))

    timeout_sec = float(os.environ.get("OPENAI_TIMEOUT_SEC", "120"))
    try:
        r = requests.post(
            url,
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            json=payload,
            timeout=timeout_sec,
        )
    except requests.RequestException as ex:
        return rpc_error(
            None,
            -32013,
            "upstream request failed",
            {"detail": str(ex)},
        )

    if r.status_code >= 400:
        return rpc_error(
            None,
            -32013,
            f"upstream HTTP {r.status_code}",
            {"body": r.text[:2000]},
        )

    try:
        data = r.json()
    except json.JSONDecodeError:
        return rpc_error(None, -32603, "upstream returned non-JSON", {})

    choices = data.get("choices") or []
    if not choices:
        return rpc_error(None, -32603, "upstream returned no choices", {"raw": data})

    msg = choices[0].get("message") or {}
    text = msg.get("content")
    if text is None:
        return rpc_error(None, -32603, "upstream message has no content", {"raw": data})

    if isinstance(text, list):
        text = "".join(
            p.get("text", "") if isinstance(p, dict) else str(p) for p in text
        )

    return {"text": str(text).strip()}


def handle_method(_method, params):
    if not isinstance(params, dict):
        params = {}

    if _method == "llm.generate":
        prompt = params.get("prompt") or ""
        model = params.get("model") or ""
        return _openai_chat(str(prompt), str(model), tag_mode=False)

    if _method == "llm.generate_tag":
        prompt = params.get("prompt") or ""
        model = params.get("model") or ""
        return _openai_chat(str(prompt), str(model), tag_mode=True)

    return handle_stub_plugin_method(_method, params)


def main():
    banner = (
        "oclive OpenAI-compatible sidecar on http://%s:%s%s\n"
        "Set OPENAI_API_KEY (and optionally OPENAI_BASE_URL / OPENAI_MODEL)."
        % (HOST, PORT, PATH)
    )
    run_server(HOST, PORT, PATH, handle_method, banner)


if __name__ == "__main__":
    main()
