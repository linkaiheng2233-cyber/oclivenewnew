#!/usr/bin/env python3
"""
Minimal JSON-RPC 2.0 HTTP server for oclive remote_plugin / remote_llm handshake.
Protocol: docs/REMOTE_PLUGIN_PROTOCOL.md
"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import sys

HOST = "127.0.0.1"
PORT = 8765
PATH = "/rpc"


def rpc_result(req_id, result):
    return {"jsonrpc": "2.0", "id": req_id, "result": result}


def rpc_error(req_id, code, message):
    return {"jsonrpc": "2.0", "id": req_id, "error": {"code": code, "message": message}}


def handle_method(_method, params):
    if not isinstance(params, dict):
        params = {}
    # memory.rank
    if _method == "memory.rank":
        memories = params.get("memories") or []
        limit = int(params.get("limit") or 8)
        ids = [m.get("id") for m in memories if isinstance(m, dict) and m.get("id")]
        return {"ordered_ids": ids[:limit]}
    # emotion.analyze
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
    # event.estimate — EventImpactEstimate（EventType 为 serde 默认「外部标签」枚举，见 REMOTE_PLUGIN_PROTOCOL.md）
    if _method == "event.estimate":
        return {"event_type": {"Ignore": None}, "impact_factor": 0.0, "confidence": 0.5}
    # prompt.*
    if _method == "prompt.build_prompt":
        return {"prompt": "[remote stub] 请在侧车实现真实 prompt 组装。\n"}
    if _method == "prompt.top_topic_hint":
        return {"hint": None}
    # llm.*
    if _method == "llm.generate":
        return {"text": "[remote stub] llm.generate"}
    if _method == "llm.generate_tag":
        return {"text": "neutral"}
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
            if result is None:
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
    print("oclive minimal remote plugin listening on http://%s:%s%s" % (HOST, PORT, PATH))
    httpd.serve_forever()


if __name__ == "__main__":
    main()
