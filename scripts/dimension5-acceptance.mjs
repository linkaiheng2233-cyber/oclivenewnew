#!/usr/bin/env node
/**
 * Dimension 5 engineering-discipline acceptance gate.
 * Chains existing ratchets; exits 0 only when all checks pass.
 *
 * Usage:
 *   node scripts/dimension5-acceptance.mjs        # full (includes sample cargo tests)
 *   node scripts/dimension5-acceptance.mjs --ci     # CI: skip slow workspace sample tests
 */
import { execFileSync, spawnSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const ciMode = process.argv.includes('--ci');

const results = [];

function runStep(name, fn) {
  process.stdout.write(`[dimension5] ${name}… `);
  try {
    fn();
    console.log('PASS');
    results.push({ name, ok: true });
  } catch (err) {
    console.log('FAIL');
    console.error(err instanceof Error ? err.message : String(err));
    results.push({ name, ok: false, error: String(err) });
  }
}

function sh(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    ...opts,
  });
  if (r.status !== 0) {
    const detail = [r.stdout, r.stderr].filter(Boolean).join('\n').trim();
    throw new Error(detail || `${cmd} ${args.join(' ')} exited ${r.status}`);
  }
  return r.stdout ?? '';
}

function rgCount(pattern, file) {
  try {
    const out = execFileSync('rg', ['-c', pattern, file], {
      encoding: 'utf8',
      cwd: repoRoot,
    }).trim();
    if (!out) return 0;
    let total = 0;
    for (const line of out.split('\n')) {
      const m = line.match(/:(\d+)$/);
      if (m) total += Number(m[1]);
    }
    return total;
  } catch {
    return 0;
  }
}

runStep('layering ratchet', () => {
  sh('node', ['scripts/check-domain-layering.mjs']);
});

runStep('cargo audit (no-fetch stale)', () => {
  sh('cargo', ['audit', '--no-fetch', '--stale']);
});

runStep('cargo deny (licenses + bans)', () => {
  sh('cargo', ['deny', 'check', 'licenses', 'bans']);
});

runStep('Cargo.lock excludes sqlx-mysql / rsa', () => {
  const lockPath = path.join(repoRoot, 'Cargo.lock');
  const lock = fs.readFileSync(lockPath, 'utf8');
  if (/name = "sqlx-mysql"/.test(lock)) {
    throw new Error('Cargo.lock contains sqlx-mysql');
  }
  if (/name = "rsa"/.test(lock)) {
    throw new Error('Cargo.lock contains rsa');
  }
});

runStep('kernel ensure plan snapshot', () => {
  sh('cargo', ['test', '-p', 'oclive-cli', '--test', 'kernel_ensure_plan_snapshot']);
});

runStep('RFC affect drift ratchet', () => {
  sh('node', ['scripts/check-rfc-affect-drift.mjs']);
});

runStep('CHANGELOG [Unreleased] parity', () => {
  sh('node', ['scripts/check-changelog-parity.mjs']);
});

runStep('doc registry ratchet', () => {
  sh('node', ['scripts/check-doc-registry.mjs']);
});

runStep('debt marathon contracts', () => {
  sh('node', ['scripts/check-debt-marathon.mjs']);
});

runStep('module quality contracts', () => {
  sh('node', ['scripts/module-quality-harness.mjs', '--self-test']);
  sh('node', ['scripts/module-quality-runner.mjs', '--self-test']);
  sh('node', ['scripts/module-quality-compare.mjs', '--self-test']);
});

runStep('CI execution policy contracts', () => {
  sh('node', ['scripts/ci-execution-policy.mjs', '--self-test']);
  sh('node', ['scripts/collect-ci-compare-evidence.mjs', '--self-test']);
});

runStep('debt marathon stop-hook self-test', () => {
  sh('node', ['scripts/test-cursor-marathon-hook.mjs']);
});

runStep('human module and critical SSOT Markdown links', () => {
  sh('node', ['scripts/check-markdown-links.mjs', '--self-test']);
  sh('node', ['scripts/check-markdown-links.mjs']);
});

runStep('doc mirror ratchet', () => {
  sh('node', ['scripts/check-doc-mirror.mjs', '--self-test']);
  sh('node', ['scripts/check-doc-mirror.mjs', '--warn-drift-high-traffic']);
});

runStep('cargo dedup ratchet (K-SUPPLY-05)', () => {
  sh('node', ['scripts/check-cargo-dedup-ratchet.mjs']);
});

runStep('kernel error codes drift', () => {
  sh('node', ['scripts/check-error-codes-drift.mjs']);
});

runStep('voice TTS ratchet', () => {
  sh('node', ['scripts/check-voice-tts-ratchet.mjs']);
});

runStep('stale doc paths ratchet', () => {
  sh('node', ['scripts/check-stale-paths.mjs', '--docs-only']);
});

runStep('stale code paths ratchet', () => {
  sh('node', ['scripts/check-stale-paths.mjs', '--code-only']);
});

runStep('runtime state is not tracked', () => {
  const tracked = sh('git', [
    'ls-files',
    'distros/chat-pro/roles/.oclive_directory_plugin_data',
  ]).trim();
  if (tracked) {
    throw new Error(
      'runtime state under roles/.oclive_directory_plugin_data must stay ignored and untracked',
    );
  }
});

runStep('host runtime re-export ratchet', () => {
  sh('node', ['scripts/check-host-reexport-imports.mjs']);
});

if (!ciMode) {
  runStep('sample workspace lib tests (host + runtime)', () => {
    sh('cargo', ['test', '-p', 'oclive_kernel_host', '-p', 'oclive_kernel_runtime', '--lib']);
  });
} else {
  console.log('[dimension5] sample workspace lib tests… SKIP (--ci)');
  results.push({ name: 'sample workspace lib tests', ok: true, skipped: true });
}

runStep('theater prompt drift', () => {
  sh('node', ['scripts/theater-prompt-drift.mjs']);
});

runStep('frontend verify:ui anchors', () => {
  sh('node', ['scripts/verify-frontend-patches.mjs']);
});

runStep('module compatibility ratchet', () => {
  sh('node', ['scripts/check-module-compat.mjs']);
  sh('node', ['scripts/validate-plugins-index.mjs']);
});

runStep('tauri major 2 + docs narrative (K-PLATFORM-01c)', () => {
  const cargoToml = fs.readFileSync(
    path.join(repoRoot, 'distros', 'desktop-tauri', 'Cargo.toml'),
    'utf8',
  );
  if (!/^\s*tauri\s*=\s*\{[^}]*version\s*=\s*"2"/m.test(cargoToml)) {
    throw new Error('distros/desktop-tauri/Cargo.toml must declare tauri version "2"');
  }
  const narrativePaths = [
    path.join(repoRoot, 'CONTRIBUTING.md'),
    path.join(repoRoot, 'human-docs', '10_SETUP_WINDOWS.md'),
  ];
  for (const p of narrativePaths) {
    const text = fs.readFileSync(p, 'utf8');
    if (/webkit2gtk-4\.0/.test(text)) {
      throw new Error(`${path.relative(repoRoot, p)} must not require webkit2gtk-4.0 (use 4.1 / Tauri 2)`);
    }
  }
});

runStep('tauri beforeBuildCommand path ratchet', () => {
  const confPath = path.join(repoRoot, 'distros', 'desktop-tauri', 'tauri.conf.json');
  const conf = fs.readFileSync(confPath, 'utf8');
  const parsed = JSON.parse(conf);
  const build = parsed.build?.beforeBuildCommand ?? '';
  const dev = parsed.build?.beforeDevCommand ?? '';
  if (build !== 'node scripts/tauri-run.cjs build' || dev !== 'node scripts/tauri-run.cjs dev') {
    throw new Error(
      'tauri.conf.json beforeBuildCommand/beforeDevCommand must invoke scripts/tauri-run.cjs from the repository root',
    );
  }
  const resources = parsed.bundle?.resources ?? parsed.tauri?.bundle?.resources;
  if (!Array.isArray(resources)) {
    throw new Error('tauri.conf.json bundle.resources must be an array');
  }
  const rolesResourceCount = resources.filter((e) => e === 'resources/roles').length;
  const chatProRolesCount = resources.filter((e) => e === '../chat-pro/roles').length;
  const migrationsResourceCount = resources.filter((e) => e === 'resources/migrations').length;
  if (rolesResourceCount !== 1) {
    throw new Error(
      `tauri.conf.json must have exactly one sanitized resources/roles entry (found ${rolesResourceCount})`,
    );
  }
  if (chatProRolesCount !== 0) {
    throw new Error(
      `tauri.conf.json must not bundle the mutable ../chat-pro/roles tree directly (found ${chatProRolesCount})`,
    );
  }
  if (migrationsResourceCount !== 1) {
    throw new Error(
      `tauri.conf.json must bundle exactly one resources/migrations directory (found ${migrationsResourceCount})`,
    );
  }
  const MAX_RESOURCES = 10;
  if (resources.length > MAX_RESOURCES) {
    throw new Error(
      `tauri.conf.json bundle.resources has ${resources.length} entries (max ${MAX_RESOURCES}); dedupe role paths`,
    );
  }
});

runStep('vite build', () => {
  sh('node', [path.join(repoRoot, 'node_modules/vite/bin/vite.js'), 'build']);
});

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Dimension 5 acceptance: ${failed.length === 0 ? 'PASS' : 'FAIL'} (${results.length} checks)`);
if (failed.length > 0) {
  for (const f of failed) {
    console.error(`  ✗ ${f.name}`);
  }
  process.exit(1);
}
