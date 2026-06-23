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
const domainDir = path.join(repoRoot, 'kernel', 'crates', 'oclive_kernel_host', 'src', 'domain');

function countPatternInFile(filePath, pattern) {
  const text = fs.readFileSync(filePath, 'utf8');
  let count = 0;
  let inTest = false;
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith('#[cfg(test)]')) {
      inTest = true;
      continue;
    }
    if (inTest && trimmed.startsWith('mod ') && trimmed.endsWith(';')) {
      continue;
    }
    if (trimmed.startsWith('//!') || trimmed.startsWith('///')) {
      continue;
    }
    if (line.includes(pattern)) {
      count += 1;
    }
  }
  return count;
}

function walkRsFiles(dir) {
  const out = [];
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      out.push(...walkRsFiles(full));
    } else if (ent.name.endsWith('.rs')) {
      out.push(full);
    }
  }
  return out;
}

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

function countDomainInfraFqRefs() {
  const pattern = 'crate::infrastructure::';
  let total = 0;
  for (const file of walkRsFiles(domainDir)) {
    const rel = path.relative(path.join(repoRoot, 'kernel', 'crates', 'oclive_kernel_host', 'src'), file);
    if (rel.replace(/\\/g, '/').includes('/tests/')) {
      continue;
    }
    const text = fs.readFileSync(file, 'utf8');
    if (text.includes('#[cfg(test)]')) {
      const parts = text.split('#[cfg(test)]');
      total += countFqInProduction(parts[0], pattern);
    } else {
      total += countFqInProduction(text, pattern);
    }
  }
  return total;
}

function countFqInProduction(text, pattern) {
  let count = 0;
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith('//!') || trimmed.startsWith('///')) {
      continue;
    }
    if (line.includes(pattern)) {
      count += 1;
    }
  }
  return count;
}

function loadBaseline() {
  const raw = fs.readFileSync(baselinePath, 'utf8');
  return JSON.parse(raw);
}

function main() {
  const useCount = countDomainInfraImports();
  const fqCount = countDomainInfraFqRefs();
  const baseline = loadBaseline();
  const maxUse = baseline.domain_to_infrastructure_imports;
  const maxFq = baseline.domain_to_infrastructure_fq_refs;

  console.log(`domain→infrastructure use imports: ${useCount} (baseline max ${maxUse})`);
  console.log(`domain→infrastructure FQ refs (prod): ${fqCount} (baseline max ${maxFq})`);

  let failed = false;
  if (useCount > maxUse) {
    console.error(
      `FAIL use: ${useCount} > ${maxUse}. Extract a port or reduce imports; update baseline only when net-decreasing.`,
    );
    failed = true;
  }
  if (fqCount > maxFq) {
    console.error(
      `FAIL FQ: ${fqCount} > ${maxFq}. Production domain must not add fully-qualified infrastructure refs.`,
    );
    failed = true;
  }
  if (failed) {
    process.exit(1);
  }

  if (useCount < maxUse) {
    console.log(
      `Ratchet down: update LAYERING_BASELINE.json domain_to_infrastructure_imports from ${maxUse} to ${useCount}.`,
    );
  }
  if (fqCount < maxFq) {
    console.log(
      `Ratchet down: update LAYERING_BASELINE.json domain_to_infrastructure_fq_refs from ${maxFq} to ${fqCount}.`,
    );
  }

  console.log('layering ratchet ok');
}

main();
