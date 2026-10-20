#!/usr/bin/env node
/**
 * Copy the vendored documentation navigator from its upstream checkout and
 * refresh scripts/vendor/stackmap/VENDOR.json.
 *
 * Usage: node scripts/sync-vendor.mjs [--from PATH]
 * Default upstream checkout: ../stackmap-check (sibling directory).
 */
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = fileURLToPath(new URL('../', import.meta.url));
const vendorDir = path.join(repoRoot, 'scripts/vendor/stackmap');
const files = ['cli.mjs', 'cli.test.mjs', 'eval.mjs'];

const index = process.argv.indexOf('--from');
const from = index >= 0 ? path.resolve(process.argv[index + 1]) : path.resolve(repoRoot, '../stackmap-check');

for (const name of files) {
  const source = path.join(from, name);
  if (!fs.existsSync(source)) {
    console.error(`sync-vendor: missing ${source}`);
    process.exitCode = 1;
  } else {
    fs.copyFileSync(source, path.join(vendorDir, name));
  }
}

let commit = 'unknown';
try {
  commit = execFileSync('git', ['-C', from, 'rev-parse', 'HEAD'], { encoding: 'utf8', windowsHide: true }).trim();
} catch {
  // A source checkout without git history still syncs; the commit stays unknown.
}

const hashes = Object.fromEntries(files.map(name => [name,
  createHash('sha256').update(fs.readFileSync(path.join(vendorDir, name), 'utf8').replace(/\r\n/g, '\n')).digest('hex')]));
const manifest = {
  source: 'https://github.com/linkaiheng2233-cyber/stackmap-check',
  commit,
  files: hashes,
};
fs.writeFileSync(path.join(vendorDir, 'VENDOR.json'), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
console.log(`sync-vendor: ${files.length} files from ${from} @ ${commit.slice(0, 8)}`);
