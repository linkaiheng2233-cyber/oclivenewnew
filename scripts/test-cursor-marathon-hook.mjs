#!/usr/bin/env node
/** Self-test: marathon stop hook must bind after long first turns and fail open. */
import { spawnSync } from "child_process";
import fs from "fs";
import path from "path";
import process from "process";
import { fileURLToPath } from "url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cursorDir = path.join(repoRoot, ".cursor");
const statePath = path.join(cursorDir, "oclive-marathon-session.json");
const backupPath = `${statePath}.selftest-bak`;

function runHook(payload, { bom = false } = {}) {
  const body = Buffer.from(JSON.stringify(payload), "utf8");
  const input = bom
    ? Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), body])
    : body;
  return spawnSync(
    process.execPath,
    [path.join(repoRoot, ".cursor", "hooks", "oclive-marathon-stop.mjs")],
    { cwd: repoRoot, input, encoding: "utf8" },
  );
}

function writeActiveState(overrides = {}) {
  const state = {
    version: 1,
    sessionId: "selftest",
    active: true,
    conversationId: null,
    startedAt: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
    updatedAt: new Date().toISOString(),
    baseSha: "selftest",
    maxTurns: 30,
    stopTurns: 0,
    maxNoProgressTurns: 2,
    noProgressTurns: 0,
    checkpointSerial: 2,
    hookSeenCheckpointSerial: 0,
    outcome: "progress",
    current: null,
    attempts: {},
    lastCheckpoint: null,
    stopReason: null,
    ...overrides,
  };
  fs.mkdirSync(cursorDir, { recursive: true });
  fs.writeFileSync(statePath, `${JSON.stringify(state, null, 2)}\n`, "utf8");
  return state;
}

function readState() {
  return JSON.parse(fs.readFileSync(statePath, "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const hadState = fs.existsSync(statePath);
if (hadState) fs.copyFileSync(statePath, backupPath);

try {
  writeActiveState();
  const first = runHook({
    conversation_id: "conv-selftest",
    status: "completed",
    loop_count: 0,
  });
  assert(first.status === 0, `first stop exit ${first.status}: ${first.stderr}`);
  const firstOut = JSON.parse((first.stdout || "{}").trim() || "{}");
  assert(
    typeof firstOut.followup_message === "string" &&
      firstOut.followup_message.includes("OCLive debt marathon"),
    "expected followup_message after >5min first stop",
  );
  let state = readState();
  assert(state.active === true, "session should stay active");
  assert(state.conversationId === "conv-selftest", "should bind conversation");
  assert(state.stopTurns === 1, "stopTurns should increment");
  assert(state.stopReason == null, "must not expire unbound by wall clock");

  const bom = runHook(
    { conversation_id: "conv-selftest", status: "completed", loop_count: 1 },
    { bom: true },
  );
  assert(bom.status === 0, `BOM stop exit ${bom.status}: ${bom.stderr}`);
  const bomOut = JSON.parse((bom.stdout || "{}").trim() || "{}");
  assert(
    typeof bomOut.followup_message === "string",
    "BOM stdin should still continue",
  );
  state = readState();
  assert(state.active === true, "BOM path must not kill session");
  assert(state.stopTurns === 2, "second stop should count");

  // Fail-open: even if inner hook gets garbage, wrapper must not finish/kill.
  writeActiveState({ conversationId: "conv-selftest", stopTurns: 2 });
  const bad = spawnSync(
    process.execPath,
    [path.join(repoRoot, ".cursor", "hooks", "oclive-marathon-stop.mjs")],
    { cwd: repoRoot, input: Buffer.from("not-json", "utf8"), encoding: "utf8" },
  );
  assert(bad.status === 0, "wrapper must exit 0 on fail-open");
  state = readState();
  assert(state.active === true, "fail-open must not deactivate session");
  assert(state.outcome !== "failed" || state.stopReason !== "stop-hook-error", "must not finish with stop-hook-error");

  console.log("cursor-marathon hook self-test: PASS");
} finally {
  if (hadState) {
    fs.copyFileSync(backupPath, statePath);
    fs.rmSync(backupPath, { force: true });
  } else {
    fs.rmSync(statePath, { force: true });
    fs.rmSync(backupPath, { force: true });
  }
}
