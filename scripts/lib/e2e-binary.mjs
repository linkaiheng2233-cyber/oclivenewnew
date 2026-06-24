/**
 * SSOT: kernel / CLI binary discovery for e2e scripts (monorepo layout).
 */
import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { resolveRepoRoot } from './chat-pro-roles-dir.mjs';

export function kernelExeName() {
  return process.platform === 'win32' ? 'oclive-kernel-server.exe' : 'oclive-kernel-server';
}

export function cliExeName() {
  return process.platform === 'win32' ? 'oclive-cli.exe' : 'oclive-cli';
}

export function exeSuffix() {
  return process.platform === 'win32' ? '.exe' : '';
}

/** Cargo `target_directory` for the workspace (honours CARGO_TARGET_DIR). */
export function cargoTargetDir(repoRoot = resolveRepoRoot()) {
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

function externalArtifactTarget(repoRoot) {
  return path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target');
}

/** Candidate paths for a workspace binary (debug profile). */
export function kernelBinaryCandidates(repoRoot = resolveRepoRoot()) {
  const name = kernelExeName();
  const suffix = exeSuffix();
  const candidates = [];
  const target = cargoTargetDir(repoRoot);
  if (target) {
    candidates.push(path.join(target, 'debug', name));
    candidates.push(path.join(target, 'release', name));
  }
  const ext = externalArtifactTarget(repoRoot);
  candidates.push(path.join(ext, 'debug', name));
  candidates.push(path.join(ext, 'release', name));
  candidates.push(path.join(repoRoot, 'target', 'debug', name));
  candidates.push(path.join(repoRoot, 'target', 'release', name));
  candidates.push(path.join(repoRoot, 'distros', 'desktop-tauri', 'target', 'debug', name));
  candidates.push(path.join(repoRoot, 'distros', 'desktop-tauri', 'target', 'release', name));
  // Legacy tauri binary names (cross-host smoke)
  candidates.push(path.join(ext, 'debug', `oclivenewnew-tauri${suffix}`));
  candidates.push(path.join(repoRoot, 'target', 'debug', `oclivenewnew-tauri${suffix}`));
  return candidates;
}

export function cliBinaryCandidates(repoRoot = resolveRepoRoot()) {
  const name = `oclive-cli${exeSuffix()}`;
  const candidates = [];
  const target = cargoTargetDir(repoRoot);
  if (target) {
    candidates.push(path.join(target, 'debug', name));
    candidates.push(path.join(target, 'release', name));
  }
  const ext = externalArtifactTarget(repoRoot);
  candidates.push(path.join(ext, 'debug', name));
  candidates.push(path.join(repoRoot, 'target', 'debug', name));
  return candidates;
}

export function findKernelBinary(repoRoot = resolveRepoRoot()) {
  const fromEnv = process.env.OCLIVE_E2E_KERNEL;
  if (fromEnv && fs.existsSync(fromEnv)) {
    return fromEnv;
  }
  return kernelBinaryCandidates(repoRoot).find((p) => fs.existsSync(p)) ?? null;
}

export function findCliBinary(repoRoot = resolveRepoRoot()) {
  const fromEnv = process.env.OCLIVE_E2E_CLI;
  if (fromEnv && fs.existsSync(fromEnv)) {
    return fromEnv;
  }
  return cliBinaryCandidates(repoRoot).find((p) => fs.existsSync(p)) ?? null;
}

export function resolveBinary(binaryName, repoRoot = resolveRepoRoot()) {
  if (binaryName === 'oclive-kernel-server') {
    return findKernelBinary(repoRoot);
  }
  if (binaryName === 'oclive-cli') {
    return findCliBinary(repoRoot);
  }
  const suffix = exeSuffix();
  const target = cargoTargetDir(repoRoot);
  const candidates = [];
  if (target) {
    candidates.push(path.join(target, 'debug', `${binaryName}${suffix}`));
  }
  candidates.push(
    path.join(externalArtifactTarget(repoRoot), 'debug', `${binaryName}${suffix}`),
    path.join(repoRoot, 'target', 'debug', `${binaryName}${suffix}`),
  );
  return candidates.find((p) => fs.existsSync(p)) ?? null;
}
