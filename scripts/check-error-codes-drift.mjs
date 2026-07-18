#!/usr/bin/env node
/**
 * Dimension 5 gate: kernel static error codes == frontend enum == ERROR_CODES.md markers.
 *
 * Usage: node scripts/check-error-codes-drift.mjs
 */
import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

function readKernelCodesFromRust() {
  const out = execFileSync(
    'cargo',
    ['test', '-p', 'oclive_kernel_types', 'export_kernel_error_codes_json', '--', '--nocapture'],
    { cwd: repoRoot, encoding: 'utf8' },
  );
  const marker = 'KERNEL_ERROR_CODES_JSON:';
  const line = out.split('\n').find(l => l.includes(marker));
  if (!line) {
    throw new Error('export_kernel_error_codes_json did not emit KERNEL_ERROR_CODES_JSON line');
  }
  return JSON.parse(line.slice(line.indexOf(marker) + marker.length));
}

function readDocCodes() {
  const md = fs.readFileSync(
    path.join(repoRoot, 'creator-docs/getting-started/ERROR_CODES.md'),
    'utf8',
  );
  const codes = [...md.matchAll(/<!-- code:([A-Z0-9_]+) -->/g)].map(m => m[1]);
  return [...new Set(codes)].sort();
}

function readFrontendCodes() {
  const ts = fs.readFileSync(
    path.join(repoRoot, 'distros/shared/src/api/generated/kernelErrorCodes.ts'),
    'utf8',
  );
  const m = ts.match(/export const KERNEL_STATIC_ERROR_CODES = (\[[\s\S]*?\]) as const/);
  if (!m) {
    throw new Error('KERNEL_STATIC_ERROR_CODES block not found in kernelErrorCodes.ts');
  }
  const codes = [...m[1].matchAll(/['"]([A-Z][A-Z0-9_]*)['"]/g)].map(match => match[1]);
  if (codes.length === 0) {
    throw new Error('KERNEL_STATIC_ERROR_CODES contains no string literals');
  }
  return codes.sort();
}

function diffSets(labelA, a, labelB, b) {
  const setA = new Set(a);
  const setB = new Set(b);
  const onlyA = a.filter(x => !setB.has(x));
  const onlyB = b.filter(x => !setA.has(x));
  if (onlyA.length || onlyB.length) {
    const parts = [];
    if (onlyA.length) parts.push(`only in ${labelA}: ${onlyA.join(', ')}`);
    if (onlyB.length) parts.push(`only in ${labelB}: ${onlyB.join(', ')}`);
    throw new Error(parts.join('; '));
  }
}

const kernel = readKernelCodesFromRust().sort();
const doc = readDocCodes();
const frontend = readFrontendCodes();

diffSets('kernel', kernel, 'ERROR_CODES.md', doc);
diffSets('kernel', kernel, 'kernelErrorCodes.ts', frontend);

console.log(`check-error-codes-drift: PASS (${kernel.length} static codes aligned)`);
