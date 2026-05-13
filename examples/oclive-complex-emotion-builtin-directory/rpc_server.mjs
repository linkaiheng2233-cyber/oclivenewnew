/**
 * 与 `oclive_complex_emotion_builtin::BuiltinKeywordComplexEmotionProvider::resolve_turn_inner` 对齐。
 */
import http from "node:http";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

const SOURCE = "builtin_keyword_v1";
const CONF = 0.7;
const INT = 0.5;

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

function userTextLenChars(s) {
  return String(s ?? "").trim().length;
}

function containsAny(hay, needles) {
  const h = hay ?? "";
  return needles.some((n) => h.includes(n));
}

function baseOutput(pattern, narrativeHint, labels) {
  return {
    source: SOURCE,
    narrative_hint: narrativeHint,
    labels,
    pattern,
    confidence: CONF,
    intensity: INT,
    dissonance_score: 0,
    degraded_to_builtin: false,
  };
}

function defaultFallback() {
  return {
    source: SOURCE,
    narrative_hint: "未命中特定模式；保持自然对话节奏即可。",
    labels: [],
    pattern: null,
    confidence: 0.35,
    intensity: 0.25,
    dissonance_score: 0,
    degraded_to_builtin: false,
  };
}

function resolveTurnInner(params) {
  const u = String(params?.user_message ?? "");
  const v = params?.user_valence != null ? Number(params.user_valence) : 0;
  const d = params?.user_dominance != null ? Number(params.user_dominance) : 0;
  const prev = params?.previous_user_message;

  if (prev != null && userTextLenChars(prev) <= 2 && userTextLenChars(u) <= 2) {
    return baseOutput(
      "waning_engagement",
      "对话热度下降，角色可尝试提出新话题或幽默打破沉闷。",
      ["low_energy"],
    );
  }

  if (containsAny(u, ["没事", "我没事", "不用管我"]) && v < 0) {
    return baseOutput(
      "suppressed_distress",
      "用户可能在掩饰情绪，角色宜保持温柔关注，不必追问。",
      ["masking", "support"],
    );
  }

  if (containsAny(u, ["随便", "都行", "你定"])) {
    return baseOutput(
      "disengagement",
      "用户可能缺乏兴致，角色可主动提供简单选项或转换话题。",
      ["low_drive"],
    );
  }

  if (containsAny(u, ["真好", "真羡慕你"]) && d < 0) {
    return baseOutput(
      "wistful_envy",
      "用户流露向往与轻微落差感，角色可适度分享脆弱面拉近距离。",
      ["social_compare"],
    );
  }

  return defaultFallback();
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
    if (msg.method === "complex_emotion.resolve_turn") {
      res.writeHead(200);
      res.end(jsonRpcResult(id, resolveTurnInner(msg.params)));
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
