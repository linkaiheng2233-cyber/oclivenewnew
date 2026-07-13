#!/usr/bin/env node
/**
 * English mirror doc gate (G14 companion):
 * 1. Every creator-docs-en markdown file (except hub README) starts with valid [中文](…) link.
 * 2. creator-docs markdown (excl. video-script/) has EN peer, RFC summary, or registry pending entry.
 * 3. human-docs L0–L8 numbered pages have EN peer (04 may use 04_ENGINEERING_RULES_SUMMARY).
 * 4. Optional drift hints when ZH mtime > EN + 30 days.
 * 5. High-traffic drift is a hard failure with --warn-drift-high-traffic (independent of --warn-drift).
 *
 * Usage:
 *   node scripts/check-doc-mirror.mjs [--warn-drift] [--warn-drift-high-traffic]
 *   node scripts/check-doc-mirror.mjs --self-test
 */
import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const WARN_DRIFT = process.argv.includes('--warn-drift');
const WARN_DRIFT_HIGH_TRAFFIC = process.argv.includes('--warn-drift-high-traffic');
const SELF_TEST = process.argv.includes('--self-test');
const DRIFT_DAYS = 30;

/**
 * High-traffic ZH paths under creator-docs: drift >30d vs EN peer is a hard failure
 * when --warn-drift-high-traffic is set. Only creator-docs paths (walker-reachable).
 */
const HIGH_TRAFFIC_DRIFT_ZH = new Set([
  'creator-docs/role-pack/ROLE_PACK_SPEC.md',
  'creator-docs/plugin-and-architecture/PLUGIN_V1.md',
  'creator-docs/security/KNOWN_VULNERABILITIES.md',
  'creator-docs/cli/OCLIVE_CLI_GUIDE.md',
  'creator-docs/COMPATIBILITY.md',
  'creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md',
  'creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md',
  'creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md',
]);

/** ZH paths (posix, from repo root) intentionally without 1:1 EN file yet. */
const CREATOR_PENDING = new Set([
  'creator-docs/video-script/PLUGIN_DEVELOPMENT_SCRIPT.md',
  'creator-docs/architecture/DESIGN_DECISIONS.md',
  'creator-docs/rfc/RFC_PORTRAIT_FACILITY.md',
  'creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md',
  'creator-docs/rfc/RFC_PROFILE_AND_DOMAIN_REEXPORT.md',
  'creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md',
  'creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md',
  'creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md',
  'creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md',
  'creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md',
]);

/** EN summary replaces full ZH RFC mirror. */
const RFC_SUMMARY_MAP = {
  'creator-docs/rfc/RFC_PORTRAIT_FACILITY.md':
    'creator-docs-en/rfc/RFC_PORTRAIT_FACILITY_SUMMARY.md',
  'creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md':
    'creator-docs-en/rfc/RFC_VISUAL_PRESENTATION_FACILITY_SUMMARY.md',
  'creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md':
    'creator-docs-en/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS_SUMMARY.md',
  'creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md':
    'creator-docs-en/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md',
  'creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md':
    'creator-docs-en/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN_SUMMARY.md',
  'creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md':
    'creator-docs-en/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE_SUMMARY.md',
  'creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md':
    'creator-docs-en/rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md',
};

const HUMAN_LADDER_EXCEPTIONS = {
  'human-docs/04_ENGINEERING_RULES.md': 'human-docs-en/04_ENGINEERING_RULES_SUMMARY.md',
};

const EN_HUB_SKIP = new Set([
  'creator-docs-en/README.md',
  'human-docs-en/README.md',
]);

function walkMd(dirAbs, relPrefix, out = []) {
  if (!fs.existsSync(dirAbs)) return out;
  for (const name of fs.readdirSync(dirAbs)) {
    const abs = path.join(dirAbs, name);
    const rel = relPrefix ? `${relPrefix}/${name}` : name;
    if (fs.statSync(abs).isDirectory()) {
      if (name === 'archive') continue;
      walkMd(abs, rel, out);
    } else if (name.endsWith('.md')) {
      out.push(rel.replace(/\\/g, '/'));
    }
  }
  return out;
}

function exists(relPosix, root = ROOT) {
  return fs.existsSync(path.join(root, relPosix));
}

function enPeerForZh(zhRel) {
  if (!zhRel.startsWith('creator-docs/')) return null;
  const tail = zhRel.slice('creator-docs/'.length);
  if (RFC_SUMMARY_MAP[zhRel]) return RFC_SUMMARY_MAP[zhRel];
  return `creator-docs-en/${tail}`;
}

function checkZhLink(fileRel, root = ROOT) {
  const abs = path.join(root, fileRel);
  const content = fs.readFileSync(abs, 'utf8');
  const head = content.split('\n').slice(0, 50).join('\n');
  const m = head.match(/\[中文\]\(([^)]+)\)/) ?? content.match(/\[中文\]\(([^)]+)\)/);
  if (!m) {
    return { ok: false, reason: 'missing [中文](…) link' };
  }
  const target = m[1].split('#')[0];
  let resolved;
  if (target.startsWith('http://') || target.startsWith('https://')) {
    return { ok: true };
  }
  if (target.startsWith('/')) {
    resolved = target.slice(1);
  } else {
    resolved = path
      .normalize(path.join(path.dirname(fileRel), target))
      .replace(/\\/g, '/');
  }
  if (!exists(resolved, root)) {
    return { ok: false, reason: `[中文] target missing: ${resolved}` };
  }
  return { ok: true };
}

/**
 * Evaluate mtime drift for a ZH/EN pair.
 * High-traffic paths are hard errors when warnHighTraffic (even without warnAll).
 * Other paths warn only when warnAll.
 */
function evaluateDrift(zh, peer, root, highTrafficSet, warnAll, warnHighTraffic) {
  const zhM = fs.statSync(path.join(root, zh)).mtimeMs;
  const enM = fs.statSync(path.join(root, peer)).mtimeMs;
  const days = (zhM - enM) / (86400 * 1000);
  if (days <= DRIFT_DAYS) return { error: null, warning: null };
  const msg = `drift hint: ${zh} newer than EN by ${Math.round(days)}d`;
  if (warnHighTraffic && highTrafficSet.has(zh)) {
    return { error: msg, warning: null };
  }
  if (warnAll) {
    return { error: null, warning: msg };
  }
  return { error: null, warning: null };
}

function checkCoverage(
  root = ROOT,
  highTrafficSet = HIGH_TRAFFIC_DRIFT_ZH,
  warnAll = WARN_DRIFT,
  warnHighTraffic = WARN_DRIFT_HIGH_TRAFFIC,
) {
  const errors = [];
  const warnings = [];

  const zhFiles = walkMd(path.join(root, 'creator-docs'), 'creator-docs').filter(
    (f) =>
      !f.includes('/video-script/') &&
      f !== 'creator-docs/README.md' &&
      !f.includes('/architecture-en/') &&
      !f.endsWith('.en.md'),
  );

  for (const zh of zhFiles) {
    if (CREATOR_PENDING.has(zh)) continue;
    const peer = enPeerForZh(zh);
    if (peer && exists(peer, root)) {
      if (warnAll || warnHighTraffic) {
        const d = evaluateDrift(zh, peer, root, highTrafficSet, warnAll, warnHighTraffic);
        if (d.error) errors.push(d.error);
        if (d.warning) warnings.push(d.warning);
      }
      continue;
    }
    errors.push(`no EN mirror: ${zh} (expected ${peer ?? '?'})`);
  }

  return { errors, warnings };
}

function checkEnLinks(root = ROOT) {
  const errors = [];
  const enFiles = walkMd(path.join(root, 'creator-docs-en'), 'creator-docs-en');
  for (const f of enFiles) {
    if (EN_HUB_SKIP.has(f)) continue;
    const r = checkZhLink(f, root);
    if (!r.ok) errors.push(`${f}: ${r.reason}`);
  }
  return errors;
}

function checkHumanLadder(root = ROOT) {
  const errors = [];
  const humanRoot = path.join(root, 'human-docs');
  if (!fs.existsSync(humanRoot)) return errors;
  const numbered = fs
    .readdirSync(humanRoot)
    .filter((n) => /^(0[0-9]|10)_.*\.md$/.test(n))
    .sort();

  for (const name of numbered) {
    const zh = `human-docs/${name}`;
    const en = HUMAN_LADDER_EXCEPTIONS[zh] ?? `human-docs-en/${name}`;
    if (!exists(en, root)) {
      errors.push(`human ladder missing EN: ${zh} → expected ${en}`);
    }
  }
  return errors;
}

function runSelfTest() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-doc-mirror-'));
  try {
    const zhRel = 'creator-docs/security/KNOWN_VULNERABILITIES.md';
    const enRel = 'creator-docs-en/security/KNOWN_VULNERABILITIES.md';
    fs.mkdirSync(path.join(tmp, 'creator-docs/security'), { recursive: true });
    fs.mkdirSync(path.join(tmp, 'creator-docs-en/security'), { recursive: true });
    fs.writeFileSync(path.join(tmp, zhRel), '# ZH fixture\n');
    fs.writeFileSync(
      path.join(tmp, enRel),
      '# EN fixture\n\n[中文](../../creator-docs/security/KNOWN_VULNERABILITIES.md)\n',
    );

    const now = Date.now() / 1000;
    const enStale = now - (DRIFT_DAYS + 5) * 86400;
    fs.utimesSync(path.join(tmp, zhRel), now, now);
    fs.utimesSync(path.join(tmp, enRel), enStale, enStale);

    const highSet = new Set([zhRel]);

    // dimension5 argv shape: ONLY --warn-drift-high-traffic (no --warn-drift)
    const htOnly = checkCoverage(tmp, highSet, /*warnAll*/ false, /*warnHighTraffic*/ true);
    if (htOnly.errors.length === 0) {
      console.error(
        'check-doc-mirror --self-test: expected hard failure with --warn-drift-high-traffic alone',
      );
      process.exit(1);
    }
    if (!htOnly.errors.some((e) => e.includes(zhRel))) {
      console.error(`check-doc-mirror --self-test: unexpected errors: ${htOnly.errors.join('; ')}`);
      process.exit(1);
    }

    // Neither flag → silent (the old CI bug mode)
    const silent = checkCoverage(tmp, highSet, false, false);
    if (silent.errors.length !== 0 || silent.warnings.length !== 0) {
      console.error('check-doc-mirror --self-test: no-flag mode should skip drift');
      process.exit(1);
    }

    // Fresh EN peer must pass high-traffic gate
    fs.utimesSync(path.join(tmp, enRel), now, now);
    const fresh = checkCoverage(tmp, highSet, false, true);
    if (fresh.errors.some((e) => e.includes('drift hint'))) {
      console.error('check-doc-mirror --self-test: fresh EN should not drift-fail');
      process.exit(1);
    }

    console.log('check-doc-mirror --self-test: OK');
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

function main() {
  if (SELF_TEST) {
    runSelfTest();
    return;
  }

  const allErrors = [
    ...checkEnLinks().map((e) => `[pair-link] ${e}`),
    ...checkCoverage().errors.map((e) => `[coverage] ${e}`),
    ...checkHumanLadder().map((e) => `[human-ladder] ${e}`),
  ];
  const warnings = checkCoverage().warnings;

  for (const w of warnings) {
    console.warn(`::warning::${w}`);
  }

  if (allErrors.length === 0) {
    console.log('check-doc-mirror: OK');
    return;
  }

  for (const e of allErrors) {
    console.error(`::error title=check-doc-mirror::${e}`);
  }
  console.error(`check-doc-mirror: ${allErrors.length} violation(s)`);
  process.exit(1);
}

main();
