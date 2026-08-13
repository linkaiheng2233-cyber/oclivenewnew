#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveRepoRoot } from "./lib/chat-pro-roles-dir.mjs";
import { captureObservations } from "./lib/module-quality/capture.mjs";
import {
  assert,
  buildFixtureReply,
  buildSafePrompt,
  fail,
  readJson,
  writeJson,
} from "./lib/module-quality/contracts.mjs";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolveRepoRoot();
const DEFAULT_SUITE = join(
  REPO_ROOT,
  "examples",
  "module-quality-harness",
  "fixtures",
  "suite.v1.json",
);

function valueAfter(args, flag) {
  const index = args.indexOf(flag);
  return index >= 0 ? args[index + 1] : undefined;
}

function runScorer(suitePath, observationsPath) {
  const harness = join(SCRIPT_DIR, "module-quality-harness.mjs");
  const result = spawnSync(
    process.execPath,
    [
      harness,
      "--suite",
      suitePath,
      "--observations",
      observationsPath,
      "--json",
    ],
    { cwd: REPO_ROOT, encoding: "utf8" },
  );
  if (result.status !== 0) {
    fail(result.stderr || result.stdout || "module-quality scorer failed");
  }
  return JSON.parse(result.stdout);
}

function selfTest() {
  const testCase = {
    id: "safe-case",
    role_id: "role",
    scene_id: "scene",
    replay: [{ speaker: "user", text: "secret user text" }],
    expectations: {
      memory: { required: ["fixture fact"], forbidden: ["private"] },
      emotion: { allowed: ["happy"] },
      prompt: { required: ["role rule"], forbidden: ["private"] },
      llm: {
        required_any: ["safe answer"],
        forbidden: ["secret user text"],
        max_user_echo_ratio: 0.5,
      },
    },
  };
  const params = {
    memories: [
      { id: "mq-safe-case-1", content: "fixture fact" },
      { id: "real-private-memory", content: "private" },
    ],
  };
  const prompt = buildSafePrompt(testCase, params);
  assert(prompt.includes("fixture fact"), "safe fixture memory must be kept");
  assert(!prompt.includes("private"), "non-fixture memory must be redacted");
  const reply = buildFixtureReply(testCase);
  assert(reply.includes("safe answer"), "fixture reply must satisfy contract");
  assert(
    !reply.includes("secret user text"),
    "fixture reply must not echo user text",
  );
  console.log("module-quality-runner self-test: PASS");
}

async function main() {
  const args = process.argv.slice(2);
  if (args.includes("--self-test")) {
    selfTest();
    return;
  }
  const suitePath = resolve(valueAfter(args, "--suite") ?? DEFAULT_SUITE);
  const outputPath = valueAfter(args, "--output");
  const suite = readJson(suitePath, "suite");
  assert(
    suite?.schema_version === 1 && Array.isArray(suite.cases),
    "suite must be a v1 module-quality suite",
  );
  const observations = await captureObservations(suite, REPO_ROOT);
  const tempOutput = outputPath
    ? null
    : join(
        mkdtempSync(join(tmpdir(), "oclive-module-quality-report-")),
        "observations.json",
      );
  const observationsPath = writeJson(outputPath ?? tempOutput, observations);
  try {
    const report = runScorer(suitePath, observationsPath);
    console.log(JSON.stringify({ observations, report }, null, 2));
  } finally {
    if (tempOutput) {
      rmSync(dirname(tempOutput), { recursive: true, force: true });
    }
  }
}

main().catch((error) => {
  console.error(`module-quality-runner: FAIL\n${error.stack ?? error.message}`);
  process.exitCode = 1;
});
