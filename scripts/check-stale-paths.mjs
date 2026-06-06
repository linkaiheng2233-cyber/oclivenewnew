#!/usr/bin/env node
/**
 * Warn when normative docs still reference legacy orchestration paths or banned aliases.
 * CI: continue-on-error until doc debt is fully cleared.
 */
import { readFileSync, readdirSync, statSync } from 'fs';
import { join } from 'path';

const ROOT = new URL('..', import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1');

const SKIP_FILES = new Set([
  join(ROOT, 'creator-docs/NAMING_CONVENTIONS.md'),
]);

const GLOBS = [
  'README.md',
  'README.en.md',
  'AGENTS.md',
  'CONTRIBUTING.md',
  'roles/README_MANIFEST.md',
];

function walkMd(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (name === 'archive') continue;
    const st = statSync(p);
    if (st.isDirectory()) walkMd(p, out);
    else if (name.endsWith('.md')) out.push(p);
  }
  return out;
}

const files = [
  ...GLOBS.map((f) => join(ROOT, f)),
  ...walkMd(join(ROOT, 'creator-docs')),
];

const patterns = [
  { re: /src-tauri\/src\/domain/g, label: 'legacy orchestration path src-tauri/src/domain' },
  { re: /\bmemory_backend\b/g, label: 'banned alias memory_backend (use plugin_backends / slot_registry)' },
];

let hits = 0;
for (const fp of files) {
  if (SKIP_FILES.has(fp)) continue;
  let text;
  try {
    text = readFileSync(fp, 'utf8');
  } catch {
    continue;
  }
  for (const { re, label } of patterns) {
    re.lastIndex = 0;
    if (re.test(text)) {
      console.warn(`::warning file=${fp.replace(/\\/g, '/')},title=stale-path-check::${label}`);
      hits++;
    }
  }
}

if (hits === 0) {
  console.log('check-stale-paths: OK (no legacy paths or banned aliases in normative docs)');
} else {
  console.warn(`check-stale-paths: ${hits} warning(s) — see NAMING_CONVENTIONS.md`);
  process.exitCode = 1;
}
