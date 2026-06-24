#!/usr/bin/env node
/**
 * Copy release oclive-kernel-server + manifest sidecar into Tauri resources/
 * for Chat Pro bundled-first spawn (K-SCHED-05 / Phase 2a).
 *
 * Usage: node scripts/bundle-kernel-for-tauri.mjs [--profile release|debug]
 */
import { spawnSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { cargoTargetDir, kernelExeName } from './lib/e2e-binary.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const profileArg = process.argv.indexOf('--profile');
const profile = profileArg >= 0 ? process.argv[profileArg + 1] : 'release';
const releaseFlag = profile === 'release' ? ['--release'] : [];

function sh(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { encoding: 'utf8', ...opts });
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} failed: ${r.stderr || r.stdout}`);
  }
  return (r.stdout || '').trim();
}

function manifestName() {
  return 'oclive-kernel-server.oclive-manifest.json';
}

console.log(`[bundle-kernel-for-tauri] building oclive_kernel_server (${profile})...`);
sh('cargo', ['build', '-p', 'oclive_kernel_server', ...releaseFlag], {
  cwd: repoRoot,
  stdio: 'inherit',
});

const srcBin = path.join(cargoTargetDir(repoRoot), profile, kernelExeName());
if (!fs.existsSync(srcBin)) {
  throw new Error(`kernel binary not found: ${srcBin}`);
}

const destDir = path.join(repoRoot, 'distros', 'desktop-tauri', 'resources');
fs.mkdirSync(destDir, { recursive: true });
const destBin = path.join(destDir, kernelExeName());
fs.copyFileSync(srcBin, destBin);

const manifestJson = sh(destBin, ['--version-json'], { cwd: repoRoot });
const destManifest = path.join(destDir, manifestName());
fs.writeFileSync(destManifest, `${manifestJson.trim()}\n`, 'utf8');

// Tauri resources list includes both platform names; stub the non-primary so build succeeds cross-platform.
const stubName =
  process.platform === 'win32' ? 'oclive-kernel-server' : 'oclive-kernel-server.exe';
const stubPath = path.join(destDir, stubName);
if (!fs.existsSync(stubPath)) {
  fs.writeFileSync(stubPath, '');
}

console.log(`[bundle-kernel-for-tauri] bundled -> ${destBin}`);
console.log(`[bundle-kernel-for-tauri] manifest -> ${destManifest}`);

const sumsOut = path.join(destDir, 'SHA256SUMS');
sh('node', [
  path.join(repoRoot, 'scripts/generate-sha256sums.mjs'),
  '--out',
  sumsOut,
  destBin,
  destManifest,
], { cwd: repoRoot, stdio: 'inherit' });
console.log(`[bundle-kernel-for-tauri] checksums -> ${sumsOut}`);
