#!/usr/bin/env node
/**
 * Hard gate: normative docs must not reference legacy layout paths or banned aliases.
 * Wired into dimension5-acceptance.mjs (check 11).
 */
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, relative } from 'path';
import { fileURLToPath } from 'url';

const ROOT = join(fileURLToPath(new URL('..', import.meta.url)));

const SKIP_FILES = new Set([
  join(ROOT, 'handoff/distros/STALE_PATHS_MIGRATION_CHECKLIST.md'),
  join(ROOT, 'handoff/COMMENT_ENGLISH_MIGRATION_PLAN.md'),
  join(ROOT, 'CHANGELOG.md'),
  join(ROOT, 'CHANGELOG.en.md'),
  join(ROOT, 'creator-docs/NAMING_CONVENTIONS.md'),
  join(ROOT, 'crates/README.md'),
]);

const ROOT_MD = [
  'README.md',
  'README.en.md',
  'AGENTS.md',
  'CONTRIBUTING.md',
  'CONTRIBUTING.en.md',
  'HARDWARE_INTEGRATION.md',
  'roles/README_MANIFEST.md',
];

function walkMd(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (name === 'archive' || name === 'node_modules' || name === 'dist') continue;
    const st = statSync(p);
    if (st.isDirectory()) walkMd(p, out);
    else if (name.endsWith('.md')) out.push(p);
  }
  return out;
}

function collectFiles() {
  const dirs = [
    join(ROOT, 'creator-docs'),
    join(ROOT, 'creator-docs-en'),
    join(ROOT, 'human-docs'),
    join(ROOT, 'human-docs-en'),
    join(ROOT, 'handoff'),
  ];
  const files = [];
  for (const d of dirs) {
    if (statSync(d).isDirectory()) walkMd(d, files);
  }
  for (const f of ROOT_MD) {
    const p = join(ROOT, f);
    try {
      statSync(p);
      files.push(p);
    } catch {
      /* missing */
    }
  }
  return files;
}

function hasBareCrates(line) {
  let s = line;
  while (true) {
    const idx = s.indexOf('crates/');
    if (idx === -1) return false;
    const prefix = s.slice(Math.max(0, idx - 7), idx);
    if (!prefix.endsWith('kernel/')) return true;
    s = s.slice(idx + 'crates/'.length);
  }
}

function lineHasBannedMemoryBackend(line) {
  if (!/\bmemory_backend\b/.test(line)) return false;
  if (/不再/.test(line) && /使用/.test(line)) return false;
  if (/禁止/.test(line) && /\bmemory_backend\b/.test(line)) return false;
  if (/不是/.test(line) && /\bmemory_backend\b/.test(line)) return false;
  if (/\|\s*`memory_backend`\s*\|/.test(line)) return false;
  if (/banned alias|non-`memory_backend`|not `memory_backend`|use plugin_backends/i.test(line)) {
    return false;
  }
  return true;
}

const LEGACY_FRONTEND_SRC_PREFIXES = [
  'src/stores/',
  'src/views/',
  'src/components/',
  'src/composables/',
  'src/api/',
  'src/shells/',
  'src/build/',
  'src/theater/',
  'src/i18n/',
  'src/utils/',
  'src/adapters/',
  'src/styles/',
  'src/lib/',
  'src/main.js',
  'src/DirectoryShellApp.vue',
];

function isUnderDistrosOrKernel(line, idx) {
  const before = line.slice(0, idx);
  if (/distros\/(?:shared|chat-pro|theater|desktop-tauri)\/$/.test(before)) return true;
  if (/oclive-(?:pack-editor|vscode|launcher)\/$/.test(before)) return true;
  return false;
}

function hasLegacyFrontendSrc(line) {
  for (const p of LEGACY_FRONTEND_SRC_PREFIXES) {
    let idx = 0;
    while ((idx = line.indexOf(p, idx)) !== -1) {
      if (isUnderDistrosOrKernel(line, idx)) {
        idx += p.length;
        continue;
      }
      return true;
    }
  }
  if (/`src\/`\s*[（(]/.test(line)) return true;
  if (/\(`src\/`/.test(line)) return true;
  if (/\[`src\/`/.test(line)) return true;
  if (/根 `src\//.test(line)) return true;
  if (/\.\.\/\.\.\/src\//.test(line)) return true;
  if (/(?:^|\|)\s*`?src\/`?\s*\|/.test(line) && /Vue|前端|frontend/i.test(line)) return true;
  return false;
}

function scanLine(line) {
  const hits = [];
  if (/\bsrc-tauri\/src\/domain\b/.test(line)) {
    hits.push('legacy orchestration path src-tauri/src/domain');
  }
  if (lineHasBannedMemoryBackend(line)) {
    hits.push('banned alias memory_backend (use plugin_backends / slot_registry)');
  }
  if (/domain\/prompt_builder\.rs/.test(line) && !/prompt_builder\/mod\.rs/.test(line)) {
    hits.push('stale prompt_builder.rs path (use prompt_builder/mod.rs + sections.rs)');
  }
  if (hasBareCrates(line)) hits.push('legacy root layout path crates/ (use kernel/crates/)');
  if (/\bsrc-tauri\//.test(line)) {
    hits.push('legacy root layout path src-tauri/ (use distros/desktop-tauri/)');
  }
  if (hasLegacyFrontendSrc(line)) {
    hits.push('legacy root layout path src/ (use distros/shared|chat-pro|theater)');
  }
  return hits;
}

const files = collectFiles();
let violations = 0;

for (const fp of files) {
  if (SKIP_FILES.has(fp)) continue;
  let text;
  try {
    text = readFileSync(fp, 'utf8');
  } catch {
    continue;
  }
  const rel = relative(ROOT, fp).replace(/\\/g, '/');
  const lines = text.split('\n');
  for (let i = 0; i < lines.length; i++) {
    for (const label of scanLine(lines[i])) {
      console.error(`::error file=${rel},line=${i + 1},title=stale-path-check::${label}`);
      violations++;
    }
  }
}

if (violations === 0) {
  console.log('check-stale-paths: OK (no legacy paths or banned aliases in normative docs)');
} else {
  console.error(`check-stale-paths: ${violations} violation(s) — see NAMING_CONVENTIONS.md`);
  process.exit(1);
}
