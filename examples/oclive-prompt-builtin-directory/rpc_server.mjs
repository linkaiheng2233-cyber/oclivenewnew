/**
 * `prompt.build_prompt`：stdin JSON → `oclive_prompt_from_json`（与 `oclive_prompt_builtin::PromptBuilder` 一致）。
 * `prompt.top_topic_hint`：与 `TopicHintContext::top_topic_name_for_scene` 等价的轻量选取。
 */
import http from "node:http";
import { spawnSync } from "node:child_process";

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

function resolvePromptFromJsonExe() {
  const p = process.env.OCLIVE_PROMPT_FROM_JSON?.trim();
  if (!p) {
    throw new Error(
      "Missing env OCLIVE_PROMPT_FROM_JSON: path to oclive_prompt_from_json (build with: cargo build -p oclive_prompt_builtin --features prompt-from-json-bin --bin oclive_prompt_from_json)"
    );
  }
  return p;
}

function handlePromptBuildPrompt(params) {
  const exe = resolvePromptFromJsonExe();
  const input = JSON.stringify(params ?? {});
  const r = spawnSync(exe, [], {
    input,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
    windowsHide: true,
  });
  if (r.error) {
    throw r.error;
  }
  if (r.status !== 0) {
    const err = (r.stderr && String(r.stderr).trim()) || `exit ${r.status}`;
    throw new Error(`oclive_prompt_from_json: ${err}`);
  }
  const out = JSON.parse(String(r.stdout || "{}"));
  if (out && typeof out.prompt === "string") {
    return { prompt: out.prompt };
  }
  throw new Error("oclive_prompt_from_json: bad stdout shape, expected { prompt: string }");
}

function handlePromptTopTopicHint(params) {
  const sceneId = params?.scene_id != null ? String(params.scene_id) : "";
  const ctx = params?.topic_hint_context ?? {};
  const tw = ctx.topic_weights;
  if (!tw || typeof tw !== "object" || !sceneId) {
    return { hint: null };
  }
  const sceneMap = tw[sceneId];
  if (!sceneMap || typeof sceneMap !== "object") {
    return { hint: null };
  }
  let best = null;
  let bestW = -Infinity;
  for (const [name, w] of Object.entries(sceneMap)) {
    const n = Number(w);
    if (!Number.isFinite(n)) continue;
    if (n > bestW || (n === bestW && best === null)) {
      bestW = n;
      best = name;
    }
  }
  return { hint: best };
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
    try {
      if (msg.method === "prompt.build_prompt") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handlePromptBuildPrompt(msg.params)));
        return;
      }
      if (msg.method === "prompt.top_topic_hint") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handlePromptTopTopicHint(msg.params)));
        return;
      }
    } catch (e) {
      res.writeHead(200);
      res.end(
        jsonRpcError(
          id,
          -32000,
          e && typeof e.message === "string" ? e.message : String(e)
        )
      );
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
