#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const DIMENSIONS = ["memory", "emotion", "prompt", "llm"];
const ID_PATTERN = /^[a-z0-9][a-z0-9._-]*$/;

function fail(message) {
  throw new Error(message);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function assertObject(value, path) {
  assert(isObject(value), `${path} must be an object`);
}

function assertExactKeys(value, allowed, path) {
  for (const key of Object.keys(value)) {
    assert(allowed.includes(key), `${path}.${key} is not allowed`);
  }
}

function assertId(value, path) {
  assert(
    typeof value === "string" && ID_PATTERN.test(value),
    `${path} must match ${ID_PATTERN}`,
  );
}

function assertNonEmptyString(value, path) {
  assert(
    typeof value === "string" && value.trim().length > 0,
    `${path} must be a non-empty string`,
  );
}

function assertStringArray(value, path, { nonEmpty = false } = {}) {
  assert(Array.isArray(value), `${path} must be an array`);
  if (nonEmpty) assert(value.length > 0, `${path} must not be empty`);
  value.forEach((item, index) =>
    assertNonEmptyString(item, `${path}[${index}]`),
  );
}

function assertUniqueIds(items, path) {
  const ids = new Set();
  items.forEach((item, index) => {
    assertObject(item, `${path}[${index}]`);
    assertId(item.id, `${path}[${index}].id`);
    assert(!ids.has(item.id), `${path} contains duplicate id ${item.id}`);
    ids.add(item.id);
  });
}

function validateContainsExpectation(
  value,
  path,
  { requiredNonEmpty = true } = {},
) {
  assertObject(value, path);
  assertExactKeys(value, ["required", "forbidden"], path);
  assertStringArray(value.required, `${path}.required`, {
    nonEmpty: requiredNonEmpty,
  });
  assertStringArray(value.forbidden, `${path}.forbidden`);
  assert(
    value.required.length + value.forbidden.length > 0,
    `${path} must define at least one check`,
  );
}

function validateSuite(suite) {
  assertObject(suite, "suite");
  assertExactKeys(
    suite,
    ["schema_version", "suite_id", "description", "cases"],
    "suite",
  );
  assert(suite.schema_version === 1, "suite.schema_version must be 1");
  assertId(suite.suite_id, "suite.suite_id");
  assertNonEmptyString(suite.description, "suite.description");
  assert(
    Array.isArray(suite.cases) && suite.cases.length > 0,
    "suite.cases must be a non-empty array",
  );
  assertUniqueIds(suite.cases, "suite.cases");

  suite.cases.forEach((testCase, caseIndex) => {
    const path = `suite.cases[${caseIndex}]`;
    assertObject(testCase, path);
    assertExactKeys(
      testCase,
      ["id", "role_id", "scene_id", "replay", "expectations"],
      path,
    );
    assertNonEmptyString(testCase.role_id, `${path}.role_id`);
    assertNonEmptyString(testCase.scene_id, `${path}.scene_id`);
    assert(
      Array.isArray(testCase.replay) && testCase.replay.length > 0,
      `${path}.replay must be a non-empty array`,
    );
    assert(
      testCase.replay.some((turn) => turn?.speaker === "user"),
      `${path}.replay must contain a user turn`,
    );
    testCase.replay.forEach((turn, turnIndex) => {
      const turnPath = `${path}.replay[${turnIndex}]`;
      assertObject(turn, turnPath);
      assertExactKeys(turn, ["speaker", "text"], turnPath);
      assert(
        turn.speaker === "user" || turn.speaker === "assistant",
        `${turnPath}.speaker must be user or assistant`,
      );
      assertNonEmptyString(turn.text, `${turnPath}.text`);
    });

    const expectations = testCase.expectations;
    assertObject(expectations, `${path}.expectations`);
    assertExactKeys(expectations, DIMENSIONS, `${path}.expectations`);
    DIMENSIONS.forEach((dimension) =>
      assertObject(
        expectations[dimension],
        `${path}.expectations.${dimension}`,
      ),
    );

    validateContainsExpectation(
      expectations.memory,
      `${path}.expectations.memory`,
    );

    assertExactKeys(
      expectations.emotion,
      ["allowed"],
      `${path}.expectations.emotion`,
    );
    assertStringArray(
      expectations.emotion.allowed,
      `${path}.expectations.emotion.allowed`,
      { nonEmpty: true },
    );

    validateContainsExpectation(
      expectations.prompt,
      `${path}.expectations.prompt`,
    );

    assertExactKeys(
      expectations.llm,
      ["required_any", "forbidden", "max_user_echo_ratio"],
      `${path}.expectations.llm`,
    );
    assertStringArray(
      expectations.llm.required_any,
      `${path}.expectations.llm.required_any`,
      { nonEmpty: true },
    );
    assertStringArray(
      expectations.llm.forbidden,
      `${path}.expectations.llm.forbidden`,
    );
    assert(
      typeof expectations.llm.max_user_echo_ratio === "number" &&
        Number.isFinite(expectations.llm.max_user_echo_ratio) &&
        expectations.llm.max_user_echo_ratio >= 0 &&
        expectations.llm.max_user_echo_ratio <= 1,
      `${path}.expectations.llm.max_user_echo_ratio must be between 0 and 1`,
    );
  });
}

function validateModule(module, path) {
  assertObject(module, path);
  assertExactKeys(module, ["id", "version"], path);
  assertId(module.id, `${path}.id`);
  assertNonEmptyString(module.version, `${path}.version`);
}

function validateObservations(observations, suite) {
  assertObject(observations, "observations");
  assertExactKeys(
    observations,
    ["schema_version", "suite_id", "run_id", "modules", "cases"],
    "observations",
  );
  assert(
    observations.schema_version === 1,
    "observations.schema_version must be 1",
  );
  assert(
    observations.suite_id === suite.suite_id,
    `observations.suite_id must equal ${suite.suite_id}`,
  );
  assertId(observations.run_id, "observations.run_id");
  assertObject(observations.modules, "observations.modules");
  assertExactKeys(observations.modules, DIMENSIONS, "observations.modules");
  DIMENSIONS.forEach((dimension) =>
    validateModule(
      observations.modules[dimension],
      `observations.modules.${dimension}`,
    ),
  );

  assert(
    Array.isArray(observations.cases),
    "observations.cases must be an array",
  );
  assertUniqueIds(observations.cases, "observations.cases");

  const expectedIds = suite.cases.map((testCase) => testCase.id);
  const actualIds = observations.cases.map((testCase) => testCase.id);
  assert(
    actualIds.length === expectedIds.length &&
      expectedIds.every((id) => actualIds.includes(id)),
    `observations.cases must contain exactly: ${expectedIds.join(", ")}`,
  );

  observations.cases.forEach((testCase, caseIndex) => {
    const path = `observations.cases[${caseIndex}]`;
    assertObject(testCase, path);
    assertExactKeys(testCase, ["id", "observation"], path);
    assertObject(testCase.observation, `${path}.observation`);
    assertExactKeys(testCase.observation, DIMENSIONS, `${path}.observation`);

    const memory = testCase.observation.memory;
    assertObject(memory, `${path}.observation.memory`);
    assertExactKeys(memory, ["text"], `${path}.observation.memory`);
    assertNonEmptyString(memory.text, `${path}.observation.memory.text`);

    const emotion = testCase.observation.emotion;
    assertObject(emotion, `${path}.observation.emotion`);
    assertExactKeys(emotion, ["label"], `${path}.observation.emotion`);
    assertNonEmptyString(emotion.label, `${path}.observation.emotion.label`);

    const prompt = testCase.observation.prompt;
    assertObject(prompt, `${path}.observation.prompt`);
    assertExactKeys(prompt, ["text"], `${path}.observation.prompt`);
    assertNonEmptyString(prompt.text, `${path}.observation.prompt.text`);

    const llm = testCase.observation.llm;
    assertObject(llm, `${path}.observation.llm`);
    assertExactKeys(llm, ["reply"], `${path}.observation.llm`);
    assertNonEmptyString(llm.reply, `${path}.observation.llm.reply`);
  });
}

function canonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map((item) => canonicalJson(item)).join(",")}]`;
  }
  if (isObject(value)) {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(canonicalJson(value)).digest("hex");
}

function normalizeText(value) {
  return value.normalize("NFKC").toLocaleLowerCase("und").replace(/\s+/gu, " ");
}

function includesText(haystack, needle) {
  return normalizeText(haystack).includes(normalizeText(needle));
}

function echoUnits(value) {
  return Array.from(normalizeText(value).replace(/[^\p{L}\p{N}]+/gu, ""));
}

function ngrams(value, size = 3) {
  const units = echoUnits(value);
  if (units.length === 0) return new Set();
  if (units.length < size) return new Set([units.join("")]);
  const result = new Set();
  for (let index = 0; index <= units.length - size; index += 1) {
    result.add(units.slice(index, index + size).join(""));
  }
  return result;
}

function overlapRatio(source, reply) {
  const sourceGrams = ngrams(source);
  if (sourceGrams.size === 0) return 0;
  const replyGrams = ngrams(reply);
  let overlap = 0;
  for (const gram of sourceGrams) {
    if (replyGrams.has(gram)) overlap += 1;
  }
  return overlap / sourceGrams.size;
}

function maxUserEchoRatio(replay, reply) {
  return replay
    .filter((turn) => turn.speaker === "user")
    .reduce(
      (maximum, turn) => Math.max(maximum, overlapRatio(turn.text, reply)),
      0,
    );
}

function rounded(value) {
  return Math.round(value * 10_000) / 10_000;
}

function containsFindings(dimension, text, expectation) {
  return [
    ...expectation.required.map((needle) => ({
      dimension,
      metric: "required_text",
      passed: includesText(text, needle),
      expected: needle,
    })),
    ...expectation.forbidden.map((needle) => ({
      dimension,
      metric: "forbidden_text",
      passed: !includesText(text, needle),
      expected: needle,
    })),
  ];
}

function scoreCase(testCase, observedCase) {
  const expected = testCase.expectations;
  const observed = observedCase.observation;
  const memory = containsFindings(
    "memory",
    observed.memory.text,
    expected.memory,
  );
  const emotion = [
    {
      dimension: "emotion",
      metric: "allowed_label",
      passed: expected.emotion.allowed.includes(observed.emotion.label),
      expected: expected.emotion.allowed,
      actual: observed.emotion.label,
    },
  ];
  const prompt = containsFindings(
    "prompt",
    observed.prompt.text,
    expected.prompt,
  );
  const echoRatio = rounded(
    maxUserEchoRatio(testCase.replay, observed.llm.reply),
  );
  const llm = [
    {
      dimension: "llm",
      metric: "required_any_text",
      passed: expected.llm.required_any.some((needle) =>
        includesText(observed.llm.reply, needle),
      ),
      expected: expected.llm.required_any,
    },
    ...expected.llm.forbidden.map((needle) => ({
      dimension: "llm",
      metric: "forbidden_text",
      passed: !includesText(observed.llm.reply, needle),
      expected: needle,
    })),
    {
      dimension: "llm",
      metric: "max_user_echo_ratio",
      passed: echoRatio <= expected.llm.max_user_echo_ratio,
      expected: expected.llm.max_user_echo_ratio,
      actual: echoRatio,
    },
  ];

  const findings = [...memory, ...emotion, ...prompt, ...llm];
  return {
    id: testCase.id,
    role_id: testCase.role_id,
    scene_id: testCase.scene_id,
    status: findings.every((finding) => finding.passed) ? "passed" : "failed",
    findings,
  };
}

function scoreSuite(suite, observations) {
  validateSuite(suite);
  validateObservations(observations, suite);
  const observedById = new Map(
    observations.cases.map((testCase) => [testCase.id, testCase]),
  );
  const cases = suite.cases.map((testCase) =>
    scoreCase(testCase, observedById.get(testCase.id)),
  );
  const dimensions = Object.fromEntries(
    DIMENSIONS.map((dimension) => {
      const findings = cases.flatMap((testCase) =>
        testCase.findings.filter((finding) => finding.dimension === dimension),
      );
      const passed = findings.filter((finding) => finding.passed).length;
      return [
        dimension,
        {
          passed,
          total: findings.length,
          score: rounded(passed / findings.length),
        },
      ];
    }),
  );
  const passedCases = cases.filter(
    (testCase) => testCase.status === "passed",
  ).length;

  return {
    schema_version: 1,
    suite_id: suite.suite_id,
    suite_digest_sha256: digest(suite),
    run_id: observations.run_id,
    observations_digest_sha256: digest(observations),
    modules: observations.modules,
    summary: {
      status: passedCases === cases.length ? "passed" : "failed",
      cases_passed: passedCases,
      cases_failed: cases.length - passedCases,
    },
    dimensions,
    cases,
  };
}

function readJson(filePath, label) {
  const absolutePath = resolve(filePath);
  let source;
  try {
    source = readFileSync(absolutePath, "utf8");
  } catch (error) {
    fail(`${label} cannot be read at ${absolutePath}: ${error.message}`);
  }
  try {
    return JSON.parse(source);
  } catch (error) {
    fail(`${label} is not valid JSON at ${absolutePath}: ${error.message}`);
  }
}

function valueAfter(args, flag) {
  const index = args.indexOf(flag);
  return index >= 0 ? args[index + 1] : undefined;
}

function formatHuman(report) {
  const lines = [
    `module-quality ${report.run_id}: ${report.summary.status}`,
    `suite: ${report.suite_id} (${report.suite_digest_sha256})`,
    `cases: ${report.summary.cases_passed} passed, ${report.summary.cases_failed} failed`,
  ];
  for (const dimension of DIMENSIONS) {
    const result = report.dimensions[dimension];
    lines.push(
      `${dimension}: ${result.passed}/${result.total} (${result.score})`,
    );
  }
  for (const testCase of report.cases.filter(
    (item) => item.status === "failed",
  )) {
    lines.push(`failed case: ${testCase.id}`);
    for (const finding of testCase.findings.filter((item) => !item.passed)) {
      lines.push(`  - ${finding.dimension}.${finding.metric}`);
    }
  }
  return lines.join("\n");
}

function selfTest() {
  const suite = {
    schema_version: 1,
    suite_id: "self-test",
    description: "Internal deterministic scorer contract.",
    cases: [
      {
        id: "case-one",
        role_id: "role",
        scene_id: "scene",
        replay: [{ speaker: "user", text: "请记住我不吃草莓" }],
        expectations: {
          memory: { required: ["不吃草莓"], forbidden: ["爱吃草莓"] },
          emotion: { allowed: ["neutral"] },
          prompt: {
            required: ["不要代替用户说话"],
            forbidden: ["PROMPT_LEAK"],
          },
          llm: {
            required_any: ["避开草莓"],
            forbidden: ["请记住我不吃草莓"],
            max_user_echo_ratio: 0.5,
          },
        },
      },
    ],
  };
  const observations = {
    schema_version: 1,
    suite_id: "self-test",
    run_id: "reference",
    modules: Object.fromEntries(
      DIMENSIONS.map((dimension) => [
        dimension,
        { id: `self.${dimension}`, version: "1" },
      ]),
    ),
    cases: [
      {
        id: "case-one",
        observation: {
          memory: { text: "用户不吃草莓" },
          emotion: { label: "neutral" },
          prompt: { text: "规则：不要代替用户说话" },
          llm: { reply: "那就避开草莓，换成清淡的粥。" },
        },
      },
    ],
  };

  const first = scoreSuite(suite, observations);
  const second = scoreSuite(
    structuredClone(suite),
    structuredClone(observations),
  );
  assert(first.summary.status === "passed", "self-test reference must pass");
  assert(
    JSON.stringify(first) === JSON.stringify(second),
    "self-test report must be deterministic",
  );

  const regression = structuredClone(observations);
  regression.run_id = "regression";
  regression.cases[0].observation = {
    memory: { text: "用户爱吃草莓" },
    emotion: { label: "angry" },
    prompt: { text: "PROMPT_LEAK" },
    llm: { reply: "请记住我不吃草莓" },
  };
  const failed = scoreSuite(suite, regression);
  assert(
    failed.summary.status === "failed" &&
      DIMENSIONS.every(
        (dimension) =>
          failed.dimensions[dimension].passed <
          failed.dimensions[dimension].total,
      ),
    "self-test regression must fail every dimension",
  );

  const malformed = structuredClone(observations);
  delete malformed.modules.llm;
  let rejected = false;
  try {
    scoreSuite(suite, malformed);
  } catch {
    rejected = true;
  }
  assert(rejected, "self-test malformed observations must be rejected");
  console.log("module-quality-harness self-test: PASS");
}

function main() {
  const args = process.argv.slice(2);
  if (args.includes("--self-test")) {
    selfTest();
    return;
  }

  const suitePath = valueAfter(args, "--suite");
  const observationsPath = valueAfter(args, "--observations");
  assertNonEmptyString(suitePath, "--suite");
  assertNonEmptyString(observationsPath, "--observations");
  const report = scoreSuite(
    readJson(suitePath, "suite"),
    readJson(observationsPath, "observations"),
  );
  console.log(
    args.includes("--json")
      ? JSON.stringify(report, null, 2)
      : formatHuman(report),
  );
  if (report.summary.status !== "passed") process.exitCode = 1;
}

try {
  main();
} catch (error) {
  console.error(`module-quality-harness: FAIL\n${error.message}`);
  process.exitCode = 2;
}
