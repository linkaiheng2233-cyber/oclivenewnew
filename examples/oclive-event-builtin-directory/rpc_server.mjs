/**
 * 最小 `event.estimate` JSON-RPC，与进程内规则回退量级对齐的占位结果（侧车演示 / directory 槽位）。
 */
import http from "node:http";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

function jsonRpcResult(id, result) {
  return JSON.stringify({ jsonrpc: "2.0", id, result });
}

function jsonRpcError(id, code, message) {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    error: { code, message },
  });
}

function handleEventEstimate(_params) {
  return {
    event_type: "Ignore",
    impact_factor: 0.0,
    confidence: 0.35,
  };
}

const server = http.createServer((req, res) => {
  if (req.method !== "POST" || !req.url?.startsWith("/rpc")) {
    res.writeHead(404);
    res.end();
    return;
  }
  let body = "";
  req.on("data", (c) => {
    body += c;
  });
  req.on("end", () => {
    if (req.headers[PROTOCOL_HEADER] !== PROTOCOL_VALUE) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(jsonRpcError(null, -32600, "missing protocol header"));
      return;
    }
    let msg;
    try {
      msg = JSON.parse(body);
    } catch {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(jsonRpcError(null, -32700, "parse error"));
      return;
    }
    const id = msg.id;
    const method = msg.method;
    if (method === "event.estimate") {
      const result = handleEventEstimate(msg.params);
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(jsonRpcResult(id, result));
      return;
    }
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(jsonRpcError(id, -32601, `unknown method: ${method}`));
  });
});

const port = Number(process.env.OCLIVE_EVENT_DIR_PORT || 8791);
server.listen(port, "127.0.0.1", () => {
  console.error(`oclive-event-builtin-directory listening http://127.0.0.1:${port}/rpc`);
});
