#!/usr/bin/env node
/**
 * Validate Cursor debt-marathon queue entries and machine contracts.
 *
 * A Ready auto plan must embed one JSON object in:
 *   <!-- oclive-marathon-contract
 *   { ... }
 *   -->
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const marathonRoot = path.join(repoRoot, "handoff", "debt-marathon");
const queuePath = path.join(marathonRoot, "MARATHON_QUEUE.md");
const onlyId = valueAfter("--id");
const onlyStage = valueAfter("--stage");
const requireReady = process.argv.includes("--require-ready");
const VALID_PLAN_STATES = new Set(["ready", "blocked", "closed"]);
const VALID_PARENT_DISPOSITIONS = new Set(["keep-open", "done-eligible"]);

function valueAfter(flag) {
  const index = process.argv.indexOf(flag);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function fail(message) {
  throw new Error(message);
}

function parseQueue(markdown) {
  const rows = [];
  for (const line of markdown.split(/\r?\n/)) {
    const cells = line
      .split("|")
      .slice(1, -1)
      .map((cell) => cell.trim());
    if (cells.length !== 5 || !/^\d+$/.test(cells[0])) continue;
    const link = cells[3].match(/\[[^\]]+\]\(([^)]+)\)/);
    if (!link) fail(`queue row ${cells[0]} has no plan link`);
    rows.push({
      seq: Number(cells[0]),
      id: cells[1],
      runner: cells[2],
      plan: link[1],
      progress: cells[4],
    });
  }
  if (new Set(rows.map((row) => row.seq)).size !== rows.length)
    fail("queue has duplicate seq values");
  if (new Set(rows.map((row) => row.id)).size !== rows.length)
    fail("queue has duplicate debt ids");
  return rows;
}

function parseContract(markdown, planPath) {
  const match = markdown.match(
    /<!--\s*oclive-marathon-contract\s*([\s\S]*?)-->/,
  );
  if (!match) fail(`${planPath}: missing oclive-marathon-contract JSON block`);
  try {
    return JSON.parse(match[1]);
  } catch (error) {
    fail(`${planPath}: invalid contract JSON: ${error.message}`);
  }
}

function nonEmptyStrings(value) {
  return (
    Array.isArray(value) &&
    value.length > 0 &&
    value.every((item) => typeof item === "string" && item.trim())
  );
}

function validateStage(stage, planPath) {
  const prefix = `${planPath}: stage ${stage?.id ?? "?"}`;
  if (!Number.isInteger(stage?.id) || stage.id < 0)
    fail(`${prefix}: id must be a non-negative integer`);
  if (typeof stage.title !== "string" || !stage.title.trim())
    fail(`${prefix}: title is required`);
  if (!nonEmptyStrings(stage.files))
    fail(
      `${prefix}: files must be a non-empty string array (use "read-only" when applicable)`,
    );
  if (!nonEmptyStrings(stage.actions))
    fail(`${prefix}: actions must be a non-empty string array`);
  if (!Array.isArray(stage.checks) || stage.checks.length === 0)
    fail(`${prefix}: checks are required`);
  for (const check of stage.checks) {
    if (typeof check?.command !== "string" || !check.command.trim())
      fail(`${prefix}: every check needs command`);
    if (typeof check?.why !== "string" || !check.why.trim())
      fail(`${prefix}: every check needs why`);
  }
  if (!nonEmptyStrings(stage.outputs))
    fail(`${prefix}: outputs must be a non-empty string array`);
  if (typeof stage.rollback !== "string" || !stage.rollback.trim())
    fail(`${prefix}: rollback is required`);
}

function validateContract(contract, row, planPath) {
  if (contract.version !== 1) fail(`${planPath}: contract version must be 1`);
  if (contract.id !== row.id)
    fail(`${planPath}: contract id ${contract.id} != queue id ${row.id}`);
  if (contract.runner !== row.runner)
    fail(
      `${planPath}: contract runner ${contract.runner} != queue runner ${row.runner}`,
    );
  if (!VALID_PLAN_STATES.has(contract.planStatus))
    fail(`${planPath}: invalid planStatus ${contract.planStatus}`);
  if (!VALID_PARENT_DISPOSITIONS.has(contract.parentDebtDisposition)) {
    fail(
      `${planPath}: invalid parentDebtDisposition ${contract.parentDebtDisposition}`,
    );
  }
  if (!Number.isInteger(contract.currentStage) || contract.currentStage < 0)
    fail(`${planPath}: currentStage must be a non-negative integer`);
  if (!Array.isArray(contract.prerequisites))
    fail(`${planPath}: prerequisites must be an array`);
  if (
    !contract.prerequisites.every(
      (item) => typeof item === "string" && item.trim(),
    )
  )
    fail(`${planPath}: prerequisites must contain non-empty strings`);
  if (!Array.isArray(contract.stages) || contract.stages.length === 0)
    fail(`${planPath}: stages are required`);
  const ids = contract.stages.map((stage) => stage.id);
  if (new Set(ids).size !== ids.length)
    fail(`${planPath}: duplicate stage ids`);
  for (const stage of contract.stages) validateStage(stage, planPath);
  if (!ids.includes(contract.currentStage) && contract.planStatus === "ready") {
    fail(`${planPath}: currentStage ${contract.currentStage} is absent`);
  }
  if (
    contract.planStatus === "blocked" &&
    !nonEmptyStrings(contract.prerequisites)
  ) {
    fail(`${planPath}: blocked plan needs an explicit prerequisite`);
  }
  if (contract.planStatus === "ready" && contract.prerequisites.length > 0) {
    fail(`${planPath}: ready plan cannot retain unmet prerequisites`);
  }
  if (contract.planStatus === "closed" && row.progress !== "done")
    fail(`${planPath}: closed plan requires queue progress done`);
  if (contract.planStatus === "blocked" && !row.progress.startsWith("blocked:"))
    fail(
      `${planPath}: blocked plan requires a stable blocked:<code> queue state`,
    );
  if (
    contract.planStatus === "ready" &&
    (row.progress === "done" || row.progress.startsWith("blocked:"))
  ) {
    fail(
      `${planPath}: ready plan conflicts with queue progress ${row.progress}`,
    );
  }
}

function main() {
  if (!fs.existsSync(queuePath))
    fail(`missing ${path.relative(repoRoot, queuePath)}`);
  const rows = parseQueue(fs.readFileSync(queuePath, "utf8"));
  const autoRows = rows.filter(
    (row) => row.runner === "auto" && (!onlyId || row.id === onlyId),
  );
  if (onlyId && autoRows.length !== 1)
    fail(`auto queue id not found: ${onlyId}`);
  if (!onlyId && autoRows.length === 0) fail("queue contains no auto plans");

  for (const row of autoRows) {
    const planPath = path.resolve(marathonRoot, row.plan);
    if (
      !planPath.startsWith(path.join(marathonRoot, "long-plans") + path.sep)
    ) {
      fail(`${row.id}: plan must stay under handoff/debt-marathon/long-plans`);
    }
    if (!fs.existsSync(planPath))
      fail(`${row.id}: missing plan ${path.relative(repoRoot, planPath)}`);
    const markdown = fs.readFileSync(planPath, "utf8");
    if (!markdown.includes("../AI_AND_PIPELINE_GATES.md"))
      fail(`${row.id}: plan does not link the mandatory gates`);
    const contract = parseContract(markdown, path.relative(repoRoot, planPath));
    validateContract(contract, row, path.relative(repoRoot, planPath));
    if (requireReady && contract.planStatus !== "ready")
      fail(`${row.id}: plan is ${contract.planStatus}, not ready`);
    if (onlyStage !== undefined) {
      const stage = Number(onlyStage);
      if (
        !Number.isInteger(stage) ||
        !contract.stages.some((item) => item.id === stage)
      )
        fail(`${row.id}: stage ${onlyStage} is absent`);
      if (requireReady && contract.currentStage !== stage)
        fail(
          `${row.id}: stage ${stage} is not currentStage ${contract.currentStage}`,
        );
    }
  }

  console.log(`debt-marathon contracts: PASS (${autoRows.length} auto plans)`);
}

try {
  main();
} catch (error) {
  console.error(`debt-marathon contracts: FAIL\n${error.message}`);
  process.exit(1);
}
