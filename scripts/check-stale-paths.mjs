#!/usr/bin/env node
/**
 * Hard gate: normative docs and monorepo code must not reference legacy layout paths.
 * Wired into dimension5-acceptance.mjs (docs + code checks).
 * Whitelist rules SSOT: handoff/AI_CHANGE_BOUNDARIES.md G5
 *
 * Usage:
 *   node scripts/check-stale-paths.mjs            # docs + code
 *   node scripts/check-stale-paths.mjs --docs-only
 *   node scripts/check-stale-paths.mjs --code-only
 */
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, relative } from 'path';
import { fileURLToPath } from 'url';

const ROOT = join(fileURLToPath(new URL('..', import.meta.url)));
const argv = new Set(process.argv.slice(2));
const docsOnly = argv.has('--docs-only');
const codeOnly = argv.has('--code-only');
const runDocs = !codeOnly;
const runCode = !docsOnly;

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
  'distros/chat-pro/roles/README_MANIFEST.md',
];

/** Relative paths (posix) exempt from code-path ratchet. */
const CODE_SKIP_REL = new Set([
  'scripts/check-stale-paths.mjs',
  'scripts/migrate-doc-paths.mjs',
  'scripts/split-db-rs.mjs',
  'scripts/fix-empty-after-errors-doc.mjs',
  'scripts/add-missing-errors-doc-tauri.mjs',
  'scripts/theater-env.mjs',
  'scripts/tauri-shell-dist.mjs',
  'kernel/crates/oclive_kernel_runtime/src/kernel_discovery.rs',
  'kernel/crates/oclive_kernel_host/src/state/roles_dir.rs',
  'kernel/crates/oclive-cli/src/role_pack.rs',
  'kernel/crates/oclive-cli/src/market_cmd.rs',
  'kernel/crates/oclive-cli/src/pack_cmd.rs',
  'kernel/crates/oclive-cli/src/generator.rs',
  'kernel/crates/oclive-cli/src/ci_cmd.rs',
  'kernel/crates/oclive-cli/src/templates/CONFIG_REFERENCE.md',
  'distros/desktop-tauri/tests/reply_post_processor_directory_roundtrip.rs',
  'kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/backends/hybrid_store.rs',
  'kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/store_trait_tests.rs',
  'kernel/crates/oclive-cli/tests/e2e_init_legacy.rs',
  'kernel/crates/oclive-cli/tests/e2e_init_templates.rs',
  'kernel/crates/oclive-cli/tests/e2e_init_minimal.rs',
  'kernel/crates/oclive_kernel_runtime/src/lib.rs',
  'kernel/crates/oclive_kernel_types/src/lib.rs',
  'kernel/crates/oclive_kernel_host/src/infrastructure/sql_migrate.rs',
  'kernel/crates/oclive-cli/src/doctor_kernel_contracts.rs',
  'distros/desktop-tauri/src/lib.rs',
]);

const CODE_SKIP_PREFIXES = [
  'handoff/archive/',
  'scripts/lib/',
];

const CODE_DIRS = [
  'kernel/crates',
  'distros',
  'scripts',
  'examples',
  '.github',
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

function walkCodeFiles(dir, out = [], depth = 0) {
  if (depth > 12) return out;
  let entries;
  try {
    entries = readdirSync(dir);
  } catch {
    return out;
  }
  for (const name of entries) {
    if (name === 'node_modules' || name === 'target' || name === 'dist') continue;
    const p = join(dir, name);
    let st;
    try {
      st = statSync(p);
    } catch {
      continue;
    }
    if (st.isDirectory()) {
      walkCodeFiles(p, out, depth + 1);
      continue;
    }
    if (
      name.endsWith('.rs')
      || name.endsWith('.mjs')
      || name.endsWith('.sh')
      || name.endsWith('.yml')
      || name.endsWith('.yaml')
    ) {
      out.push(p);
    }
  }
  return out;
}

function collectDocFiles() {
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

function collectCodeFiles() {
  const files = [];
  for (const rel of CODE_DIRS) {
    const d = join(ROOT, rel);
    try {
      if (statSync(d).isDirectory()) walkCodeFiles(d, files);
    } catch {
      /* missing */
    }
  }
  return files;
}

function shouldSkipCode(rel) {
  if (CODE_SKIP_REL.has(rel)) return true;
  return CODE_SKIP_PREFIXES.some((p) => rel.startsWith(p));
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

function lineDocumentsForbiddenPath(line) {
  if (/bare `roles\/`|应覆盖|禁止.*roles|→.*distros|过期|legacy|迁移|不要|不应|勿|错布局|D-ORDER|check-stale-paths/.test(line)) {
    return true;
  }
  if (/`src-tauri`|src-tauri\/Cargo/.test(line) && /应|禁止|覆盖|过期|→|错|bare/.test(line)) {
    return true;
  }
  if (/\bcrates\//.test(line) && /应|禁止|kernel\/crates|bare|覆盖/.test(line)) {
    return true;
  }
  return false;
}

function hasBareMonorepoRolesDoc(line, rel = '') {
  if (rel.startsWith('distros/chat-pro/roles/')) return false;
  if (lineDocumentsForbiddenPath(line)) return false;
  if (/distros\/chat-pro\/roles/.test(line)) return false;
  if (/\broles\/README_MANIFEST\.md\b/.test(line) && !/distros\/chat-pro\/roles/.test(line)) {
    return true;
  }
  return false;
}

/** Active docs may link archived closure checklists only through their archive path. */
function lineHasStaleHandoffClosure(line, rel = '') {
  if (rel.startsWith('handoff/archive/')) return [];
  const checks = [
    ['handoff/A3_CLOSURE_SUMMARY', 'handoff/archive/A3_CLOSURE_SUMMARY'],
    ['handoff/PRODUCT_RELEASE_CHECKLIST', 'handoff/archive/PRODUCT_RELEASE_CHECKLIST'],
    ['handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST', 'handoff/archive/PRODUCT_AND_KERNEL_GAP_CHECKLIST'],
  ];
  const hits = [];
  for (const [bad, good] of checks) {
    const stripped = line.split(good).join('__ARCHIVE_OK__');
    if (stripped.includes(bad)) {
      hits.push(`stale handoff root path ${bad} (use handoff/archive/...)`);
    }
  }
  return hits;
}

/** G3/G12: archived product checklists may be cited as history, never as current truth. */
function lineHasArchivedProductTruth(line, rel = '') {
  if (rel.startsWith('handoff/archive/')) return [];
  if (!/archive\/PRODUCT_(?:RELEASE_CHECKLIST|AND_KERNEL_GAP_CHECKLIST)\.md/.test(line)) {
    return [];
  }
  if (/历史|归档|追溯|不作.*truth|historical|not current truth|history only/i.test(line)) {
    return [];
  }
  return ['archived product checklist used as current truth (G3/G12)'];
}

function scanDocLine(line, rel = '') {
  if (lineDocumentsForbiddenPath(line)) return [];
  const inRolePackTree = rel.startsWith('distros/chat-pro/roles/');
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
  if (hasBareCrates(line) && !inRolePackTree) {
    hits.push('legacy root layout path crates/ (use kernel/crates/)');
  }
  if (/\bsrc-tauri\//.test(line) && !inRolePackTree) {
    hits.push('legacy root layout path src-tauri/ (use distros/desktop-tauri/)');
  }
  if (hasLegacyFrontendSrc(line)) {
    hits.push('legacy root layout path src/ (use distros/shared|chat-pro|theater)');
  }
  if (hasBareMonorepoRolesDoc(line, rel)) {
    hits.push('bare roles/ path (use distros/chat-pro/roles/)');
  }
  hits.push(...lineHasStaleHandoffClosure(line, rel));
  hits.push(...lineHasArchivedProductTruth(line, rel));
  return hits;
}

function isCommentLine(line, ext) {
  const t = line.trim();
  if (ext === '.rs' && (t.startsWith('//') || t.startsWith('/*') || t.startsWith('*'))) return true;
  if ((ext === '.mjs' || ext === '.sh') && t.startsWith('//')) return true;
  if ((ext === '.mjs' || ext === '.sh') && t.startsWith('#')) return true;
  return false;
}

function scanCodeLine(line, rel, ext) {
  if (isCommentLine(line, ext)) return [];
  const hits = [];
  if (/\bsrc-tauri\//.test(line) && !/distros\/desktop-tauri/.test(line)) {
    if (!/legacy scaffold|generated scaffold|legacy_ws/i.test(line)) {
      hits.push('legacy src-tauri/ path (use distros/desktop-tauri/)');
    }
  }
  if (/\bcd fuzz\b/.test(line) && !/kernel\/fuzz/.test(line)) {
    hits.push('cd fuzz without kernel/fuzz prefix');
  }
  if (/testDir:\s*['"]e2e['"]/.test(line)) {
    hits.push('playwright testDir should be distros/chat-pro/e2e');
  }
  if (/working-directory:\s*src-tauri\b/.test(line)) {
    hits.push('CI working-directory src-tauri (use distros/desktop-tauri or repo root)');
  }
  if (/directory:\s*["']\/src-tauri["']/.test(line)) {
    hits.push('dependabot directory /src-tauri (use / or distros/desktop-tauri)');
  }
  if (/\.\.\/\.\.\/roles\b/.test(line) && !/chat-pro\/roles/.test(line)) {
    hits.push('legacy ../../roles (use ../../distros/chat-pro/roles)');
  }
  if (/join\(['"]\.\.\/roles['"]\)/.test(line)) {
    hits.push('legacy join("../roles")');
  }
  if (/\.join\(['"]roles['"]\)/.test(line)) {
    if (
      !/resolve_project_roles_dir|chat_pro_roles_dir|chat-pro/.test(line)
      && !/app_data|out\.join|dir\.path\(\)|args\.output/.test(line)
    ) {
      hits.push('monorepo .join("roles") without distros/chat-pro/roles');
    }
  }
  if (/\.join\(["']plugins\//.test(line) && !/distros\/chat-pro\/plugins/.test(line)) {
    hits.push('bare plugins/ path (use distros/chat-pro/plugins/)');
  }
  return hits;
}

let violations = 0;

function report(rel, lineNo, label) {
  console.error(`::error file=${rel},line=${lineNo},title=stale-path-check::${label}`);
  violations++;
}

if (runDocs) {
  for (const fp of collectDocFiles()) {
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
      for (const label of scanDocLine(lines[i], rel)) {
        report(rel, i + 1, label);
      }
    }
  }
}

if (runCode) {
  for (const fp of collectCodeFiles()) {
    const rel = relative(ROOT, fp).replace(/\\/g, '/');
    if (shouldSkipCode(rel)) continue;
    let text;
    try {
      text = readFileSync(fp, 'utf8');
    } catch {
      continue;
    }
    const ext = rel.slice(rel.lastIndexOf('.'));
    const lines = text.split('\n');
    for (let i = 0; i < lines.length; i++) {
      for (const label of scanCodeLine(lines[i], rel, ext)) {
        report(rel, i + 1, label);
      }
    }
  }
}

if (violations === 0) {
  const scope = runDocs && runCode ? 'docs + code' : runDocs ? 'docs' : 'code';
  console.log(`check-stale-paths: OK (${scope}; no legacy paths, banned aliases, or archive truth)`);
} else {
  console.error(`check-stale-paths: ${violations} violation(s) — see NAMING_CONVENTIONS.md`);
  process.exit(1);
}
