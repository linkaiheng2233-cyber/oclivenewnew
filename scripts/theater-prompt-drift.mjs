#!/usr/bin/env node
/**
 * Compare official plugin vs Rust builtin patch prompt key substrings (drift guard).
 * Run: node scripts/theater-prompt-drift.mjs
 */
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { buildTheaterPrompt, DRIFT_MARKERS } from "../distros/chat-pro/plugins/com.oclive.theater_director_official/prompts/index.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pluginDir = path.join(repoRoot, "distros/chat-pro/plugins/com.oclive.theater_director_official");

const sampleInput = {
  mode: "patch",
  cast_a_name: "木木",
  cast_b_name: "枫侵月",
  cast_a_role_id: "mumu",
  cast_b_role_id: "feng",
  scene_id: "home",
  theater_scene: "breakfast",
  persona_a: "傲娇",
  persona_b: "温柔",
  patch_max_lines: 3,
  patch_variant: 1,
  patch_tweak: {
    lead_cast: "a",
    chip_label: "苦中药",
    drama_seed: "苦药变笑料",
  },
  patch_prefix_beats: [
    { id: "b1", cast: "b", name: "枫侵月", text: "快吃。" },
    { id: "b2", cast: "a", name: "木木", text: "知道了。" },
  ],
  patch_canned_patch: [{ id: "tea-1", cast: "a", name: "木木", text: "（罐头）" }],
};

const pluginPrompt = buildTheaterPrompt(sampleInput);
const markers = [
  DRIFT_MARKERS.patchTitle,
  DRIFT_MARKERS.guardrailsHeader,
  "性格反差",
  "第二版候选",
  "苦药变笑料",
];

let failed = false;
for (const m of markers) {
  if (!pluginPrompt.includes(m)) {
    console.error(`plugin prompt missing marker: ${m}`);
    failed = true;
  }
}

const rustTest = spawnSync(
  "cargo",
  [
    "test",
    "-p",
    "oclive_kernel_host",
    "build_patch_prompt_includes_lead_and_variant",
    "--",
    "--nocapture",
  ],
  { cwd: repoRoot, encoding: "utf8", shell: true },
);

if (rustTest.status !== 0) {
  console.error(rustTest.stdout);
  console.error(rustTest.stderr);
  failed = true;
} else {
  console.log("Rust builtin patch prompt markers: OK");
}

console.log(`Plugin prompt length: ${pluginPrompt.length} (from ${pluginDir})`);
if (failed) {
  process.exit(1);
}
console.log("theater-prompt-drift: all markers OK");
