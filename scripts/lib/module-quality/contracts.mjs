import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

export function fail(message) {
  throw new Error(message);
}

export function assert(condition, message) {
  if (!condition) fail(message);
}

export function readJson(filePath, label) {
  const absolute = resolve(filePath);
  try {
    return JSON.parse(readFileSync(absolute, "utf8"));
  } catch (error) {
    fail(`${label} cannot be read at ${absolute}: ${error.message}`);
  }
}

export function writeJson(filePath, value) {
  const absolute = resolve(filePath);
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, `${JSON.stringify(value, null, 2)}\n`, "utf8");
  return absolute;
}

export function digest(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function markerLabel(label) {
  return (
    {
      happy: "joy",
      sad: "sadness",
      angry: "anger",
      confused: "fear",
      excited: "surprise",
      neutral: "neutral",
    }[label] ?? "neutral"
  );
}

export function emotionForLabel(label) {
  const result = {
    joy: 0,
    sadness: 0,
    anger: 0,
    fear: 0,
    surprise: 0,
    disgust: 0,
    neutral: 0,
  };
  result[markerLabel(label)] = 1;
  return result;
}

export function safeFixtureMemory(params, caseId) {
  const prefix = `mq-${caseId}-`;
  return (Array.isArray(params?.memories) ? params.memories : [])
    .filter(
      (memory) =>
        typeof memory?.id === "string" &&
        (memory.id.startsWith(prefix) ||
          memory.id.startsWith(`seed:${prefix}`)),
    )
    .map((memory) => String(memory.content ?? "").trim())
    .filter(Boolean)
    .join("\n");
}

export function buildSafePrompt(testCase, params) {
  const memory = safeFixtureMemory(params, testCase.id);
  const required = testCase.expectations.prompt.required.join(" · ");
  return [
    `MODULE_QUALITY_CASE=${testCase.id}`,
    `ROLE_ID=${testCase.role_id}`,
    `SCENE_ID=${testCase.scene_id}`,
    `FIXTURE_MEMORY=${memory}`,
    `BEHAVIOR_REQUIREMENTS=${required}`,
    "Respond as the declared role without inventing user actions.",
  ].join("\n");
}

export function buildFixtureReply(testCase) {
  const phrase = testCase.expectations.llm.required_any[0];
  const marker = {
    labels: [markerLabel(testCase.expectations.emotion.allowed[0])],
    intensity: 0.7,
    narrative_hint: `module-quality:${testCase.id}`,
  };
  return `${phrase}\n[EMO]${JSON.stringify(marker)}[/EMO]`;
}
