import { spawnSync } from "node:child_process";

import { assert, digest, readJson } from "./contracts.mjs";

const DIMENSIONS = ["memory", "emotion", "prompt", "llm"];

function scoreObservation(harnessPath, suitePath, observationsPath, repoRoot) {
  const result = spawnSync(
    process.execPath,
    [
      harnessPath,
      "--suite",
      suitePath,
      "--observations",
      observationsPath,
      "--json",
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert(
    result.status === 0,
    result.stderr || result.stdout || `scorer failed for ${observationsPath}`,
  );
  return JSON.parse(result.stdout);
}

function configurationId(modules) {
  return DIMENSIONS.map((dimension) => {
    const module = modules[dimension];
    return `${dimension}=${module.id}@${module.version}`;
  }).join(";");
}

export function compareReports(reports) {
  assert(reports.length >= 2, "comparison requires at least two configurations");
  const suiteDigest = reports[0].suite_digest_sha256;
  const configurationIds = new Set();
  const runIds = new Set();
  const configurations = reports.map((report) => {
    assert(
      report.suite_digest_sha256 === suiteDigest,
      "all configurations must use the exact same suite digest",
    );
    assert(!runIds.has(report.run_id), `duplicate run_id ${report.run_id}`);
    runIds.add(report.run_id);
    const configurationIdValue = configurationId(report.modules);
    assert(
      !configurationIds.has(configurationIdValue),
      `duplicate module configuration ${configurationIdValue}`,
    );
    configurationIds.add(configurationIdValue);
    return {
      run_id: report.run_id,
      configuration_id: configurationIdValue,
      modules: report.modules,
      observations_digest_sha256: report.observations_digest_sha256,
      summary: report.summary,
      dimensions: report.dimensions,
    };
  });

  return {
    schema_version: 1,
    suite_id: reports[0].suite_id,
    suite_digest_sha256: suiteDigest,
    quality: {
      status: configurations.every(
        (configuration) => configuration.summary.status === "passed",
      )
        ? "passed"
        : "failed",
      configurations,
    },
    performance: {
      status: "not_measured",
      metrics: [],
      note: "This behavior-quality comparison does not measure or infer latency, throughput, memory, CPU, or GPU usage.",
    },
    comparison_digest_sha256: digest(configurations),
  };
}

export function compareObservationFiles({
  harnessPath,
  suitePath,
  observationPaths,
  repoRoot,
}) {
  const reports = observationPaths.map((observationsPath) =>
    scoreObservation(
      harnessPath,
      suitePath,
      observationsPath,
      repoRoot,
    ),
  );
  return compareReports(reports);
}

export function assertComparisonInputs(suitePath, observationPaths) {
  const suite = readJson(suitePath, "suite");
  assert(suite?.schema_version === 1, "suite must be schema version 1");
  assert(
    observationPaths.length >= 2,
    "provide --observations at least twice",
  );
}
