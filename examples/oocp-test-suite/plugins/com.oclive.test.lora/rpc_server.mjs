import http from "node:http";

const protocolHeader = "x-oclive-remote-protocol";
const protocolValue = "oclive-remote-jsonrpc-v1";

function result(id, text) {
  return JSON.stringify({ jsonrpc: "2.0", id, result: { text } });
}

function error(id, code, message) {
  return JSON.stringify({ jsonrpc: "2.0", id, error: { code, message } });
}

function streamResult(id, event, fields = {}) {
  return `${JSON.stringify({
    jsonrpc: "2.0",
    id,
    result: { event, ...fields },
  })}\n`;
}

const server = http.createServer((request, response) => {
  if (request.method !== "POST" || !request.url?.startsWith("/rpc")) {
    response.writeHead(404);
    response.end();
    return;
  }
  const chunks = [];
  request.on("data", (chunk) => chunks.push(chunk));
  request.on("end", () => {
    let message;
    try {
      message = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    } catch {
      response.writeHead(400, { "Content-Type": "application/json" });
      response.end(error(null, -32700, "parse error"));
      return;
    }

    response.setHeader("Content-Type", "application/json");
    response.setHeader(protocolHeader, protocolValue);
    if (message.method === "llm.generate_stream") {
      const prompt = String(message.params?.prompt || "");
      const id = message.id ?? null;
      response.writeHead(200, {
        "Content-Type": "application/x-ndjson; charset=utf-8",
      });
      response.write(streamResult(id, "token", { text: "lora-" }));
      setTimeout(() => {
        if (prompt.includes("force-lora-partial-failure")) {
          response.end(`${error(id, -32603, "forced partial failure")}\n`);
          return;
        }
        response.write(streamResult(id, "token", { text: "adapter-selected" }));
        response.end(streamResult(id, "done"));
      }, 25);
      return;
    }
    if (message.method === "llm.generate") {
      if (String(message.params?.prompt || "").includes("force-lora-failure")) {
        response.end(error(message.id ?? null, -32603, "forced LoRA failure"));
        return;
      }
      response.end(result(message.id ?? null, "lora-adapter-selected"));
      return;
    }
    if (message.method === "llm.generate_tag") {
      response.end(result(message.id ?? null, "neutral"));
      return;
    }
    response.end(error(message.id ?? null, -32601, "method not found"));
  });
});

server.listen(0, "127.0.0.1", () => {
  const address = server.address();
  const port = typeof address === "object" && address ? address.port : 0;
  process.stdout.write(`OCLIVE_READY http://127.0.0.1:${port}/rpc\n`);
});

process.on("SIGTERM", () => server.close());
process.on("SIGINT", () => server.close());
