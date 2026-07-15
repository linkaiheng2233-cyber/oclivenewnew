#!/usr/bin/env node
/** Cursor IDE long-running debt-marathon session coordinator. */
import { execFileSync } from "child_process";
import crypto from "crypto";
import fs from "fs";
import path from "path";
import process from "process";
import { fileURLToPath } from "url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const cursorDir = path.join(repoRoot, ".cursor");
const statePath = path.join(cursorDir, "oclive-marathon-session.json");
const lockPath = path.join(cursorDir, "oclive-marathon-session.lock");
const command = process.argv[2] ?? "status";
const VALID_CAPABILITIES = new Set([
  "local-write",
  "test",
  "commit",
  "push",
  "open-pr",
  "merge",
  "sibling-repo",
  "network",
  "secrets",
]);
const DEFAULT_CAPABILITIES = new Set(["local-write", "test"]);
const VALID_CHECKPOINT_OUTCOMES = new Set([
  "progress",
  "done",
  "blocked",
  "failed",
]);
const LEASE_MS = 30 * 60 * 1000;

function flag(name, fallback) {
  const index = process.argv.indexOf(`--${name}`);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

function git(args) {
  return execFileSync("git", args, { cwd: repoRoot, encoding: "utf8" }).trim();
}

function now() {
  return new Date().toISOString();
}

function readState() {
  if (!fs.existsSync(statePath)) return null;
  return JSON.parse(fs.readFileSync(statePath, "utf8"));
}

function writeState(state) {
  fs.mkdirSync(cursorDir, { recursive: true });
  const temp = `${statePath}.${process.pid}.${crypto.randomUUID()}.tmp`;
  fs.writeFileSync(temp, `${JSON.stringify(state, null, 2)}\n`, "utf8");
  fs.renameSync(temp, statePath);
}

async function withLock(fn) {
  fs.mkdirSync(cursorDir, { recursive: true });
  let handle;
  const ownerToken = crypto.randomUUID();
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      handle = fs.openSync(lockPath, "wx");
      try {
        fs.writeFileSync(
          handle,
          `${JSON.stringify({ ownerToken, pid: process.pid, acquiredAt: now() })}\n`,
          "utf8",
        );
      } catch (error) {
        fs.closeSync(handle);
        handle = undefined;
        fs.rmSync(lockPath, { force: true });
        throw error;
      }
      break;
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
      try {
        const ageMs = Date.now() - fs.statSync(lockPath).mtimeMs;
        if (ageMs > 120_000) {
          fs.rmSync(lockPath, { force: true });
          continue;
        }
      } catch (statError) {
        if (statError.code !== "ENOENT") throw statError;
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }
  if (handle === undefined) throw new Error("marathon session lock is busy");
  try {
    return await fn();
  } finally {
    fs.closeSync(handle);
    try {
      const lock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
      if (lock.ownerToken === ownerToken) fs.rmSync(lockPath, { force: true });
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
  }
}

function parseHookInput() {
  return new Promise((resolve, reject) => {
    let input = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      input += chunk;
    });
    process.stdin.on("end", () => {
      try {
        resolve(input.trim() ? JSON.parse(input) : {});
      } catch (error) {
        reject(error);
      }
    });
    process.stdin.on("error", reject);
  });
}

function assertCleanWorktree() {
  const dirty = git(["status", "--porcelain"]);
  if (dirty) {
    throw new Error(
      "refusing to start in a dirty worktree; use Cursor worktree mode and keep user changes in the original worktree",
    );
  }
}

function loadStageContract(debt, stage) {
  const planPath = path.join(
    repoRoot,
    "handoff",
    "debt-marathon",
    "long-plans",
    `${debt}.md`,
  );
  const markdown = fs.readFileSync(planPath, "utf8");
  const match = markdown.match(
    /<!--\s*oclive-marathon-contract\s*([\s\S]*?)-->/,
  );
  if (!match) throw new Error(`${debt}: missing machine contract`);
  const contract = JSON.parse(match[1]);
  const stageContract = contract.stages.find((item) => item.id === stage);
  if (!stageContract)
    throw new Error(`${debt}: stage ${stage} missing from contract`);
  return stageContract;
}

function changedFiles() {
  const status = execFileSync("git", ["status", "--porcelain"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  return status
    .split(/\r?\n/)
    .filter(Boolean)
    .map((line) => {
      const name = line.slice(3).trim();
      return (
        name.includes(" -> ") ? name.split(" -> ").at(-1) : name
      ).replaceAll("\\", "/");
    });
}

function pathAllowed(file, allowed) {
  const normalized = file.replaceAll("\\", "/");
  const controllerFiles = [
    "handoff/debt-marathon/MARATHON_QUEUE.md",
    "handoff/debt-marathon/waves/",
    "handoff/debt-marathon/long-plans/",
  ];
  return [...allowed, ...controllerFiles].some((entry) => {
    if (entry === "read-only") return false;
    const prefix = entry.replaceAll("\\", "/");
    return prefix.endsWith("/")
      ? normalized.startsWith(prefix)
      : normalized === prefix;
  });
}

function commandMatches(template, actual) {
  const escapedParts = template
    .split(/<[^>]+>/)
    .map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  return new RegExp(`^${escapedParts.join(".+")}$`).test(actual);
}

async function start() {
  execFileSync(process.execPath, ["scripts/check-debt-marathon.mjs"], {
    cwd: repoRoot,
    stdio: "inherit",
  });
  assertCleanWorktree();
  const maxTurns = Number(flag("max-turns", "30"));
  if (!Number.isInteger(maxTurns) || maxTurns < 2 || maxTurns > 100)
    throw new Error("--max-turns must be an integer from 2 to 100");
  await withLock(async () => {
    const existing = readState();
    if (existing?.active)
      throw new Error(`marathon already active: ${existing.sessionId}`);
    writeState({
      version: 1,
      sessionId: crypto.randomUUID(),
      active: true,
      conversationId: null,
      startedAt: now(),
      updatedAt: now(),
      baseSha: git(["rev-parse", "HEAD"]),
      maxTurns,
      stopTurns: 0,
      maxNoProgressTurns: 2,
      noProgressTurns: 0,
      checkpointSerial: 0,
      hookSeenCheckpointSerial: 0,
      outcome: "running",
      current: null,
      attempts: existing?.attempts ?? {},
      lastCheckpoint: null,
      stopReason: null,
    });
  });
  console.log(
    `Cursor marathon armed (${maxTurns} max turns). The next stop hook binds this conversation.`,
  );
}

async function claim() {
  const debt = flag("debt");
  const stage = Number(flag("stage"));
  const agent = flag("agent", "oclive-debt-stage");
  const capabilities = flag("capabilities", "local-write,test")
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  const authorizationRef = flag("authorization");
  if (!debt || !Number.isInteger(stage) || stage < 0)
    throw new Error("claim requires --debt and non-negative --stage");
  const unknown = capabilities.filter((item) => !VALID_CAPABILITIES.has(item));
  if (unknown.length)
    throw new Error(`unknown capabilities: ${unknown.join(", ")}`);
  const elevated = capabilities.filter(
    (item) => !DEFAULT_CAPABILITIES.has(item),
  );
  if (elevated.length && !authorizationRef)
    throw new Error(
      `elevated capabilities require --authorization: ${elevated.join(", ")}`,
    );
  execFileSync(
    process.execPath,
    [
      "scripts/check-debt-marathon.mjs",
      "--id",
      debt,
      "--stage",
      String(stage),
      "--require-ready",
    ],
    { cwd: repoRoot, stdio: "inherit" },
  );
  const stageContract = loadStageContract(debt, stage);
  await withLock(async () => {
    const state = readState();
    if (!state?.active) throw new Error("no active marathon session");
    if (state.current)
      throw new Error(`claim already active: ${state.current.claimId}`);
    const key = `${debt}:s${stage}`;
    state.attempts[key] = (state.attempts[key] ?? 0) + 1;
    state.current = {
      claimId: crypto.randomUUID(),
      debt,
      stage,
      agent,
      attempt: state.attempts[key],
      claimedAt: now(),
      heartbeatAt: now(),
      leaseExpiresAt: new Date(Date.now() + LEASE_MS).toISOString(),
      baseSha: git(["rev-parse", "HEAD"]),
      baselineChangedFiles: changedFiles(),
      capabilities,
      authorizationRef: authorizationRef ?? null,
      stageContract,
    };
    state.updatedAt = now();
    writeState(state);
    console.log(JSON.stringify(state.current, null, 2));
  });
}

async function heartbeat() {
  const claimId = flag("claim");
  await withLock(async () => {
    const state = readState();
    if (!state?.active || !state.current || state.current.claimId !== claimId)
      throw new Error("heartbeat does not match an active claim");
    state.current.heartbeatAt = now();
    state.current.leaseExpiresAt = new Date(
      Date.now() + LEASE_MS,
    ).toISOString();
    state.updatedAt = now();
    writeState(state);
  });
  console.log("marathon heartbeat recorded");
}

async function recover() {
  const claimId = flag("claim");
  const action = flag("action", "release");
  const reason = flag("reason");
  if (!["release", "block"].includes(action) || !reason)
    throw new Error(
      "recover requires --claim, --action release|block and --reason",
    );
  await withLock(async () => {
    const state = readState();
    if (!state?.active || !state.current || state.current.claimId !== claimId)
      throw new Error("recover does not match an active claim");
    state.lastRecovery = { at: now(), claim: state.current, action, reason };
    state.current = null;
    state.outcome = action === "block" ? "blocked" : "running";
    state.updatedAt = now();
    writeState(state);
  });
  console.log(`marathon claim recovered: ${action}`);
}

async function checkpoint() {
  const debt = flag("debt");
  const stage = Number(flag("stage"));
  const claimId = flag("claim");
  const outcome = flag("outcome", "progress");
  const wave = flag("wave");
  const nextCommand = flag("next");
  const lastCommand = flag("last-command");
  if (
    !claimId ||
    !debt ||
    !Number.isInteger(stage) ||
    stage < 0 ||
    !wave ||
    !nextCommand ||
    !lastCommand
  ) {
    throw new Error(
      "checkpoint requires --claim, --debt, --stage, --wave, --last-command and --next",
    );
  }
  if (!VALID_CHECKPOINT_OUTCOMES.has(outcome))
    throw new Error(`invalid checkpoint outcome: ${outcome}`);
  const wavePath = path.resolve(repoRoot, wave);
  if (
    !wavePath.startsWith(
      path.join(repoRoot, "handoff", "debt-marathon", "waves") + path.sep,
    ) ||
    !fs.existsSync(wavePath)
  ) {
    throw new Error(
      "--wave must reference an existing file under handoff/debt-marathon/waves",
    );
  }
  await withLock(async () => {
    const state = readState();
    if (!state?.active) throw new Error("no active marathon session");
    if (
      !state.current ||
      state.current.claimId !== claimId ||
      state.current.debt !== debt ||
      state.current.stage !== stage
    ) {
      throw new Error("checkpoint does not match the active claim");
    }
    const completedClaim = state.current;
    const currentHead = git(["rev-parse", "HEAD"]);
    try {
      execFileSync(
        "git",
        ["merge-base", "--is-ancestor", completedClaim.baseSha, currentHead],
        { cwd: repoRoot, stdio: "ignore" },
      );
    } catch {
      throw new Error("current HEAD is not descended from the claim base SHA");
    }
    const currentChangedFiles = changedFiles();
    const newChangedFiles = currentChangedFiles.filter(
      (file) => !completedClaim.baselineChangedFiles.includes(file),
    );
    const scopeViolations = newChangedFiles.filter(
      (file) => !pathAllowed(file, completedClaim.stageContract.files),
    );
    if (scopeViolations.length)
      throw new Error(
        `changed files outside Stage scope: ${scopeViolations.join(", ")}`,
      );
    const matchingCheck = completedClaim.stageContract.checks.some((check) =>
      commandMatches(check.command, lastCommand),
    );
    if (!matchingCheck)
      throw new Error(
        "--last-command must match one applicable Stage contract check",
      );
    const waveText = fs.readFileSync(wavePath, "utf8");
    if (!waveText.includes(claimId))
      throw new Error("Wave must contain the active claim id");
    if (fs.statSync(wavePath).mtimeMs < Date.parse(completedClaim.claimedAt))
      throw new Error("Wave predates the active claim");
    state.checkpointSerial += 1;
    state.updatedAt = now();
    state.outcome = outcome;
    state.lastCheckpoint = {
      at: now(),
      claimId,
      attempt: completedClaim.attempt,
      debt,
      stage,
      outcome,
      baseSha: completedClaim.baseSha,
      headSha: currentHead,
      changedFiles: currentChangedFiles,
      wave: path.relative(repoRoot, wavePath).replaceAll("\\", "/"),
      lastCommand,
      nextExactCommand: nextCommand,
    };
    state.current = null;
    writeState(state);
  });
  console.log("marathon checkpoint recorded");
}

async function finish() {
  const outcome = flag("outcome", "done");
  const reason = flag("reason", outcome);
  await withLock(async () => {
    const state = readState();
    if (!state) throw new Error("no marathon session");
    if (!VALID_CHECKPOINT_OUTCOMES.has(outcome))
      throw new Error(`invalid finish outcome: ${outcome}`);
    if (outcome === "done" && state.current)
      throw new Error(
        `cannot finish done with active claim ${state.current.claimId}`,
      );
    state.active = false;
    state.outcome = outcome;
    state.stopReason = reason;
    state.updatedAt = now();
    writeState(state);
  });
  console.log(`marathon stopped: ${reason}`);
}

async function hook() {
  const input = await parseHookInput();
  const output = await withLock(async () => {
    const state = readState();
    if (!state?.active) return {};
    if (!input.conversation_id || !input.status) {
      state.active = false;
      state.outcome = "failed";
      state.stopReason = "Cursor stop hook omitted conversation_id or status";
      state.updatedAt = now();
      writeState(state);
      return {};
    }
    if (state.conversationId && state.conversationId !== input.conversation_id)
      return {};
    if (!state.conversationId) {
      if (Date.now() - Date.parse(state.startedAt) > 5 * 60 * 1000) {
        state.active = false;
        state.outcome = "failed";
        state.stopReason = "unbound Cursor session expired";
        state.updatedAt = now();
        writeState(state);
        return {};
      }
      state.conversationId = input.conversation_id;
    }
    if (input.status !== "completed") {
      state.active = false;
      state.outcome = "failed";
      state.stopReason = `Cursor stop status: ${input.status}`;
      state.updatedAt = now();
      writeState(state);
      return {};
    }
    if (
      state.current &&
      Date.now() > Date.parse(state.current.leaseExpiresAt)
    ) {
      state.current.orphanedAt = now();
      state.updatedAt = now();
      writeState(state);
      return {
        followup_message: `[OCLive debt marathon] Claim ${state.current.claimId} lease expired. Reconcile its worktree and run cursor-marathon recover; do not dispatch a replacement before recovery.`,
      };
    }
    state.stopTurns += 1;
    const progressed = state.checkpointSerial > state.hookSeenCheckpointSerial;
    state.noProgressTurns = progressed ? 0 : state.noProgressTurns + 1;
    state.hookSeenCheckpointSerial = state.checkpointSerial;
    state.updatedAt = now();
    if (["done", "blocked", "failed"].includes(state.outcome)) {
      state.active = false;
      state.stopReason = `terminal checkpoint: ${state.outcome}`;
      writeState(state);
      return {};
    }
    if (
      state.stopTurns >= state.maxTurns ||
      state.noProgressTurns >= state.maxNoProgressTurns
    ) {
      state.active = false;
      state.outcome = "failed";
      state.stopReason =
        state.stopTurns >= state.maxTurns
          ? "max turns reached"
          : "no-progress fuse opened";
      writeState(state);
      return {};
    }
    writeState(state);
    return {
      followup_message: `[OCLive debt marathon ${state.stopTurns + 1}/${state.maxTurns}] Continue as the parent controller. Read .cursor/oclive-marathon-session.json and the last Wave. Validate the previous subagent result before dispatching exactly one next Stage. Never stash/switch/reset the shared worktree. Record a checkpoint before stopping; finish explicitly on done, blocked, or failure.`,
    };
  });
  process.stdout.write(JSON.stringify(output));
}

async function status() {
  const state = readState();
  console.log(state ? JSON.stringify(state, null, 2) : "no marathon session");
}

try {
  if (command === "start") await start();
  else if (command === "claim") await claim();
  else if (command === "heartbeat") await heartbeat();
  else if (command === "recover") await recover();
  else if (command === "checkpoint") await checkpoint();
  else if (command === "finish") await finish();
  else if (command === "hook") await hook();
  else if (command === "status") await status();
  else throw new Error(`unknown command: ${command}`);
} catch (error) {
  console.error(`cursor-marathon: ${error.message}`);
  process.exit(1);
}
