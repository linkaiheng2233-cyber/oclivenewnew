/**
 * 目录插件 JSON-RPC 侧车：实现 llm.generate / llm.generate_tag，
 * 将请求转发到本机 llama.cpp HTTP server（见 README）。
 *
 * 就绪行：OCLIVE_READY http://127.0.0.1:<port>/rpc
 */
import http from "node:http";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

const LLAMA_BASE = (
  process.env.OCLIVE_LLAMACPP_SERVER_URL || "http://127.0.0.1:8080"
).replace(/\/$/, "");

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

async function fetchJson(url, init) {
  const r = await fetch(url, init);
  const text = await r.text();
  let body;
  try {
    body = text ? JSON.parse(text) : {};
  } catch {
    throw new Error(`non-json response status=${r.status} body=${text.slice(0, 400)}`);
  }
  if (!r.ok) {
    const detail = body?.error?.message || JSON.stringify(body).slice(0, 400);
    throw new Error(`upstream ${r.status}: ${detail}`);
  }
  return body;
}

/** OpenAI-compatible chat completions（llama-server 常见路径） */
async function openaiChat(model, prompt, temperature, max_tokens) {
  const url = `${LLAMA_BASE}/v1/chat/completions`;
  return fetchJson(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      model: model || "gpt-3.5-turbo",
      messages: [{ role: "user", content: prompt }],
      temperature,
      max_tokens,
    }),
  });
}

/** 旧版 /completion 回退（部分 llama-server 构建） */
async function legacyCompletion(prompt, temperature, n_predict) {
  const url = `${LLAMA_BASE}/completion`;
  return fetchJson(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      prompt,
      temperature,
      n_predict,
      stream: false,
    }),
  });
}

function extractChatText(j) {
  const c = j?.choices?.[0]?.message?.content;
  if (typeof c === "string" && c.length > 0) return c;
  const alt = j?.choices?.[0]?.text;
  if (typeof alt === "string" && alt.length > 0) return alt;
  return null;
}

function extractCompletionText(j) {
  if (typeof j?.content === "string" && j.content.length > 0) return j.content;
  const g0 = j?.generations?.[0];
  if (typeof g0?.text === "string" && g0.text.length > 0) return g0.text;
  return null;
}

async function runLlm(model, prompt, { temperature, max_tokens, tag }) {
  let j;
  try {
    j = await openaiChat(model, prompt, temperature, max_tokens);
  } catch (e1) {
    try {
      j = await legacyCompletion(prompt, temperature, max_tokens);
    } catch (e2) {
      const a = e1 instanceof Error ? e1.message : String(e1);
      const b = e2 instanceof Error ? e2.message : String(e2);
      throw new Error(`chat: ${a} | completion: ${b}`);
    }
    const t = extractCompletionText(j);
    if (t == null) {
      throw new Error(
        `completion shape not recognized: ${JSON.stringify(j).slice(0, 500)}`
      );
    }
    return tag ? t.trim().split(/\s+/)[0] || t.trim() : t;
  }
  let text = extractChatText(j);
  if (text == null) {
    throw new Error(
      `chat completions shape not recognized: ${JSON.stringify(j).slice(0, 500)}`
    );
  }
  if (tag) {
    text = text.trim().split(/\s+/)[0] || text.trim();
  }
  return text;
}

async function handleLlmGenerate(params) {
  const model = params && typeof params.model === "string" ? params.model : "";
  const prompt = params && typeof params.prompt === "string" ? params.prompt : "";
  if (!prompt) {
    throw new Error("missing params.prompt");
  }
  const text = await runLlm(model, prompt, {
    temperature: 0.7,
    max_tokens: 2048,
    tag: false,
  });
  return { text };
}

async function handleLlmGenerateTag(params) {
  const model = params && typeof params.model === "string" ? params.model : "";
  const prompt = params && typeof params.prompt === "string" ? params.prompt : "";
  if (!prompt) {
    throw new Error("missing params.prompt");
  }
  const text = await runLlm(model, prompt, {
    temperature: 0.2,
    max_tokens: 64,
    tag: true,
  });
  return { text };
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
    void (async () => {
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
        let result;
        if (msg.method === "llm.generate") {
          result = await handleLlmGenerate(msg.params);
        } else if (msg.method === "llm.generate_tag") {
          result = await handleLlmGenerateTag(msg.params);
        } else {
          res.writeHead(200);
          res.end(jsonRpcError(id, -32601, `method not found: ${msg.method}`));
          return;
        }
        res.writeHead(200);
        res.end(jsonRpcResult(id, result));
      } catch (e) {
        const m = e instanceof Error ? e.message : String(e);
        res.writeHead(200);
        res.end(jsonRpcError(id, -32603, `llamacpp proxy: ${m}`));
      }
    })();
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
