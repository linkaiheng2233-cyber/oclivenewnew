#!/usr/bin/env node

import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveRepoRoot } from "./lib/chat-pro-roles-dir.mjs";
import {
  assertComparisonInputs,
  compareObservationFiles,
  compareReports,
} from "./lib/module-quality/comparison.mjs";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolveRepoRoot();
const DEFAULT_SUITE = join(
  REPO_ROOT,
  "examples",
  "module-quality-harness",
  "fixtures",
  "suite.v1.json",
);

function valuesAfter(args, flag) {
  return args.flatMap((value, index) =>
    value === flag && args[index + 1] ? [args[index + 1]] : [],
  );
}

function valueAfter(args, flag) {
  return valuesAfter(args, flag)[0];
}

function fixtureReport(runId, suffix) {
  const dimensions = Object.fromEntries(
    ["memory", "emotion", "prompt", "llm"].map((dimension) => [
      dimension,
      { passed: 1, total: 1, score: 1 },
    ]),
  );
  return {
    suite_id: "self-test",
    suite_digest_sha256: "suite-digest",
    run_id: runId,
    observations_digest_sha256: `${runId}-digest`,
    modules: Object.fromEntries(
      ["memory", "emotion", "prompt", "llm"].map((dimension) => [
        dimension,
        { id: `self.${dimension}.${suffix}`, version: "1" },
      ]),
    ),
    summary: { status: "passed", cases_passed: 1, cases_failed: 0 },
    dimensions,
  };
}

function selfTest() {
  const comparison = compareReports([
    fixtureReport("reference", "a"),
    fixtureReport("candidate", "b"),
  ]);
  if (
    comparison.quality.status !== "passed" ||
    comparison.quality.configurations.length !== 2 ||
    comparison.performance.status !== "not_measured"
  ) {
    throw new Error("comparison self-test did not preserve report boundaries");
  }
  let rejected = false;
  try {
    compareReports([fixtureReport("only", "a")]);
  } catch {
    rejected = true;
  }
  if (!rejected) throw new Error("single-configuration comparison was accepted");
  console.log("module-quality-compare self-test: PASS");
}

function main() {
  const args = process.argv.slice(2);
  if (args.includes("--self-test")) {
    selfTest();
    return;
  }
  const suitePath = resolve(valueAfter(args, "--suite") ?? DEFAULT_SUITE);
  const observationPaths = valuesAfter(args, "--observations").map((value) =>
    resolve(value),
  );
  assertComparisonInputs(suitePath, observationPaths);
  const comparison = compareObservationFiles({
    harnessPath: join(SCRIPT_DIR, "module-quality-harness.mjs"),
    suitePath,
    observationPaths,
    repoRoot: REPO_ROOT,
  });
  console.log(JSON.stringify(comparison, null, 2));
  if (comparison.quality.status !== "passed") process.exitCode = 1;
}

try {
  main();
} catch (error) {
  console.error(`module-quality-compare: FAIL\n${error.message}`);
  process.exitCode = 2;
}
