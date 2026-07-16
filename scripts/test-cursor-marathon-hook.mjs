#!/usr/bin/env node
/** Self-test: marathon stop hook must bind after long first turns and fail open. */
import { spawnSync } from "child_process";
import fs from "fs";
import os from "os";
import path from "path";
import process from "process";
import { fileURLToPath } from "url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
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

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    ...options,
  });
  return result;
}

function git(cwd, args) {
  const result = run("git", args, { cwd });
  assert(
    result.status === 0,
    `git ${args.join(" ")} failed: ${result.stderr || result.stdout}`,
  );
  return result.stdout.trim();
}

function writeFixture(progress = "pending") {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oclive-marathon-test-"));
  fs.mkdirSync(path.join(root, "scripts"), { recursive: true });
  fs.mkdirSync(path.join(root, "handoff", "debt-marathon", "long-plans"), {
    recursive: true,
  });
  fs.mkdirSync(path.join(root, "handoff", "debt-marathon", "waves"), {
    recursive: true,
  });
  fs.copyFileSync(
    path.join(repoRoot, "scripts", "check-debt-marathon.mjs"),
    path.join(root, "scripts", "check-debt-marathon.mjs"),
  );
  fs.copyFileSync(
    path.join(repoRoot, "scripts", "cursor-marathon.mjs"),
    path.join(root, "scripts", "cursor-marathon.mjs"),
  );
  fs.writeFileSync(
    path.join(root, ".gitignore"),
    ".cursor/oclive-marathon-session.json\n.cursor/oclive-marathon-session.lock\n.cursor/oclive-marathon-hook.log\n",
    "utf8",
  );
  fs.writeFileSync(
    path.join(root, "handoff", "debt-marathon", "MARATHON_QUEUE.md"),
    `| seq | debt | runner | plan | progress |\n|---|---|---|---|---|\n| 10 | TEST-DEBT | auto | [long-plans/TEST-DEBT.md](./long-plans/TEST-DEBT.md) | ${progress} |\n`,
    "utf8",
  );
  fs.writeFileSync(
    path.join(root, "handoff", "debt-marathon", "long-plans", "TEST-DEBT.md"),
    `# TEST-DEBT\n\n[mandatory gates](../AI_AND_PIPELINE_GATES.md)\n\n<!-- oclive-marathon-contract\n${JSON.stringify(
      {
        version: 1,
        id: "TEST-DEBT",
        runner: "auto",
        planStatus: "ready",
        parentDebtDisposition: "keep-open",
        currentStage: 0,
        prerequisites: [],
        stages: [
          {
            id: 0,
            title: "Fixture stage",
            files: ["allowed.txt"],
            actions: ["Change the allowed fixture"],
            checks: [
              { command: "node verify.mjs", why: "Fixture verification" },
            ],
            outputs: ["Fixture output"],
            rollback: "Remove fixture output",
          },
        ],
      },
      null,
      2,
    )}\n-->\n`,
    "utf8",
  );
  git(root, ["init"]);
  git(root, ["config", "user.email", "marathon-test@example.invalid"]);
  git(root, ["config", "user.name", "Marathon Test"]);
  git(root, ["add", "."]);
  git(root, ["commit", "-m", "fixture"]);
  return root;
}

function runCoordinator(root, args) {
  return run(
    process.execPath,
    [path.join(root, "scripts", "cursor-marathon.mjs"), ...args],
    { cwd: root },
  );
}

function claimIdFrom(result) {
  const match = result.stdout.match(/\{\s*"claimId"[\s\S]*\}\s*$/);
  assert(match, `claim JSON missing: ${result.stdout} ${result.stderr}`);
  return JSON.parse(match[0]).claimId;
}

function updateFixtureForDone(root, claimId) {
  const queue = path.join(
    root,
    "handoff",
    "debt-marathon",
    "MARATHON_QUEUE.md",
  );
  fs.writeFileSync(
    queue,
    fs.readFileSync(queue, "utf8").replace("| pending |", "| done |"),
    "utf8",
  );
  const plan = path.join(
    root,
    "handoff",
    "debt-marathon",
    "long-plans",
    "TEST-DEBT.md",
  );
  fs.writeFileSync(
    plan,
    fs
      .readFileSync(plan, "utf8")
      .replace('"planStatus": "ready"', '"planStatus": "closed"'),
    "utf8",
  );
  fs.writeFileSync(path.join(root, "allowed.txt"), "done\n", "utf8");
  fs.writeFileSync(
    path.join(root, "handoff", "debt-marathon", "waves", "WAVE-TEST.md"),
    `claim ${claimId}\n`,
    "utf8",
  );
  git(root, ["add", "."]);
  git(root, ["commit", "-m", "complete fixture stage"]);
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
  assert(
    first.status === 0,
    `first stop exit ${first.status}: ${first.stderr}`,
  );
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
  assert(
    state.outcome !== "failed" || state.stopReason !== "stop-hook-error",
    "must not finish with stop-hook-error",
  );

  let fixture = writeFixture();
  try {
    let result = runCoordinator(fixture, ["start", "--max-turns", "4"]);
    assert(result.status === 0, `fixture start failed: ${result.stderr}`);
    result = runCoordinator(fixture, [
      "claim",
      "--debt",
      "TEST-DEBT",
      "--stage",
      "0",
    ]);
    assert(result.status === 0, `fixture claim failed: ${result.stderr}`);
    const claimId = claimIdFrom(result);
    fs.writeFileSync(
      path.join(fixture, "outside.txt"),
      "outside scope\n",
      "utf8",
    );
    git(fixture, ["add", "outside.txt"]);
    git(fixture, ["commit", "-m", "out of scope"]);
    fs.writeFileSync(
      path.join(fixture, "handoff", "debt-marathon", "waves", "WAVE-TEST.md"),
      `claim ${claimId}\n`,
      "utf8",
    );
    result = runCoordinator(fixture, [
      "checkpoint",
      "--claim",
      claimId,
      "--debt",
      "TEST-DEBT",
      "--stage",
      "0",
      "--outcome",
      "progress",
      "--wave",
      "handoff/debt-marathon/waves/WAVE-TEST.md",
      "--last-command",
      "node verify.mjs",
      "--next",
      "stop",
    ]);
    assert(
      result.status !== 0,
      "committed out-of-scope file must fail checkpoint",
    );
    assert(
      result.stderr.includes("outside.txt"),
      `scope failure must name outside.txt: ${result.stderr}`,
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }

  fixture = writeFixture("pr-open");
  try {
    let result = runCoordinator(fixture, ["start", "--max-turns", "4"]);
    assert(
      result.status === 0,
      `pr-open fixture start failed: ${result.stderr}`,
    );
    result = runCoordinator(fixture, [
      "claim",
      "--debt",
      "TEST-DEBT",
      "--stage",
      "0",
    ]);
    assert(result.status !== 0, "pr-open queue row must not be claimable");
    assert(result.stderr.includes("not runnable"), result.stderr);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }

  fixture = writeFixture("pendng");
  try {
    const result = runCoordinator(fixture, ["start", "--max-turns", "4"]);
    assert(result.status !== 0, "unknown queue progress must fail closed");
    assert(
      result.stderr.includes("invalid auto queue progress"),
      result.stderr,
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }

  fixture = writeFixture();
  try {
    let result = runCoordinator(fixture, ["start", "--max-turns", "4"]);
    assert(result.status === 0, `done fixture start failed: ${result.stderr}`);
    result = runCoordinator(fixture, ["finish", "--outcome", "done"]);
    assert(result.status !== 0, "done without terminal checkpoint must fail");
    assert(result.stderr.includes("terminal done checkpoint"), result.stderr);

    result = runCoordinator(fixture, [
      "claim",
      "--debt",
      "TEST-DEBT",
      "--stage",
      "0",
    ]);
    assert(result.status === 0, `done fixture claim failed: ${result.stderr}`);
    const claimId = claimIdFrom(result);
    updateFixtureForDone(fixture, claimId);
    result = runCoordinator(fixture, [
      "checkpoint",
      "--claim",
      claimId,
      "--debt",
      "TEST-DEBT",
      "--stage",
      "0",
      "--outcome",
      "done",
      "--wave",
      "handoff/debt-marathon/waves/WAVE-TEST.md",
      "--last-command",
      "node verify.mjs",
      "--next",
      "finish done",
    ]);
    assert(result.status === 0, `terminal checkpoint failed: ${result.stderr}`);
    result = runCoordinator(fixture, ["finish", "--outcome", "done"]);
    assert(result.status === 0, `validated finish failed: ${result.stderr}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }

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
