#!/usr/bin/env node
/**
 * Distro kernel e2e — spawn / attach / role_snapshot scenarios.
 * Usage: node scripts/e2e-distro-kernel.mjs [--scenario spawn|attach|role-snapshot|bundled-first|theater|all]
 */
import { spawn, spawnSync } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const port = Number(process.env.OCLIVE_E2E_PORT || 18421);
const appData = path.join(os.tmpdir(), `oclive_distro_e2e_${Date.now()}`);
const rolesDir = path.join(repoRoot, 'roles');

const scenario = (() => {
  const i = process.argv.indexOf('--scenario');
  return i >= 0 ? process.argv[i + 1] : 'all';
})();

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function healthOk(p = port) {
  try {
    const res = await fetch(`http://127.0.0.1:${p}/health`, { signal: AbortSignal.timeout(2000) });
    return res.ok;
  } catch {
    return false;
  }
}

function findKernelBinary() {
  if (process.env.OCLIVE_E2E_KERNEL && fs.existsSync(process.env.OCLIVE_E2E_KERNEL)) {
    return process.env.OCLIVE_E2E_KERNEL;
  }
  const candidates = [
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclive-kernel-server.exe'),
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclive-kernel-server'),
    path.join(repoRoot, 'target', 'debug', 'oclive-kernel-server.exe'),
    path.join(repoRoot, 'target', 'debug', 'oclive-kernel-server'),
    path.join(repoRoot, 'distros/desktop-tauri', 'target', 'debug', 'oclive-kernel-server'),
    path.join(repoRoot, 'distros/desktop-tauri', 'target', 'debug', 'oclive-kernel-server.exe'),
  ];
  return candidates.find((p) => fs.existsSync(p));
}

function spawnKernel(extraEnv = {}) {
  const bin = findKernelBinary();
  if (!bin) {
    throw new Error('no oclive-kernel-server binary');
  }
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
  return { child, bin };
}

async function waitReady() {
  for (let i = 0; i < 40; i++) {
    if (await healthOk()) {
      return;
    }
    await sleep(500);
  }
  throw new Error('/health timeout');
}

async function scenarioSpawn() {
  console.log('[e2e-distro] scenario: spawn');
  const { child } = spawnKernel();
  try {
    await waitReady();
    const res = await fetch(`http://127.0.0.1:${port}/chat`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        role_path: path.join(rolesDir, 'mumu'),
        message: 'distro spawn e2e',
        session_id: 'e2e-spawn',
        scene_id: 'desktop',
      }),
    });
    const body = await res.json();
    if (!res.ok || !body.reply) {
      throw new Error(`chat failed: ${JSON.stringify(body)}`);
    }
    console.log('[e2e-distro] spawn ok');
  } finally {
    child.kill();
  }
}

async function scenarioAttach() {
  console.log('[e2e-distro] scenario: attach');
  const first = spawnKernel();
  try {
    await waitReady();
    if (await healthOk()) {
      console.log('[e2e-distro] attach: existing kernel healthy (simulates second distro)');
    } else {
      throw new Error('expected attach target');
    }
  } finally {
    first.child.kill();
  }
}

async function scenarioRoleSnapshot() {
  console.log('[e2e-distro] scenario: role-snapshot');
  const { child } = spawnKernel();
  try {
    await waitReady();
    await fetch(`http://127.0.0.1:${port}/role/load`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ role_id: 'mumu' }),
    });
    const snapRes = await fetch(`http://127.0.0.1:${port}/role_snapshot?role_id=mumu&scene_id=desktop`);
    const text = await snapRes.text();
    const snap = text ? JSON.parse(text) : null;
    if (!snapRes.ok || typeof snap.current_favorability !== 'number') {
      throw new Error(`role_snapshot failed: ${JSON.stringify(snap)}`);
    }
    console.log('[e2e-distro] role_snapshot ok', snap.relation_state, snap.portrait_emotion);
  } finally {
    child.kill();
  }
}

async function scenarioBundledFirst() {
  console.log('[e2e-distro] scenario: bundled-first');
  const targetDir = JSON.parse(
    spawnSync('cargo', ['metadata', '--format-version=1', '--no-deps'], {
      encoding: 'utf8',
      cwd: repoRoot,
    }).stdout,
  ).target_directory;
  const exe = process.platform === 'win32' ? 'oclive-kernel-server.exe' : 'oclive-kernel-server';
  const cli = process.platform === 'win32' ? 'oclive-cli.exe' : 'oclive-cli';
  const bundled = path.join(repoRoot, 'distros/desktop-tauri', 'resources', exe);
  if (!fs.existsSync(bundled)) {
    spawnSync(process.execPath, ['scripts/bundle-kernel-for-tauri.mjs', '--profile', 'debug'], {
      cwd: repoRoot,
      stdio: 'inherit',
    });
  }
  if (!fs.existsSync(bundled)) {
    throw new Error(`missing bundled kernel at ${bundled}`);
  }
  const cliBin = path.join(targetDir, 'debug', cli);
  const profile = path.join(repoRoot, 'examples', 'distro-profiles', 'desktop.oclive.toml');
  const out = spawnSync(
    cliBin,
    [
      'kernel',
      'ensure',
      '--plan-only',
      '--json',
      '--path',
      repoRoot,
      '--roles-dir',
      rolesDir,
      '--distro',
      'desktop',
      '--distro-profile',
      profile,
      '--bundled-binary',
      bundled,
    ],
    { encoding: 'utf8', cwd: repoRoot },
  );
  if (out.status !== 0) {
    throw new Error(out.stderr || out.stdout);
  }
  const report = JSON.parse(out.stdout);
  if (report.plan?.action !== 'spawn_best' || report.plan?.candidate?.tier !== 'bundled') {
    throw new Error(`bundled-first plan mismatch: ${JSON.stringify(report.plan)}`);
  }
  console.log('[e2e-distro] bundled-first ok');
}

async function scenarioTheater() {
  console.log('[e2e-distro] scenario: theater');
  const profile = path.join(repoRoot, 'examples', 'distro-profiles', 'theater.oclive.toml');
  if (!fs.existsSync(profile)) {
    throw new Error(`missing theater profile: ${profile}`);
  }
  const { child } = spawnKernel({ OCLIVE_DISTRO_PROFILE: profile });
  try {
    await waitReady();
    const res = await fetch(`http://127.0.0.1:${port}/health`, {
      headers: { Accept: 'application/json' },
    });
    const body = await res.json();
    if (!res.ok) {
      throw new Error(`/health failed: ${JSON.stringify(body)}`);
    }
    const distroId = body.distro_id ?? body.active_profile_summary?.distro_id;
    if (distroId !== 'theater') {
      throw new Error(`expected distro_id=theater, got ${JSON.stringify(distroId)}`);
    }
    console.log('[e2e-distro] theater ok');
  } finally {
    child.kill();
  }
}

async function main() {
  if (!findKernelBinary()) {
    console.warn('[e2e-distro] skip: build oclive-kernel-server first');
    process.exit(0);
  }
  const run = scenario === 'all'
    ? ['spawn', 'attach', 'role-snapshot', 'bundled-first', 'theater']
    : [scenario];
  for (const s of run) {
    if (s === 'spawn') await scenarioSpawn();
    else if (s === 'attach') await scenarioAttach();
    else if (s === 'role-snapshot') await scenarioRoleSnapshot();
    else if (s === 'bundled-first') await scenarioBundledFirst();
    else if (s === 'theater') await scenarioTheater();
    else {
      console.error(`unknown scenario: ${s}`);
      process.exit(1);
    }
  }
  console.log('[e2e-distro] all requested scenarios passed');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
