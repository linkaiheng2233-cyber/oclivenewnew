/**
 * Minimal directory plugin: theater.build_prompt — demonstrates fork by swapping guardrails tone.
 *
 * Self-contained prompts/ — safe to copy to {app_data}/plugins/com.example.theater_director_comedy/
 * and set OCLIVE_THEATER_DIRECTOR_PLUGIN=com.example.theater_director_comedy
 */
import http from "node:http";
import { buildTheaterPrompt } from "./prompts/index.mjs";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";
const STYLE_PREFIX = "[comedy-pack] ";

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

function handleBuildPrompt(params) {
  const mode = params && typeof params.mode === "string" ? params.mode : "ripple";
  const base = buildTheaterPrompt(params || {});
  return {
    prompt: `${STYLE_PREFIX}${base.replace("【戏剧性纪律】", "【喜剧纪律】反差要更夸张、可带自嘲；")}`,
    diagnostic: "directory-plugin-theater-director-minimal",
    mode,
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
    if (msg.method === "theater.build_prompt") {
      try {
        const result = handleBuildPrompt(msg.params);
        res.writeHead(200);
        res.end(jsonRpcResult(id, result));
      } catch (e) {
        res.writeHead(200);
        res.end(jsonRpcError(id, -32000, e instanceof Error ? e.message : "build failed"));
      }
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
