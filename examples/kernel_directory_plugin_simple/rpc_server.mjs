/**
 * 极简 JSON-RPC 侧车（Kernel SDK 示例）：
 * - echo.ping：手工 curl/调试器验证 echo
 * - memory.rank：将角色包 plugin_backends.memory 设为 directory 且 directory_plugins.memory 指向本插件 id 时由内核调用
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

function handleEchoPing(params) {
  const text = params && params.text != null ? String(params.text) : "";
  return { pong: true, text, note: "com.oclive.sdk.directory_simple" };
}

function handleMemoryRank(params) {
  const memories = params && Array.isArray(params.memories) ? params.memories : [];
  const ordered_ids = memories.map((m) => (m && m.id ? String(m.id) : ""));
  return { ordered_ids };
}

const server = http.createServer((req, res) => {
  if (req.method !== "POST" || !req.url || !req.url.startsWith("/rpc")) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("not found");
    return;
  }
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", () => {
    const raw = Buffer.concat(chunks).toString("utf8");
    let msg;
    try {
      msg = JSON.parse(raw);
    } catch {
      res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
      res.end(jsonRpcError(null, -32700, "parse error"));
      return;
    }
    const id = msg.id ?? null;
    if (msg.jsonrpc !== "2.0" || typeof msg.method !== "string") {
      res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
      res.end(jsonRpcError(id, -32600, "invalid request"));
      return;
    }
    res.setHeader("Content-Type", "application/json; charset=utf-8");
    res.setHeader(PROTOCOL_HEADER, PROTOCOL_VALUE);

    if (msg.method === "echo.ping") {
      res.writeHead(200);
      res.end(jsonRpcResult(id, handleEchoPing(msg.params)));
      return;
    }
    if (msg.method === "memory.rank") {
      res.writeHead(200);
      res.end(jsonRpcResult(id, handleMemoryRank(msg.params)));
      return;
    }
    res.writeHead(200);
    res.end(jsonRpcError(id, -32601, `method not found: ${msg.method}`));
  });
});

server.listen(0, "127.0.0.1", () => {
  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : 0;
  const url = `http://127.0.0.1:${port}/rpc`;
  process.stdout.write(`OCLIVE_READY ${url}\n`);
});

process.on("SIGTERM", () => server.close());
process.on("SIGINT", () => server.close());
