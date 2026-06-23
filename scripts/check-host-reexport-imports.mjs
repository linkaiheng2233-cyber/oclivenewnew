#!/usr/bin/env node
/**
 * Ratchet: imports of runtime engine modules via `crate::domain::*` in oclive_kernel_host
 * must not increase. New code should use `oclive_kernel_runtime::domain::*` directly.
 * Baseline: handoff/HOST_REEXPORT_BASELINE.json
 */
import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

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

const pattern = `use crate::domain::(?:${RUNTIME_MODULES})`;

function countHostReexportImports() {
  const domainMod = path.join(hostCrate, 'src', 'domain', 'mod.rs');
  const out = execFileSync(
    'rg',
    ['--glob', '*.rs', '-c', pattern, hostCrate],
    { encoding: 'utf8', cwd: repoRoot },
  ).trim();
  if (!out) return 0;
  let total = 0;
  for (const line of out.split('\n')) {
    const file = line.split(':')[0]?.replace(/\\/g, '/');
    if (file === domainMod.replace(/\\/g, '/')) continue;
    const m = line.match(/:(\d+)$/);
    if (m) total += Number(m[1]);
  }
  return total;
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

main();
