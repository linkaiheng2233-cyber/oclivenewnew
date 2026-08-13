import { createServer } from "node:http";

import {
  assert,
  buildFixtureReply,
  buildSafePrompt,
  emotionForLabel,
  fail,
  safeFixtureMemory,
} from "./contracts.mjs";

export const PROTOCOL_HEADER = "oclive-remote-jsonrpc-v1";
const MAX_REQUEST_BYTES = 8 * 1024 * 1024;

export function createObservationSidecar(suite) {
  const cases = new Map(suite.cases.map((testCase) => [testCase.id, testCase]));
  const state = { activeCaseId: null, captures: new Map() };

  function activeCase() {
    const testCase = cases.get(state.activeCaseId);
    assert(testCase, "sidecar has no active fixture case");
    return testCase;
  }

  function captureFor(testCase) {
    let capture = state.captures.get(testCase.id);
    if (!capture) {
      capture = {
        memory: "",
        memoryIds: new Set(),
        prompt: "",
        methods: new Set(),
      };
      state.captures.set(testCase.id, capture);
    }
    return capture;
  }

  function dispatch(request) {
    assert(request?.jsonrpc === "2.0", "invalid JSON-RPC version");
    assert(Number.isInteger(request?.id), "invalid JSON-RPC id");
    assert(typeof request?.method === "string", "invalid JSON-RPC method");
    const testCase = activeCase();
    const capture = captureFor(testCase);
    capture.methods.add(request.method);

    switch (request.method) {
      case "memory.rank": {
        const memories = Array.isArray(request.params?.memories)
          ? request.params.memories
          : [];
        const prefix = `mq-${testCase.id}-`;
        const isFixture = (memory) =>
          typeof memory?.id === "string" &&
          (memory.id.startsWith(prefix) ||
            memory.id.startsWith(`seed:${prefix}`));
        const ordered = [
          ...memories.filter(isFixture),
          ...memories.filter((memory) => !isFixture(memory)),
        ];
        for (const memory of memories) {
          if (typeof memory?.id === "string") capture.memoryIds.add(memory.id);
        }
        const fixtureMemory = safeFixtureMemory(request.params, testCase.id);
        if (fixtureMemory) capture.memory = fixtureMemory;
        return { ordered_ids: ordered.map((memory) => memory.id) };
      }
      case "emotion.analyze":
        return emotionForLabel(testCase.expectations.emotion.allowed[0]);
      case "prompt.top_topic_hint":
        return { hint: null };
      case "prompt.build_prompt": {
        const prompt = buildSafePrompt(testCase, request.params);
        const fixtureMemory = safeFixtureMemory(request.params, testCase.id);
        if (fixtureMemory) capture.memory = fixtureMemory;
        capture.prompt = prompt;
        return { prompt };
      }
      case "llm.generate":
      case "llm.generate_stream":
        return { text: buildFixtureReply(testCase) };
      case "llm.generate_tag":
        return { text: "{}" };
      default:
        fail(`unsupported observation-sidecar method: ${request.method}`);
    }
  }

  const server = createServer((request, response) => {
    if (request.method !== "POST") {
      response.writeHead(405).end();
      return;
    }
    if (request.headers["x-oclive-remote-protocol"] !== PROTOCOL_HEADER) {
      response
        .writeHead(400, { "content-type": "application/json" })
        .end(JSON.stringify({ error: "missing protocol header" }));
      return;
    }
    let body = "";
    request.setEncoding("utf8");
    request.on("data", (chunk) => {
      body += chunk;
      if (Buffer.byteLength(body) > MAX_REQUEST_BYTES) request.destroy();
    });
    request.on("end", () => {
      try {
        const rpc = JSON.parse(body);
        const result = dispatch(rpc);
        response
          .writeHead(200, { "content-type": "application/json" })
          .end(JSON.stringify({ jsonrpc: "2.0", id: rpc.id, result }));
      } catch (error) {
        response.writeHead(200, { "content-type": "application/json" }).end(
          JSON.stringify({
            jsonrpc: "2.0",
            id: null,
            error: { code: -32603, message: error.message },
          }),
        );
      }
    });
  });

  return {
    server,
    setActiveCase(caseId) {
      assert(cases.has(caseId), `unknown case ${caseId}`);
      state.activeCaseId = caseId;
    },
    capture(caseId) {
      return state.captures.get(caseId);
    },
  };
}
