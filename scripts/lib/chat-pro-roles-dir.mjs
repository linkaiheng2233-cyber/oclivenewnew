import path from 'path';
import { fileURLToPath } from 'url';

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));

/** Monorepo root (oclivenewnew). */
export function resolveRepoRoot() {
  const fromEnv = process.env.GITHUB_WORKSPACE || process.env.OCLIVE_ROOT;
  if (fromEnv) return path.resolve(fromEnv);
  return path.resolve(scriptsDir, '..', '..');
}

/** Canonical Chat Pro role packs directory after kernel/distros split. */
export function chatProRolesDir(repoRoot = resolveRepoRoot()) {
  return path.join(repoRoot, 'distros', 'chat-pro', 'roles');
}
