#!/usr/bin/env node
/** Full contracts/types lint and tests, including their existing compatibility surface. */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runGate, runRequiredCommand } from './lib/gate-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packages = ['-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types'];
const commands = [
  ['clippy', '--locked', ...packages, '--all-targets', '--all-features', '--no-deps', '--', '-D', 'warnings'],
  ['test', '--locked', ...packages],
];

export function runKernelContractChecks(runCommand = runRequiredCommand) {
  for (const args of commands) {
    console.log(`[kernel-contracts] cargo ${args.join(' ')}`);
    const result = runCommand('cargo', args, { cwd: repoRoot, timeout: 20 * 60 * 1000 });
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
  }
  console.log('[kernel-contracts] PASS (contracts/types and compatibility only; shared runtime, Host and distro checks not run)');
}

const entryPath = process.argv[1] && path.resolve(process.argv[1]);
const scriptPath = fileURLToPath(import.meta.url);
if (entryPath && (process.platform === 'win32'
  ? entryPath.toLowerCase() === scriptPath.toLowerCase() : entryPath === scriptPath)) {
  if (process.argv.length > 2) {
    console.error('Usage: node scripts/check-kernel-contracts.mjs');
    process.exitCode = 1;
  } else {
    runGate(() => runKernelContractChecks());
  }
}
