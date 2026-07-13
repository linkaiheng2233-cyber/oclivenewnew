#!/usr/bin/env node
/**
 * K-SUPPLY-05 ratchet: `cargo tree -d` duplicate group count must not increase.
 * Baseline in handoff/LAYERING_BASELINE.json → cargo_duplicate_groups.
 */
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BASELINE_PATH = path.join(ROOT, 'handoff', 'LAYERING_BASELINE.json');

function countDuplicateGroups() {
  const out = execSync('cargo tree -d', {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  return out.split(/\r?\n/).filter((line) => /^[A-Za-z0-9_-]/.test(line)).length;
}

function main() {
  const baseline = JSON.parse(fs.readFileSync(BASELINE_PATH, 'utf8'));
  const maxGroups = baseline.cargo_duplicate_groups;
  if (typeof maxGroups !== 'number') {
    console.error('::error::LAYERING_BASELINE.json missing cargo_duplicate_groups');
    process.exit(1);
  }
  const count = countDuplicateGroups();
  console.log(`cargo tree -d duplicate roots: ${count} (baseline <= ${maxGroups})`);
  if (count > maxGroups) {
    console.error(
      `::error title=cargo-dedup-ratchet::duplicate groups ${count} exceeds baseline ${maxGroups}`,
    );
    process.exit(1);
  }
  console.log('check-cargo-dedup-ratchet: OK');
}

main();
