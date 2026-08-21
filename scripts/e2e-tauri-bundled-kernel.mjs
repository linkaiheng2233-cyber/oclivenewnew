#!/usr/bin/env node
/**
 * Tauri bundled kernel smoke — bundle into resources/ and verify K-SCHED-05 plan picks bundled.
 */
import { spawn, spawnSync } from 'child_process';
import fs from 'fs';
import net from 'net';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

import { chatProRolesDir, resolveRepoRoot } from './lib/chat-pro-roles-dir.mjs';
import { kernelExeName } from './lib/e2e-binary.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = resolveRepoRoot();

function assertPortraitBlobAllowedByCsp() {
  const configPath = path.join(repoRoot, 'distros', 'desktop-tauri', 'tauri.conf.json');
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  const csp = config.app?.security?.csp;
  if (typeof csp !== 'string') {
    throw new Error('tauri.conf.json app.security.csp must be a string');
  }
  const imgDirective = csp
    .split(';')
    .map((directive) => directive.trim().split(/\s+/))
    .find(([name]) => name === 'img-src');
  if (!imgDirective?.includes('blob:')) {
    throw new Error('tauri.conf.json img-src must allow blob: role portraits');
  }
}

function sh(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { encoding: 'utf8', cwd: repoRoot, ...opts });
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} failed:\n${r.stderr || r.stdout}`);
  }
  return (r.stdout || '').trim();
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      server.close((error) => (error ? reject(error) : resolve(port)));
    });
  });
}

async function portableRuntimeSmoke(bundled, manifest, migrations, bundledRoles) {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-bundled-runtime-'));
  const binary = path.join(temp, kernelExeName());
  const roles = path.join(temp, 'roles');
  const portableMigrations = path.join(temp, 'migrations');
  fs.copyFileSync(bundled, binary);
  fs.cpSync(bundledRoles, roles, { recursive: true });
  fs.cpSync(migrations, portableMigrations, { recursive: true });

  const port = await freePort();
  const token = `oclive-bundled-smoke-${process.pid}`;
  const env = {
    ...process.env,
    OCLIVE_API_PORT: String(port),
    OCLIVE_API_TOKEN: token,
    OCLIVE_HTTP_API_MOCK_LLM: '1',
    OCLIVE_API_USE_TEMP_APP_DATA: '1',
    OCLIVE_MIGRATIONS_DIR: portableMigrations,
  };
  delete env.OCLIVE_APP_DATA;
  delete env.OCLIVE_LOCAL_MONOREPO;
  delete env.OCLIVE_ROLES_DIR;

  const child = spawn(binary, ['--api', '--port', String(port)], {
    cwd: temp,
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', (chunk) => { stdout += chunk; });
  child.stderr.on('data', (chunk) => { stderr += chunk; });

  try {
    const base = `http://127.0.0.1:${port}`;
    let ready = false;
    for (let attempt = 0; attempt < 80; attempt += 1) {
      if (child.exitCode !== null) break;
      let health;
      try {
        health = await fetch(`${base}/health`, {
          headers: { accept: 'application/json' },
        });
      } catch {
        // Keep polling while the copied binary initializes its temporary database.
        await new Promise((resolve) => setTimeout(resolve, 125));
        continue;
      }
      if (health.ok) {
        const body = await health.json();
        const expectedManifest = JSON.parse(fs.readFileSync(manifest, 'utf8'));
        if (JSON.stringify(body.kernel_manifest) !== JSON.stringify(expectedManifest)) {
          throw new Error(
            `runtime kernel manifest drifted from sidecar: ${JSON.stringify(body.kernel_manifest)}`,
          );
        }
        ready = true;
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 125));
    }
    if (!ready) {
      throw new Error(`portable bundled kernel did not become ready\n${stderr || stdout}`);
    }
    const role = await fetch(`${base}/role_info?role_id=mumu`, {
      headers: { 'x-oclive-api-token': token },
    });
    const body = await role.json();
    if (!role.ok || body.role_id !== 'mumu') {
      throw new Error(`portable role discovery failed: ${JSON.stringify(body)}`);
    }
  } finally {
    if (child.exitCode === null) {
      const exited = new Promise((resolve) => child.once('exit', resolve));
      child.kill();
      await Promise.race([
        exited,
        new Promise((resolve) => setTimeout(resolve, 2_000)),
      ]);
    }
    fs.rmSync(temp, { recursive: true, force: true });
  }
}

console.log('[e2e-tauri-bundled] bundling kernel into Tauri resources...');
assertPortraitBlobAllowedByCsp();
sh(process.execPath, ['scripts/stage-chat-pro-roles.mjs'], {
  stdio: 'inherit',
});
sh(process.execPath, ['scripts/stage-chat-pro-plugins.mjs'], {
  stdio: 'inherit',
});
sh(process.execPath, ['scripts/bundle-kernel-for-tauri.mjs', '--profile', 'debug'], {
  stdio: 'inherit',
});

const bundled = path.join(repoRoot, 'distros/desktop-tauri', 'resources', kernelExeName());
const manifest = path.join(repoRoot, 'distros/desktop-tauri', 'resources', 'oclive-kernel-server.oclive-manifest.json');
const migrations = path.join(repoRoot, 'distros/desktop-tauri', 'resources', 'migrations');
const bundledRoles = path.join(repoRoot, 'distros/desktop-tauri', 'resources', 'roles');
const bundledPlugins = path.join(repoRoot, 'distros/desktop-tauri', 'resources', 'plugins');
const bundledRepair = path.join(repoRoot, 'distros', 'desktop-tauri', 'resources', 'support', 'Repair-AILiveChatPro.ps1');
if (!fs.existsSync(bundledRepair)) {
  throw new Error(`missing bundled recovery wrapper: ${bundledRepair}`);
}
if (!fs.existsSync(bundled)) {
  throw new Error(`missing bundled kernel: ${bundled}`);
}
if (!fs.existsSync(manifest)) {
  throw new Error(`missing manifest sidecar: ${manifest}`);
}
if (!fs.existsSync(migrations)) {
  throw new Error(`missing bundled migrations: ${migrations}`);
}
if (!fs.existsSync(bundledRoles)) {
  throw new Error(`missing bundled roles: ${bundledRoles}`);
}
for (const pluginId of [
  'com.oclive.mumu.chat-header-status',
  'com.oclive.mumu.quick-actions',
  'com.oclive.mumu.role-detail-card',
  'com.oclive.mumu.settings-panel',
  'com.oclive.mumu.sidebar-glance',
  'com.oclive.theater_director_official',
  'com.oclive.voice.asr',
]) {
  if (!fs.existsSync(path.join(bundledPlugins, pluginId, 'manifest.json'))) {
    throw new Error(`missing bundled production plugin: ${pluginId}`);
  }
}
if (fs.existsSync(path.join(bundledRoles, '.oclive_directory_plugin_data'))) {
  throw new Error('bundled roles must not contain ignored local role state');
}

const targetDir = JSON.parse(
  sh('cargo', ['metadata', '--format-version=1', '--no-deps']),
).target_directory;
const profile = path.join(repoRoot, 'examples', 'distro-profiles', 'desktop.oclive.toml');
const cliBin = path.join(targetDir, 'debug', process.platform === 'win32' ? 'oclive-cli.exe' : 'oclive-cli');
if (!fs.existsSync(cliBin)) {
  sh('cargo', ['build', '-p', 'oclive-cli'], { stdio: 'inherit' });
}

const planJson = sh(cliBin, [
  'kernel',
  'ensure',
  '--plan-only',
  '--json',
  '--path',
  repoRoot,
  '--roles-dir',
  chatProRolesDir(repoRoot),
  '--distro',
  'desktop',
  '--distro-profile',
  profile,
  '--bundled-binary',
  bundled,
]);

const report = JSON.parse(planJson);
if (report.plan?.action !== 'spawn_best') {
  throw new Error(`expected spawn_best, got ${JSON.stringify(report.plan)}`);
}
if (report.plan?.candidate?.tier !== 'bundled') {
  throw new Error(`expected bundled tier, got ${JSON.stringify(report.plan?.candidate)}`);
}
if (report.plan?.degraded) {
  throw new Error('bundled-first spawn should not be degraded');
}

console.log('[e2e-tauri-bundled] bundled-first plan ok');
await portableRuntimeSmoke(bundled, manifest, migrations, bundledRoles);
console.log('[e2e-tauri-bundled] portable resources kernel + roles + migrations ok');
