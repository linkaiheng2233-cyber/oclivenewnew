#!/usr/bin/env node
/**
 * Tauri bundled kernel smoke — bundle into resources/ and verify K-SCHED-05 plan picks bundled.
 */
import { spawnSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

import { chatProRolesDir, resolveRepoRoot } from './lib/chat-pro-roles-dir.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = resolveRepoRoot();

function sh(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { encoding: 'utf8', cwd: repoRoot, ...opts });
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} failed:\n${r.stderr || r.stdout}`);
  }
  return (r.stdout || '').trim();
}

function kernelExeName() {
  return process.platform === 'win32' ? 'oclive-kernel-server.exe' : 'oclive-kernel-server';
}

console.log('[e2e-tauri-bundled] bundling kernel into Tauri resources...');
sh(process.execPath, ['scripts/bundle-kernel-for-tauri.mjs', '--profile', 'debug'], {
  stdio: 'inherit',
});

const bundled = path.join(repoRoot, 'distros/desktop-tauri', 'resources', kernelExeName());
const manifest = path.join(repoRoot, 'distros/desktop-tauri', 'resources', 'oclive-kernel-server.oclive-manifest.json');
if (!fs.existsSync(bundled)) {
  throw new Error(`missing bundled kernel: ${bundled}`);
}
if (!fs.existsSync(manifest)) {
  throw new Error(`missing manifest sidecar: ${manifest}`);
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
console.log('[e2e-tauri-bundled] resources kernel + manifest present');
