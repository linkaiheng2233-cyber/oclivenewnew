#!/usr/bin/env node
/**
 * Print live project scale metrics.
 * Usage: node scripts/project-scale.mjs
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function countWorkspaceCrates() {
  const cargo = fs.readFileSync(path.join(repoRoot, 'Cargo.toml'), 'utf8');
  const members = [];
  let inMembers = false;
  for (const line of cargo.split('\n')) {
    if (/^members\s*=/.test(line)) {
      inMembers = true;
      continue;
    }
    if (inMembers) {
      const m = line.match(/^\s*"([^"]+)"/);
      if (m) members.push(m[1]);
      if (line.includes(']')) break;
    }
  }
  return members.length;
}

function countByExt(roots, ext) {
  let n = 0;
  const stack = [...roots];
  while (stack.length) {
    const dir = stack.pop();
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === 'node_modules' || entry.name === 'target' || entry.name === 'dist') continue;
      const abs = path.join(dir, entry.name);
      if (entry.isDirectory()) stack.push(abs);
      else if (entry.name.endsWith(ext)) n += 1;
    }
  }
  return n;
}

function countMigrations() {
  const migDir = path.join(repoRoot, 'kernel/crates/oclive_kernel_host/migrations');
  return fs.readdirSync(migDir).filter((f) => f.endsWith('.sql')).length;
}

function main() {
  const crates = countWorkspaceCrates();
  const rsFiles = countByExt(
    [path.join(repoRoot, 'kernel'), path.join(repoRoot, 'distros/desktop-tauri')],
    '.rs',
  );
  const frontendFiles = countByExt(
    [path.join(repoRoot, 'distros/shared'), path.join(repoRoot, 'distros/chat-pro'), path.join(repoRoot, 'distros/theater')],
    '.vue',
  ) + countByExt(
    [path.join(repoRoot, 'distros/shared'), path.join(repoRoot, 'distros/chat-pro'), path.join(repoRoot, 'distros/theater')],
    '.ts',
  );
  const migrations = countMigrations();

  console.log('OCLive project scale (live):');
  console.log(`  workspace crates: ${crates}`);
  console.log(`  Rust .rs files (kernel + desktop-tauri): ${rsFiles}`);
  console.log(`  frontend .vue + .ts (shared + chat-pro + theater): ${frontendFiles}`);
  console.log(`  SQL migrations: ${migrations}`);
}

main();
