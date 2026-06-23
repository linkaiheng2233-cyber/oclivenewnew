#!/usr/bin/env node
/**
 * One-shot normative doc path migration: crates/ + src-tauri/ + root src/ → kernel/ + distros/.
 *
 * Usage:
 *   node scripts/migrate-doc-paths.mjs --dry-run
 *   node scripts/migrate-doc-paths.mjs
 *   node scripts/migrate-doc-paths.mjs --validate-only
 */
import { readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { join, relative, dirname, normalize } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

const dryRun = process.argv.includes('--dry-run');
const validateOnly = process.argv.includes('--validate-only');

/** Files/dirs skipped entirely (historical or intentional old-path maps). */
const SKIP_REL = new Set([
  'handoff/archive',
  'handoff/distros/STALE_PATHS_MIGRATION_CHECKLIST.md',
  'handoff/COMMENT_ENGLISH_MIGRATION_PLAN.md',
  'CHANGELOG.md',
  'CHANGELOG.en.md',
  'crates/README.md',
]);

/** Longest-first Rust / Tauri path replacements. */
const RUST_REPLACEMENTS = [
  ['src-tauri/src/domain/', 'kernel/crates/oclive_kernel_host/src/domain/'],
  ['src-tauri/src/infrastructure/', 'kernel/crates/oclive_kernel_host/src/infrastructure/'],
  ['src-tauri/src/http_api/', 'kernel/crates/oclive_kernel_host/src/http_api/'],
  ['src-tauri/src/service/', 'kernel/crates/oclive_kernel_host/src/service/'],
  ['src-tauri/src/state/', 'kernel/crates/oclive_kernel_host/src/state/'],
  ['src-tauri/src/utils/', 'kernel/crates/oclive_kernel_host/src/utils/'],
  ['src-tauri/src/models/', 'kernel/crates/oclive_kernel_types/src/models/'],
  ['src-tauri/src/kernel_lifecycle/', 'distros/desktop-tauri/src/kernel_lifecycle/'],
  ['src-tauri/src/api/', 'distros/desktop-tauri/src/api/'],
  ['src-tauri/src/kernel_attach.rs', 'distros/desktop-tauri/src/kernel_attach.rs'],
  ['src-tauri/src/lib.rs', 'distros/desktop-tauri/src/lib.rs'],
  ['src-tauri/src/main.rs', 'distros/desktop-tauri/src/main.rs'],
  ['src-tauri/tests/', 'distros/desktop-tauri/tests/'],
  ['src-tauri/', 'distros/desktop-tauri/'],
];

/** Frontend / workspace path replacements (specific before generic). */
const FRONTEND_REPLACEMENTS = [
  ['src/shells/theater/', 'distros/theater/src/shells/theater/'],
  ['src/composables/theater/', 'distros/theater/src/composables/theater/'],
  ['public/theater/', 'distros/theater/public/theater/'],
  ['src/shells/tool/', 'distros/chat-pro/src/shells/tool/'],
  ['src/shells/fluent/', 'distros/chat-pro/src/shells/fluent/'],
  ['src/smoke.test.ts', 'distros/chat-pro/src/smoke.test.ts'],
  ['src/stores/', 'distros/shared/src/stores/'],
  ['src/components/', 'distros/shared/src/components/'],
  ['src/api/', 'distros/shared/src/api/'],
  ['src/composables/', 'distros/shared/src/composables/'],
  ['src/build/', 'distros/shared/src/build/'],
  ['src/views/', 'distros/chat-pro/src/views/'],
  ['e2e/', 'distros/chat-pro/e2e/'],
];

const ROOT_FILE_REPLACEMENTS = [
  ['roles/', 'distros/chat-pro/roles/'],
  ['plugins/', 'distros/chat-pro/plugins/'],
  ['vite.config.ts', 'distros/chat-pro/vite.config.ts'],
  ['fuzz/', 'kernel/fuzz/'],
];

function shouldSkip(absPath) {
  const rel = relative(ROOT, absPath).replace(/\\/g, '/');
  if (SKIP_REL.has(rel)) return true;
  for (const skip of SKIP_REL) {
    if (skip.endsWith('/') && rel.startsWith(skip)) return true;
  }
  return false;
}

function walkMd(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (shouldSkip(p)) continue;
    const st = statSync(p);
    if (st.isDirectory()) {
      if (name === 'archive' || name === 'node_modules' || name === 'dist') continue;
      walkMd(p, out);
    } else if (name.endsWith('.md')) {
      out.push(p);
    }
  }
  return out;
}

function collectFiles() {
  const roots = [
    join(ROOT, 'creator-docs'),
    join(ROOT, 'creator-docs-en'),
    join(ROOT, 'human-docs'),
    join(ROOT, 'human-docs-en'),
    join(ROOT, 'handoff'),
  ];
  const files = [];
  for (const r of roots) {
    if (statSync(r).isDirectory()) walkMd(r, files);
  }
  for (const name of [
    'README.md',
    'README.en.md',
    'AGENTS.md',
    'CONTRIBUTING.md',
    'CONTRIBUTING.en.md',
    'HARDWARE_INTEGRATION.md',
    'roles/README_MANIFEST.md',
  ]) {
    const p = join(ROOT, name);
    if (!shouldSkip(p)) {
      try {
        statSync(p);
        files.push(p);
      } catch {
        /* missing */
      }
    }
  }
  return [...new Set(files)];
}

function alreadyPrefixed(text, idx, prefix) {
  const before = text.slice(Math.max(0, idx - prefix.length), idx);
  return before.endsWith(prefix);
}

function replaceAll(text, from, to) {
  if (!text.includes(from)) return text;
  let out = '';
  let i = 0;
  while (i < text.length) {
    const idx = text.indexOf(from, i);
    if (idx === -1) {
      out += text.slice(i);
      break;
    }
    if (from === 'crates/' && alreadyPrefixed(text, idx, 'kernel/')) {
      out += text.slice(i, idx + from.length);
      i = idx + from.length;
      continue;
    }
    if (from === 'roles/' && alreadyPrefixed(text, idx, 'chat-pro/')) {
      out += text.slice(i, idx + from.length);
      i = idx + from.length;
      continue;
    }
    if (from === 'plugins/' && (alreadyPrefixed(text, idx, 'chat-pro/') || alreadyPrefixed(text, idx, 'com.'))) {
      out += text.slice(i, idx + from.length);
      i = idx + from.length;
      continue;
    }
    out += text.slice(i, idx) + to;
    i = idx + from.length;
  }
  return out;
}

function migrateContent(text) {
  let out = text;
  for (const [from, to] of RUST_REPLACEMENTS) out = replaceAll(out, from, to);
  for (const [from, to] of FRONTEND_REPLACEMENTS) out = replaceAll(out, from, to);
  for (const [from, to] of ROOT_FILE_REPLACEMENTS) {
    if (from === 'plugins/') {
      out = out.replace(/(?<!directory_)plugins\//g, to);
      continue;
    }
    out = replaceAll(out, from, to);
  }
  out = replaceAll(out, 'crates/', 'kernel/crates/');
  const postFixes = [
    ['kernel/kernel/crates/', 'kernel/crates/'],
    ['distros/desktop-tauri/distros/shared/src/api/', 'distros/desktop-tauri/src/api/'],
    ['directory_distros/chat-pro/plugins/', 'directory_plugins/'],
    ['distros/chat-pro/distros/chat-pro/', 'distros/chat-pro/'],
    ['distros/shared/distros/shared/', 'distros/shared/'],
  ];
  for (const [from, to] of postFixes) out = replaceAll(out, from, to);
  return out;
}

/** Extract repo-relative paths from markdown links and backticks. */
function extractPaths(text) {
  const found = new Set();
  const patterns = [
    /`([^`\n]+)`/g,
    /\]\(([^)\s#]+)\)/g,
  ];
  for (const re of patterns) {
    re.lastIndex = 0;
    let m;
    while ((m = re.exec(text)) !== null) {
      let p = m[1].trim();
      if (!p || p.startsWith('http') || p.startsWith('mailto:')) continue;
      p = p.split('#')[0];
      if (!p || p.startsWith('#')) continue;
      found.add(p.replace(/\\/g, '/'));
    }
  }
  return [...found];
}

function resolveDocPath(docFile, refPath) {
  if (refPath.startsWith('/')) return null;
  const docDir = dirname(docFile);
  return normalize(join(docDir, refPath));
}

function validatePaths(files) {
  const missing = [];
  const needsReview = [];
  for (const fp of files) {
    const text = readFileSync(fp, 'utf8');
    for (const ref of extractPaths(text)) {
      if (ref.includes('oclive-pack-editor') || ref.includes('oclive-vscode')) continue;
      if (/^(LICENSE|CONTRIBUTING|AGENTS\.md)/.test(ref) && !ref.includes('/')) continue;
      const abs = resolveDocPath(fp, ref);
      if (!abs || !abs.startsWith(ROOT)) continue;
      const relFromRoot = relative(ROOT, abs).replace(/\\/g, '/');
      if (
        relFromRoot.startsWith('handoff/archive') ||
        relFromRoot === 'handoff/distros/STALE_PATHS_MIGRATION_CHECKLIST.md'
      ) {
        continue;
      }
      try {
        statSync(abs);
      } catch {
        if (/^(src\/|src-tauri\/|crates\/)/.test(ref) || ref.includes('/src/')) {
          needsReview.push({ file: relative(ROOT, fp), ref });
        } else {
          missing.push({ file: relative(ROOT, fp), ref, resolved: relFromRoot });
        }
      }
    }
  }
  return { missing, needsReview };
}

const files = collectFiles();
let changed = 0;

if (!validateOnly) {
  for (const fp of files) {
    const before = readFileSync(fp, 'utf8');
    const after = migrateContent(before);
    if (after !== before) {
      changed++;
      if (!dryRun) writeFileSync(fp, after, 'utf8');
    }
  }
  console.log(
    `migrate-doc-paths: ${dryRun ? 'DRY-RUN ' : ''}${changed} file(s) ${dryRun ? 'would change' : 'updated'}`
  );
}

const { missing, needsReview } = validatePaths(files);
if (needsReview.length) {
  console.warn(`NEEDS-REVIEW (${needsReview.length}):`);
  for (const item of needsReview.slice(0, 30)) {
    console.warn(`  ${item.file}: ${item.ref}`);
  }
  if (needsReview.length > 30) console.warn(`  … and ${needsReview.length - 30} more`);
}
if (missing.length) {
  console.error(`MISSING PATHS (${missing.length}):`);
  for (const item of missing.slice(0, 40)) {
    console.error(`  ${item.file}: ${item.ref} → ${item.resolved}`);
  }
  process.exitCode = 1;
} else {
  console.log('migrate-doc-paths: path existence check OK');
}
