/**
 * Reply post-process polish scaffold — pass-through by default.
 * Replace `polishReply` with your LLM call (Ollama, OpenAI-compatible, etc.).
 *
 * Enable in role pack config.json:
 *   "reply_post_processor": { "enabled": true, "backend": "directory",
 *     "directory": { "plugin_id": "reply-post-process-polish" } }
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

/**
 * @param {Record<string, unknown> | null | undefined} params
 * @returns {{ display_reply: string, diagnostic?: string }}
 */
function polishReply(params) {
  const raw = params && typeof params.raw_reply === "string" ? params.raw_reply : "";
  const userMessage =
    params && typeof params.user_message === "string" ? params.user_message : "";
  const roleId = params && typeof params.role_id === "string" ? params.role_id : "";
  const locale = params && typeof params.locale === "string" ? params.locale : "zh";

  // Default: pass-through (safe for smoke tests). Uncomment and implement LLM rewrite:
  // const prompt = `Polish this character reply in ${locale}. User: ${userMessage}\nRaw: ${raw}`;
  // const display_reply = await callYourLlm(prompt);

  void userMessage;
  void roleId;

  return {
    display_reply: raw,
    diagnostic: "reply-post-process-polish:pass-through",
  };
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
    if (msg.method === "reply_post_process.process") {
      res.writeHead(200);
      res.end(jsonRpcResult(id, polishReply(msg.params)));
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
