/**
 * 目录插件 JSON-RPC 侧车：实现 llm.generate / llm.generate_tag，
 * 将请求转发到本机 llama.cpp HTTP server（见 README）。
 *
 * 就绪行：OCLIVE_READY http://127.0.0.1:<port>/rpc
 */
import http from "node:http";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

function readPluginConfig() {
  const raw = String(process.env.OCLIVE_PLUGIN_CONFIG || "").trim();
  if (!raw) return {};
  try {
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`invalid OCLIVE_PLUGIN_CONFIG: ${message}`);
  }
}

const PLUGIN_CONFIG = readPluginConfig();
const LLAMA_BASE = String(
  PLUGIN_CONFIG.base_url ||
    process.env.OCLIVE_LLAMACPP_SERVER_URL ||
    "http://127.0.0.1:8080"
).replace(/\/$/, "");
const ADAPTER_MODEL = String(
  PLUGIN_CONFIG.adapter_model || process.env.OCLIVE_LORA_MODEL || ""
).trim();

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

function openaiChatBody(model, prompt, temperature, max_tokens, stream) {
  return {
    model: ADAPTER_MODEL || model || "gpt-3.5-turbo",
    messages: [{ role: "user", content: prompt }],
    temperature,
    max_tokens,
    stream,
  };
}

/** OpenAI-compatible chat completions（llama-server 常见路径） */
async function openaiChat(model, prompt, temperature, max_tokens) {
  const url = `${LLAMA_BASE}/v1/chat/completions`;
  return fetchJson(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(
      openaiChatBody(model, prompt, temperature, max_tokens, false)
    ),
  });
}

async function openaiChatStream(
  model,
  prompt,
  temperature,
  max_tokens,
  onToken
) {
  const url = `${LLAMA_BASE}/v1/chat/completions`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(
      openaiChatBody(model, prompt, temperature, max_tokens, true)
    ),
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(
      `upstream ${response.status}: ${text.slice(0, 400) || "(empty)"}`
    );
  }

  const contentType = String(response.headers.get("content-type") || "");
  if (!contentType.includes("text/event-stream")) {
    const body = await response.json();
    const text = extractChatText(body);
    if (text == null) {
      throw new Error(
        `chat completions shape not recognized: ${JSON.stringify(body).slice(0, 500)}`
      );
    }
    onToken(text);
    return text;
  }

  const decoder = new TextDecoder();
  let pending = "";
  let dataLines = [];
  let full = "";
  const consumeEvent = () => {
    if (dataLines.length === 0) return;
    const data = dataLines.join("\n").trim();
    dataLines = [];
    if (!data || data === "[DONE]") return;
    const event = JSON.parse(data);
    const token =
      event?.choices?.[0]?.delta?.content ??
      event?.choices?.[0]?.text ??
      "";
    if (typeof token === "string" && token.length > 0) {
      full += token;
      onToken(token);
    }
  };
  const consumeLine = (rawLine) => {
    const line = rawLine.endsWith("\r") ? rawLine.slice(0, -1) : rawLine;
    if (line === "") {
      consumeEvent();
    } else if (line.startsWith("data:")) {
      dataLines.push(line.slice(5).trimStart());
    }
  };

  for await (const chunk of response.body) {
    pending += decoder.decode(chunk, { stream: true });
    let newline;
    while ((newline = pending.indexOf("\n")) >= 0) {
      consumeLine(pending.slice(0, newline));
      pending = pending.slice(newline + 1);
    }
  }
  pending += decoder.decode();
  if (pending) consumeLine(pending);
  consumeEvent();
  if (!full) {
    throw new Error("chat stream completed without text");
  }
  return full;
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

async function runLlmStream(model, prompt, { temperature, max_tokens }, onToken) {
  let emitted = false;
  const forwardToken = (token) => {
    emitted = true;
    onToken(token);
  };
  try {
    return await openaiChatStream(
      model,
      prompt,
      temperature,
      max_tokens,
      forwardToken
    );
  } catch (streamError) {
    if (emitted) {
      throw streamError;
    }
    try {
      const text = await runLlm(model, prompt, {
        temperature,
        max_tokens,
        tag: false,
      });
      onToken(text);
      return text;
    } catch (fallbackError) {
      const streamMessage =
        streamError instanceof Error ? streamError.message : String(streamError);
      const fallbackMessage =
        fallbackError instanceof Error
          ? fallbackError.message
          : String(fallbackError);
      throw new Error(
        `stream: ${streamMessage} | full-response fallback: ${fallbackMessage}`
      );
    }
  }
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

async function handleLlmGenerateStream(params, onToken) {
  const model = params && typeof params.model === "string" ? params.model : "";
  const prompt = params && typeof params.prompt === "string" ? params.prompt : "";
  if (!prompt) {
    throw new Error("missing params.prompt");
  }
  const text = await runLlmStream(
    model,
    prompt,
    {
      temperature: 0.7,
      max_tokens: 2048,
    },
    onToken
  );
  return { text };
}

function streamResult(id, event, fields = {}) {
  return `${JSON.stringify({
    jsonrpc: "2.0",
    id,
    result: { event, ...fields },
  })}\n`;
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
        if (msg.method === "llm.generate_stream") {
          res.writeHead(200, {
            "Content-Type": "application/x-ndjson; charset=utf-8",
          });
          await handleLlmGenerateStream(msg.params, (token) => {
            res.write(streamResult(id, "token", { text: token }));
          });
          res.end(streamResult(id, "done"));
          return;
        } else if (msg.method === "llm.generate") {
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
        if (res.headersSent) {
          res.end(`${jsonRpcError(id, -32603, `llamacpp proxy: ${m}`)}\n`);
        } else {
          res.writeHead(200);
          res.end(jsonRpcError(id, -32603, `llamacpp proxy: ${m}`));
        }
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
