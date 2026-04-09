"""
Shared JSON-RPC 2.0 over HTTP (single POST path) for oclive sidecar examples.

Used by remote_plugin_minimal and remote_plugin_openai_compat — keep DRY, one place for wire format.
"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import sys
from typing import Callable, Optional, Tuple

# handle_method(method, params) -> dict (result or jsonrpc error shape), or None if unknown method


def rpc_result(req_id, result):
    return {"jsonrpc": "2.0", "id": req_id, "result": result}


def rpc_error(req_id, code, message, data=None):
    err = {"code": code, "message": message}
    if data is not None:
        err["data"] = data
    return {"jsonrpc": "2.0", "id": req_id, "error": err}


def dispatch_jsonrpc(
    body: str, handle_method: Callable[[str, dict], Optional[dict]]
) -> Tuple[int, bytes]:
    """
    Returns (http_status, response_body_bytes).
    On JSON parse error: 400 empty body.
    """
    try:
        req = json.loads(body)
    except json.JSONDecodeError:
        return 400, b""

    req_id = req.get("id")
    method = req.get("method")
    params = req.get("params")
    if req.get("jsonrpc") != "2.0" or not method:
        out = rpc_error(req_id, -32600, "Invalid Request")
    else:
        if not isinstance(params, dict):
            params = {}
        result = handle_method(method, params)
        if isinstance(result, dict) and "error" in result:
            out = dict(result)
            out["id"] = req_id
        elif result is None:
            out = rpc_error(req_id, -32601, "Method not found: " + str(method))
        else:
            out = rpc_result(req_id, result)
    raw = json.dumps(out, ensure_ascii=False).encode("utf-8")
    return 200, raw


def make_handler_class(path: str, handle_method: Callable[[str, dict], Optional[dict]]):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, fmt, *args):
            sys.stderr.write("%s - %s\n" % (self.address_string(), args[0] % args[1:]))

        def do_POST(self):
            if self.path != path:
                self.send_error(404)
                return
            length = int(self.headers.get("Content-Length") or 0)
            body = self.rfile.read(length).decode("utf-8", errors="replace")
            status, payload = dispatch_jsonrpc(body, handle_method)
            self.send_response(status)
            if status == 400:
                self.end_headers()
                return
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

    return Handler


def run_server(host: str, port: int, path: str, handle_method: Callable, banner: str) -> None:
    handler_cls = make_handler_class(path, handle_method)
    httpd = HTTPServer((host, port), handler_cls)
    print(banner)
    httpd.serve_forever()
