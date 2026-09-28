#!/usr/bin/env node
/**
 * Ratchet: imports of runtime engine modules via `crate::domain::*` in oclive_kernel_host
 * must not increase. New code should use `oclive_kernel_runtime::domain::*` directly.
 * Baseline: handoff/HOST_REEXPORT_BASELINE.json
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { countMatchingLines, runGate } from './lib/gate-toolchain.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const baselinePath = path.join(repoRoot, 'handoff', 'HOST_REEXPORT_BASELINE.json');
const hostCrate = path.join(repoRoot, 'kernel', 'crates', 'oclive_kernel_host');

const RUNTIME_MODULES =
  'affect_policy|builtin_reply_post_processor|chat_llm_fallback|chat_turn|chat_turn_rules|' +
  'complex_emotion|emotion_analyzer|event_detector|knowledge_loader|life_schedule|' +
  'local_plugin_bridge|local_plugin_memory_pick|memory_engine|memory_retrieval|' +
  'personality_engine|policy|profile_personality|prompt_assembler|prompt_builder|' +
  'relation_engine|remote_life_prompt|repository|user_emotion_analyzer';

// Require a complete module segment so a host-owned sibling such as
// `complex_emotion_store` is not mistaken for the `complex_emotion` re-export.
const pattern = `use crate::domain::(?:${RUNTIME_MODULES})(?:::|\\s*;)`;

function countHostReexportImports() {
  const domainMod = path.join(hostCrate, 'src', 'domain', 'mod.rs');
  return countMatchingLines(pattern, [hostCrate], { cwd: repoRoot, excludeFiles: [domainMod] });
}

function loadBaseline() {
  return JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
}

function main() {
  const count = countHostReexportImports();
  const baseline = loadBaseline();
  const max = baseline.host_runtime_reexport_imports;

  console.log(`host runtime re-export imports: ${count} (baseline max ${max})`);

  if (count > max) {
    console.error(
      `FAIL: ${count} > ${max}. Import runtime engines from oclive_kernel_runtime::domain instead.`,
    );
    process.exit(1);
  }

  if (count < max) {
    console.log(
      `Ratchet down: update handoff/HOST_REEXPORT_BASELINE.json host_runtime_reexport_imports from ${max} to ${count}.`,
    );
  }

  console.log('host re-export ratchet ok');
}

runGate(main);
