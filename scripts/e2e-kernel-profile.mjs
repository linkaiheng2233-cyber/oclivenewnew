#!/usr/bin/env node
/**
 * Profile-aware kernel scheduling e2e (plan-only via oclive-cli when available).
 *
 * Usage: node scripts/e2e-kernel-profile.mjs
 */
import { spawn, execFileSync } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const port = Number(process.env.OCLIVE_E2E_PORT || 18422);
const rolesDir = path.join(repoRoot, 'roles');
const vscodeProfile = path.join(repoRoot, 'examples/distro-profiles/vscode.oclive.toml');
const desktopProfile = path.join(repoRoot, 'examples/distro-profiles/desktop.oclive.toml');

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function cargoTargetDir() {
  if (process.env.CARGO_TARGET_DIR) {
    return process.env.CARGO_TARGET_DIR;
  }
  try {
    const out = execFileSync(
      'cargo',
      ['metadata', '--format-version=1', '--no-deps'],
      { cwd: repoRoot, encoding: 'utf8' },
    );
    return JSON.parse(out).target_directory;
  } catch {
    return null;
  }
}

function resolveBinary(name) {
  const envKey = name === 'oclive-kernel-server' ? 'OCLIVE_E2E_KERNEL' : 'OCLIVE_E2E_CLI';
  const fromEnv = process.env[envKey];
  if (fromEnv && fs.existsSync(fromEnv)) {
    return fromEnv;
  }
  const target = cargoTargetDir();
  const suffix = process.platform === 'win32' ? '.exe' : '';
  const candidates = [];
  if (target) {
    candidates.push(path.join(target, 'debug', `${name}${suffix}`));
  }
  candidates.push(
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', `${name}${suffix}`),
    path.join(repoRoot, 'target', 'debug', `${name}${suffix}`),
  );
  return candidates.find((p) => fs.existsSync(p));
}

function findKernelBinary() {
  return resolveBinary('oclive-kernel-server');
}

function findCli() {
  return resolveBinary('oclive-cli');
}

function spawnKernel(extraEnv = {}) {
  const bin = findKernelBinary();
  if (!bin) throw new Error('build oclive-kernel-server first');
  const appData = path.join(os.tmpdir(), `oclive_profile_e2e_${Date.now()}`);
  fs.mkdirSync(appData, { recursive: true });
  const child = spawn(bin, ['--api', '--port', String(port)], {
    env: {
      ...process.env,
      OCLIVE_APP_DATA: appData,
      OCLIVE_USE_CANONICAL_APP_DATA: '1',
      OCLIVE_HTTP_API_MOCK_LLM: '1',
      OCLIVE_ROLES_DIR: rolesDir,
      ...extraEnv,
    },
    stdio: 'ignore',
    windowsHide: true,
  });
  return { child, appData };
}

async function waitHealth() {
  for (let i = 0; i < 40; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/health`, {
        headers: { Accept: 'application/json' },
        signal: AbortSignal.timeout(2000),
      });
      if (res.ok) {
        return res.json();
      }
    } catch {
      /* retry */
    }
    await sleep(500);
  }
  throw new Error('/health timeout');
}

function runEnsurePlan(distro, profilePath, extraArgs = []) {
  const cli = findCli();
  if (!cli) {
    console.warn('[e2e-profile] skip CLI tests: oclive-cli not built');
    return null;
  }
  const out = execFileSync(
    cli,
    [
      'kernel',
      'ensure',
      '--json',
      '--plan-only',
      '--port',
      String(port),
      '--path',
      repoRoot,
      '--roles-dir',
      rolesDir,
      '--distro',
      distro,
      '--distro-profile',
      profilePath,
      ...extraArgs,
    ],
    { encoding: 'utf8', windowsHide: true },
  );
  return JSON.parse(out);
}

async function main() {
  if (!findKernelBinary()) {
    console.warn('[e2e-profile] skip: no kernel binary');
    process.exit(0);
  }

  // Best-effort: avoid stale kernel on test port from prior runs.
  try {
    execFileSync('powershell', [
      '-NoProfile',
      '-Command',
      `Get-NetTCPConnection -LocalPort ${port} -State Listen -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }`,
    ], { windowsHide: true });
  } catch {
    /* ignore */
  }

  // Desktop kernel running
  const desktop = spawnKernel({
    OCLIVE_DISTRO_ID: 'desktop',
    OCLIVE_DISTRO_PROFILE: desktopProfile,
  });
  try {
    const health = await waitHealth();
    if (!health.active_profile_summary) {
      throw new Error('expected active_profile_summary on /health');
    }
    const enabled = health.active_profile_summary.enabled_modules
      ?? health.active_profile_summary.enabledModules
      ?? [];
    if (!enabled.includes('agent')) {
      throw new Error('desktop summary should enable agent');
    }

    const vscodePlan = runEnsurePlan('vscode', vscodeProfile);
    if (vscodePlan) {
      if (vscodePlan.plan.action !== 'replace_and_attach') {
        throw new Error(`vscode vs desktop kernel: expected replace, got ${vscodePlan.plan.action}`);
      }
      if (vscodePlan.plan.replace_reason !== 'profile_mismatch') {
        throw new Error(`expected profile_mismatch, got ${vscodePlan.plan.replace_reason}`);
      }
      console.log('[e2e-profile] vscode vs desktop → replace profile_mismatch ok');
    }

    const desktopPlan = runEnsurePlan('desktop', desktopProfile);
    if (desktopPlan) {
      if (desktopPlan.plan.action !== 'attach') {
        throw new Error(`desktop vs desktop kernel: expected attach, got ${desktopPlan.plan.action}`);
      }
      console.log('[e2e-profile] desktop vs desktop → attach ok');
    }

    const locked = runEnsurePlan('vscode', vscodeProfile, ['--lock-running']);
    if (locked) {
      if (locked.plan.action !== 'attach') {
        throw new Error(`lock-running: expected attach, got ${locked.plan.action}`);
      }
      if (locked.plan.attach_reason !== 'profile_mismatch_no_replace') {
        throw new Error(`lock-running: expected profile_mismatch_no_replace, got ${locked.plan.attach_reason}`);
      }
      console.log('[e2e-profile] lock-running → profile_mismatch_no_replace ok');
    }
  } finally {
    desktop.child.kill();
  }

  console.log('[e2e-profile] all scenarios passed');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
