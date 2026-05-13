/**
 * 与 `oclive_emotion_builtin::classic::EmotionAnalyzer::analyze` 对齐的 `emotion.analyze`。
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

function analyzeEmotion(text) {
  let result = {
    joy: 0,
    sadness: 0,
    anger: 0,
    fear: 0,
    surprise: 0,
    disgust: 0,
    neutral: 0,
  };
  if (!text || text.length === 0) {
    result.neutral = 1;
    return result;
  }
  const textLower = text.toLowerCase();
  const paddedEn = ` ${textLower} `;

  const joyKeywords = [
    "开心",
    "高兴",
    "太好了",
    "太棒",
    "棒",
    "爱",
    "喜欢",
    "开颜",
    "哈哈",
    "hhh",
    "感谢",
    "谢谢",
    "感激",
    "期待",
    "想见",
    "抱抱",
    "mua",
    "么么",
  ];
  for (const k of joyKeywords) {
    if (textLower.includes(k)) result.joy += 0.2;
  }
  const joyEn = [
    " happy ",
    " glad ",
    " joy ",
    " thanks ",
    " thank you ",
    " love you ",
    " lol ",
    " haha ",
    " great ",
    " nice ",
    " awesome ",
  ];
  for (const k of joyEn) {
    if (paddedEn.includes(k)) result.joy += 0.2;
  }

  const sadnessKeywords = [
    "难过",
    "伤心",
    "哭",
    "悲伤",
    "失望",
    "沮丧",
    "委屈",
    "好累",
    "疲惫",
    "心累",
    "崩溃",
    "绝望",
    "孤单",
    "寂寞",
    "想死",
    "没意思",
  ];
  for (const k of sadnessKeywords) {
    if (textLower.includes(k)) result.sadness += 0.2;
  }
  const sadnessEn = [
    " sad ",
    " depressed ",
    " tired ",
    " lonely ",
    " upset ",
    " crying ",
  ];
  for (const k of sadnessEn) {
    if (paddedEn.includes(k)) result.sadness += 0.2;
  }

  const angerKeywords = [
    "生气",
    "愤怒",
    "讨厌",
    "烦死了",
    "烦",
    "气死",
    "恨",
    "滚",
    "闭嘴",
    "无语",
    "服了",
    "凭什么",
    "有病",
  ];
  for (const k of angerKeywords) {
    if (textLower.includes(k)) result.anger += 0.2;
  }
  const angerEn = [" angry ", " hate ", " annoyed ", " pissed ", " wtf "];
  for (const k of angerEn) {
    if (paddedEn.includes(k)) result.anger += 0.2;
  }

  const fearKeywords = ["害怕", "恐惧", "担心", "紧张", "焦虑", "慌", "不安", "吓人"];
  for (const k of fearKeywords) {
    if (textLower.includes(k)) result.fear += 0.2;
  }
  const fearEn = [
    " afraid ",
    " scared ",
    " fear ",
    " worried ",
    " anxious ",
    " nervous ",
  ];
  for (const k of fearEn) {
    if (paddedEn.includes(k)) result.fear += 0.2;
  }

  const surpriseKeywords = [
    "惊讶",
    "意外",
    "哇",
    "天哪",
    "没想到",
    "吓一跳",
    "居然",
    "真的假的",
    "诶",
  ];
  for (const k of surpriseKeywords) {
    if (textLower.includes(k)) result.surprise += 0.2;
  }
  const surpriseEn = [" wow ", " omg ", " surprised ", " unbelievable "];
  for (const k of surpriseEn) {
    if (paddedEn.includes(k)) result.surprise += 0.2;
  }

  const disgustKeywords = ["厌恶", "恶心", "反感", "厌烦", "作呕"];
  for (const k of disgustKeywords) {
    if (textLower.includes(k)) result.disgust += 0.2;
  }
  const disgustEn = [" disgusting ", " gross ", " sick of "];
  for (const k of disgustEn) {
    if (paddedEn.includes(k)) result.disgust += 0.2;
  }

  const total =
    result.joy +
    result.sadness +
    result.anger +
    result.fear +
    result.surprise +
    result.disgust;
  if (total > 0) {
    result.joy /= total;
    result.sadness /= total;
    result.anger /= total;
    result.fear /= total;
    result.surprise /= total;
    result.disgust /= total;
  } else {
    result.neutral = 1;
  }
  return result;
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
    if (msg.method === "emotion.analyze") {
      const text =
        msg.params && msg.params.text != null ? String(msg.params.text) : "";
      res.writeHead(200);
      res.end(jsonRpcResult(id, analyzeEmotion(text)));
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
