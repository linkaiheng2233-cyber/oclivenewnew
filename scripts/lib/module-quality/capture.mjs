import { randomUUID } from "node:crypto";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { findKernelBinary } from "../e2e-binary.mjs";
import { assert, digest } from "./contracts.mjs";
import { prepareFixtureRoles } from "./fixture-roles.mjs";
import {
  closeServer,
  configureRemoteLlm,
  freePort,
  importReplayHistory,
  listen,
  postChat,
  spawnKernel,
  stopProcessTree,
  waitForKernel,
} from "./kernel-client.mjs";
import { createObservationSidecar } from "./observation-sidecar.mjs";

function assertCaptureComplete(testCase, capture) {
  assert(capture, `case ${testCase.id} produced no sidecar capture`);
  const traversed = [...capture.methods].sort().join(", ");
  for (const method of [
    "memory.rank",
    "emotion.analyze",
    "prompt.build_prompt",
  ]) {
    assert(
      capture.methods.has(method),
      `case ${testCase.id} did not traverse ${method}; observed: ${traversed}`,
    );
  }
  assert(
    capture.methods.has("llm.generate") ||
      capture.methods.has("llm.generate_stream"),
    `case ${testCase.id} did not traverse an LLM generation method; observed: ${traversed}`,
  );
  assert(
    capture.memory,
    `case ${testCase.id} captured no fixture memory; ids: ${[...capture.memoryIds].sort().join(", ") || "(none)"}`,
  );
  assert(capture.prompt, `case ${testCase.id} captured no safe prompt`);
}

export async function captureObservations(suite, repoRoot) {
  const binary = findKernelBinary(repoRoot);
  assert(
    binary,
    "no kernel binary found; run cargo build -p oclive-kernel-server first",
  );
  const tempRoot = mkdtempSync(join(tmpdir(), "oclive-module-quality-"));
  const rolesRoot = prepareFixtureRoles(suite, tempRoot, repoRoot);
  const sidecar = createObservationSidecar(suite);
  const sidecarPort = await freePort();
  const kernelPort = await freePort();
  const sidecarUrl = `http://127.0.0.1:${sidecarPort}/rpc`;
  const baseUrl = `http://127.0.0.1:${kernelPort}`;
  const apiToken = randomUUID();
  let child;
  let stderr = "";

  try {
    await listen(sidecar.server, sidecarPort);
    child = spawnKernel({
      binary,
      port: kernelPort,
      rolesRoot,
      appDataRoot: join(tempRoot, "app-data"),
      sidecarUrl,
      apiToken,
      repoRoot,
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
      if (stderr.length > 32_000) stderr = stderr.slice(-32_000);
    });
    await waitForKernel(baseUrl, child, () => stderr);
    await configureRemoteLlm(baseUrl, apiToken, sidecarUrl, suite.cases[0]);

    const observations = [];
    for (const testCase of suite.cases) {
      sidecar.setActiveCase(testCase.id);
      await importReplayHistory(baseUrl, apiToken, testCase);
      const response = await postChat(
        baseUrl,
        apiToken,
        rolesRoot,
        testCase,
      );
      const capture = sidecar.capture(testCase.id);
      assertCaptureComplete(testCase, capture);
      observations.push({
        id: testCase.id,
        observation: {
          memory: { text: capture.memory },
          emotion: { label: response.bot_emotion },
          prompt: { text: capture.prompt },
          llm: { reply: response.reply },
        },
      });
    }

    return {
      schema_version: 1,
      suite_id: suite.suite_id,
      run_id: `kernel-remote-adapter-${digest(suite).slice(0, 12)}`,
      modules: {
        memory: { id: "oclive.remote.memory", version: "stage2-v1" },
        emotion: { id: "oclive.remote.emotion", version: "stage2-v1" },
        prompt: { id: "oclive.remote.prompt", version: "stage2-v1" },
        llm: { id: "oclive.remote.llm", version: "stage2-v1" },
      },
      cases: observations,
    };
  } finally {
    await stopProcessTree(child);
    if (sidecar.server.listening) await closeServer(sidecar.server);
    rmSync(tempRoot, { recursive: true, force: true });
  }
}
