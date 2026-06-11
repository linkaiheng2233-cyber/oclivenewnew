#!/usr/bin/env node
/**
 * Verify oclive-vscode/distro.oclive.toml mirrors examples/distro-profiles/vscode.oclive.toml
 * (ignoring comment-only lines).
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const vscodeRoot = path.resolve(repoRoot, '..', 'oclive-vscode');

function normalizeToml(raw) {
  return raw
    .split(/\r?\n/)
    .filter((line) => {
      const t = line.trim();
      return t && !t.startsWith('#');
    })
    .join('\n');
}

const mainPath = path.join(repoRoot, 'examples', 'distro-profiles', 'vscode.oclive.toml');
const extPath = path.join(vscodeRoot, 'distro.oclive.toml');

if (!fs.existsSync(extPath)) {
  console.warn(`[diff-vscode-distro] skip: missing ${extPath}`);
  process.exit(0);
}

const a = normalizeToml(fs.readFileSync(mainPath, 'utf8'));
const b = normalizeToml(fs.readFileSync(extPath, 'utf8'));

if (a !== b) {
  console.error('[diff-vscode-distro] field mismatch between SSOT and VSIX mirror');
  console.error(`  SSOT: ${mainPath}`);
  console.error(`  VSIX: ${extPath}`);
  process.exit(1);
}

console.log('[diff-vscode-distro] vscode.oclive.toml mirror ok');
