import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  buildPresetFromRolePack,
  readReplyQualityAnchor,
  truncateExcerpt,
} from "./preset_builder.mjs";
import { shouldPolish } from "./polish_rules.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const FENGQINYUE_DIR = path.resolve(__dirname, "../../distros/chat-pro/roles/枫侵月");

test("shouldPolish detects user echo at opening", () => {
  assert.equal(
    shouldPolish("你好呀，今天怎么样？", "你好呀，今天怎么样"),
    true,
  );
});

test("shouldPolish detects markdown code fence", () => {
  assert.equal(shouldPolish("```js\nconsole.log(1)\n```", "hi"), true);
});

test("shouldPolish skips clean short reply", () => {
  assert.equal(shouldPolish("烦死了，别烦我。", "你在干嘛"), false);
});

test("shouldPolish skips empty reply", () => {
  assert.equal(shouldPolish("   ", "hello"), false);
});

test("shouldPolish detects overly long reply", () => {
  assert.equal(shouldPolish("a".repeat(1300), "hi"), true);
});

test("truncateExcerpt adds ellipsis when over limit", () => {
  const out = truncateExcerpt("abcdef", 3);
  assert.match(out, /^abc…$/);
});

test("readReplyQualityAnchor reads fengqinyue blueprint", () => {
  const anchor = readReplyQualityAnchor(path.join(FENGQINYUE_DIR, "pipeline.ocblueprint"));
  assert.ok(anchor);
  assert.match(anchor, /回复质量锚点/);
  assert.match(anchor, /温柔/);
});

test("buildPresetFromRolePack includes fengqinyue personality excerpt and anchor", () => {
  const preset = buildPresetFromRolePack(FENGQINYUE_DIR, { maxExcerpt: 200 });
  assert.match(preset, /【角色润色器 · 非扮演】/);
  assert.match(preset, /枫侵月/);
  assert.match(preset, /【不可违背】/);
  assert.match(preset, /回复质量锚点/);
  assert.ok(preset.length < 2000);
});

test("buildPresetFromRolePack prefers polish_prompt.md when present", () => {
  const tmp = fs.mkdtempSync(path.join(path.dirname(__dirname), "polish-test-"));
  try {
    fs.writeFileSync(path.join(tmp, "polish_prompt.md"), "CUSTOM PRESET ONLY");
    fs.writeFileSync(path.join(tmp, "core_personality.txt"), "ignored");
    const preset = buildPresetFromRolePack(tmp);
    assert.equal(preset, "CUSTOM PRESET ONLY");
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
});
