#!/usr/bin/env node
/** Default reference-Host feedback; not the workspace/desktop feature combination. */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runGate, runRequiredCommand } from './lib/gate-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const commands = [
  ['clippy', '--locked', '-p', 'oclive_kernel_host', '--all-targets', '--no-deps', '--', '-D', 'warnings'],
  ['test', '--locked', '-p', 'oclive_kernel_host'],
];

export function runReferenceHostChecks(runCommand = runRequiredCommand) {
  for (const args of commands) {
    console.log(`[reference-host] cargo ${args.join(' ')}`);
    const result = runCommand('cargo', args, { cwd: repoRoot, timeout: 20 * 60 * 1000 });
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
  }
  console.log('[reference-host] PASS (default reference Host only; optional features, desktop and distro acceptance not run)');
}

const entryPath = process.argv[1] && path.resolve(process.argv[1]);
const scriptPath = fileURLToPath(import.meta.url);
if (entryPath && (process.platform === 'win32'
  ? entryPath.toLowerCase() === scriptPath.toLowerCase() : entryPath === scriptPath)) {
  if (process.argv.length > 2) {
    console.error('Usage: node scripts/check-reference-host.mjs');
    process.exitCode = 1;
  } else {
    runGate(() => runReferenceHostChecks());
  }
}
