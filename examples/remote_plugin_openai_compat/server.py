#!/usr/bin/env python3
"""
JSON-RPC 侧车：llm.generate / llm.generate_tag 转发到 OpenAI 兼容 HTTPS API（chat/completions）。

其余方法与 remote_plugin_minimal 相同，占位返回，便于把 PLUGIN 与 LLM 指到同一 URL 联调。

协议：creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md
依赖：pip install -r requirements.txt
"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import os
import sys

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

HOST = "127.0.0.1"
PORT = int(os.environ.get("SIDECAR_PORT", "8765"))
PATH = "/rpc"


def rpc_result(req_id, result):
    return {"jsonrpc": "2.0", "id": req_id, "result": result}


def rpc_error(req_id, code, message, data=None):
    err = {"code": code, "message": message}
    if data is not None:
        err["data"] = data
    return {"jsonrpc": "2.0", "id": req_id, "error": err}


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
        # Some APIs return content as list of parts
        text = "".join(
            p.get("text", "") if isinstance(p, dict) else str(p) for p in text
        )

    return {"text": str(text).strip()}


def handle_method(_method, params):
    if not isinstance(params, dict):
        params = {}

    if _method == "memory.rank":
        memories = params.get("memories") or []
        limit = int(params.get("limit") or 8)
        ids = [m.get("id") for m in memories if isinstance(m, dict) and m.get("id")]
        return {"ordered_ids": ids[:limit]}

    if _method == "emotion.analyze":
        return {
            "joy": 0.0,
            "sadness": 0.0,
            "anger": 0.0,
            "fear": 0.0,
            "surprise": 0.0,
            "disgust": 0.0,
            "neutral": 1.0,
        }

    if _method == "event.estimate":
        return {"event_type": {"Ignore": None}, "impact_factor": 0.0, "confidence": 0.5}

    if _method == "prompt.build_prompt":
        return {"prompt": "[openai-compat stub] implement prompt.build_prompt in your fork if needed.\n"}

    if _method == "prompt.top_topic_hint":
        return {"hint": None}

    if _method == "llm.generate":
        prompt = params.get("prompt") or ""
        model = params.get("model") or ""
        out = _openai_chat(str(prompt), str(model), tag_mode=False)
        return out

    if _method == "llm.generate_tag":
        prompt = params.get("prompt") or ""
        model = params.get("model") or ""
        out = _openai_chat(str(prompt), str(model), tag_mode=True)
        return out

    return None


class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        sys.stderr.write("%s - %s\n" % (self.address_string(), args[0] % args[1:]))

    def do_POST(self):
        if self.path != PATH:
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(length).decode("utf-8", errors="replace")
        try:
            req = json.loads(body)
        except json.JSONDecodeError:
            self.send_response(400)
            self.end_headers()
            return
        req_id = req.get("id")
        method = req.get("method")
        params = req.get("params")
        if req.get("jsonrpc") != "2.0" or not method:
            out = rpc_error(req_id, -32600, "Invalid Request")
        else:
            result = handle_method(method, params)
            if isinstance(result, dict) and "error" in result:
                out = dict(result)
                out["id"] = req_id
            elif result is None:
                out = rpc_error(req_id, -32601, "Method not found: " + str(method))
            else:
                out = rpc_result(req_id, result)
        raw = json.dumps(out, ensure_ascii=False).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)


def main():
    httpd = HTTPServer((HOST, PORT), Handler)
    print("oclive OpenAI-compatible sidecar on http://%s:%s%s" % (HOST, PORT, PATH))
    print("Set OPENAI_API_KEY (and optionally OPENAI_BASE_URL / OPENAI_MODEL).")
    httpd.serve_forever()


if __name__ == "__main__":
    main()
