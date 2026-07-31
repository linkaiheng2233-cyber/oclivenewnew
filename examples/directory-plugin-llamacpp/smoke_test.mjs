import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
let observedModel = "";

const upstream = http.createServer((request, response) => {
  const chunks = [];
  request.on("data", (chunk) => chunks.push(chunk));
  request.on("end", () => {
    const body = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    observedModel = String(body.model || "");
    if (body.stream === true) {
      response.setHeader("Content-Type", "text/event-stream");
      response.write(
        `data: ${JSON.stringify({ choices: [{ delta: { content: "adapter-" } }] })}\n\n`
      );
      setImmediate(() => {
        response.write(
          `data: ${JSON.stringify({ choices: [{ delta: { content: "stream" } }] })}\n\n`
        );
        response.end("data: [DONE]\n\n");
      });
      return;
    }
    response.setHeader("Content-Type", "application/json");
    response.end(
      JSON.stringify({
        choices: [{ message: { content: "adapter-response" } }],
      })
    );
  });
});

await new Promise((resolve) => upstream.listen(0, "127.0.0.1", resolve));
const upstreamAddress = upstream.address();
const upstreamPort =
  typeof upstreamAddress === "object" && upstreamAddress
    ? upstreamAddress.port
    : 0;

const plugin = spawn(process.execPath, ["rpc_server.mjs"], {
  cwd: root,
  env: {
    ...process.env,
    OCLIVE_PLUGIN_CONFIG: JSON.stringify({
      base_url: `http://127.0.0.1:${upstreamPort}`,
      adapter_model: "mumu-lora",
    }),
  },
  stdio: ["ignore", "pipe", "inherit"],
});

try {
  const rpcUrl = await new Promise((resolve, reject) => {
    let pending = "";
    const timeout = setTimeout(
      () => reject(new Error("plugin ready timeout")),
      5_000
    );
    plugin.once("error", reject);
    plugin.stdout.on("data", (chunk) => {
      pending += chunk.toString("utf8");
      const match = pending.match(/OCLIVE_READY\s+(\S+)/);
      if (match) {
        clearTimeout(timeout);
        resolve(match[1]);
      }
    });
  });

  const response = await fetch(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "llm.generate",
      params: {
        model: "base-model",
        prompt: "hello",
      },
    }),
  });
  const body = await response.json();

  assert.equal(body.result.text, "adapter-response");
  assert.equal(observedModel, "mumu-lora");

  const streamResponse = await fetch(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 2,
      method: "llm.generate_stream",
      params: {
        model: "base-model",
        prompt: "hello stream",
      },
    }),
  });
  const streamLines = (await streamResponse.text())
    .trim()
    .split(/\r?\n/)
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    streamLines.map((line) => line.result),
    [
      { event: "token", text: "adapter-" },
      { event: "token", text: "stream" },
      { event: "done" },
    ]
  );
  assert.equal(observedModel, "mumu-lora");
  process.stdout.write("LoRA directory plugin smoke: OK\n");
} finally {
  plugin.kill();
  await new Promise((resolve) => upstream.close(resolve));
}
