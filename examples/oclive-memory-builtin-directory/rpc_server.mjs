/**
 * 与 `oclive_memory_builtin::providers::BuiltinMemoryRetrievalV2` 对齐的 memory.rank：
 * score = importance * weight * (1 + query_overlap_boost(user_query, content))。
 * user_query 为空时退化为按 importance×weight 排序（与 V1 一致）。
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

function queryOverlapBoost(query, content) {
  const q = (query ?? "").trim();
  if (!q) return 0;
  const ql = q.toLowerCase();
  const cl = (content ?? "").toLowerCase();
  let hits = 0;
  for (const w of ql.split(/\s+/)) {
    if (w.length >= 2 && cl.includes(w)) hits += 1;
  }
  if (hits === 0 && [...ql].length >= 2) {
    const chars = [...ql];
    for (let i = 0; i + 1 < chars.length; i++) {
      const s = chars[i] + chars[i + 1];
      if (cl.includes(s)) hits += 1;
    }
  }
  return Math.min(hits * 0.15, 0.6);
}

function handleMemoryRank(params) {
  const memories = params && Array.isArray(params.memories) ? params.memories : [];
  const limit = Math.max(1, Number(params?.limit) || 10);
  const userQuery = params?.user_query != null ? String(params.user_query) : "";
  const scored = memories.map((m) => {
    const importance = Number(m?.importance ?? 0);
    const weight = Number(m?.weight ?? 1);
    const base = importance * weight;
    const content = m?.content != null ? String(m.content) : "";
    const boost = queryOverlapBoost(userQuery, content);
    return { m, score: base * (1 + boost) };
  });
  scored.sort((a, b) => b.score - a.score);
  const ordered_ids = scored.slice(0, limit).map((x) => String(x.m?.id ?? ""));
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
