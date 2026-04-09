#!/usr/bin/env python3
"""
Minimal JSON-RPC 2.0 HTTP server for oclive remote_plugin / remote_llm handshake.
Protocol: creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md

Shared JSON-RPC wiring: ../common/jsonrpc_http.py
Shared non-LLM stubs: ../common/oclive_stub_handlers.py
"""
from pathlib import Path
import os
import sys
import time

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from common.jsonrpc_http import rpc_error, run_server  # noqa: E402
from common.oclive_stub_handlers import handle_stub_plugin_method  # noqa: E402

HOST = "127.0.0.1"
PORT = 8765
PATH = "/rpc"
DEMO_ERROR_MODE = os.environ.get("OCLIVE_DEMO_ERROR_MODE", "").strip().lower()


def handle_method(_method, params):
    if not isinstance(params, dict):
        params = {}
    if DEMO_ERROR_MODE == "timeout":
        time.sleep(9.5)
    if DEMO_ERROR_MODE == "auth":
        return rpc_error(
            None,
            -32011,
            "demo auth failed",
            {"hint": "set demo token or switch OCLIVE_DEMO_ERROR_MODE"},
        )
    if DEMO_ERROR_MODE == "rate_limit":
        return rpc_error(
            None,
            -32012,
            "demo rate limited",
            {"retry_after_ms": 3000},
        )
    if DEMO_ERROR_MODE == "upstream":
        return rpc_error(
            None,
            -32013,
            "demo upstream unavailable",
            {"upstream": "mock-llm"},
        )

    r = handle_stub_plugin_method(_method, params)
    if r is not None:
        return r

    if _method == "llm.generate":
        return {"text": "[remote stub] llm.generate"}
    if _method == "llm.generate_tag":
        return {"text": "neutral"}
    return None


def main():
    banner = "oclive minimal remote plugin listening on http://%s:%s%s" % (HOST, PORT, PATH)
    if DEMO_ERROR_MODE:
        banner += "\ndemo error mode: " + DEMO_ERROR_MODE
    run_server(HOST, PORT, PATH, handle_method, banner)


if __name__ == "__main__":
    main()
