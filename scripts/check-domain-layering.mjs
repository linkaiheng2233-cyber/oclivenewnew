#!/usr/bin/env node
/**
 * Ratchet: domain → infrastructure imports in oclive_kernel_host must not increase.
 * Baseline: handoff/LAYERING_BASELINE.json
 */
import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const baselinePath = path.join(repoRoot, 'handoff', 'LAYERING_BASELINE.json');
const domainDir = path.join(repoRoot, 'crates', 'oclive_kernel_host', 'src', 'domain');

function countDomainInfraImports() {
  const pattern = 'use crate::infrastructure';
  const out = execFileSync(
    'rg',
    ['--glob', '*.rs', '-c', pattern, domainDir],
    { encoding: 'utf8', cwd: repoRoot },
  ).trim();
  if (!out) {
    return 0;
  }
  let total = 0;
  for (const line of out.split('\n')) {
    const m = line.match(/:(\d+)$/);
    if (m) {
      total += Number(m[1]);
    }
  }
  return total;
}

function loadBaseline() {
  const raw = fs.readFileSync(baselinePath, 'utf8');
  return JSON.parse(raw);
}

function main() {
  const count = countDomainInfraImports();
  const baseline = loadBaseline();
  const max = baseline.domain_to_infrastructure_imports;

  console.log(`domain→infrastructure imports: ${count} (baseline max ${max})`);

  if (count > max) {
    console.error(
      `FAIL: ${count} > ${max}. New domain→infra imports are not allowed; extract a port or reduce imports and update baseline only when net-decreasing.`,
    );
    process.exit(1);
  }

  if (count < max) {
    console.log(
      `Ratchet down: update handoff/LAYERING_BASELINE.json domain_to_infrastructure_imports from ${max} to ${count}.`,
    );
  }

  console.log('layering ratchet ok');
}

main();
