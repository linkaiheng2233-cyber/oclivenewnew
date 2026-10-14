#!/usr/bin/env node
/** Full contracts/types lint and tests, including their existing compatibility surface. */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runGateEntry, runRequiredCommand } from './lib/gate-toolchain.mjs';

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

runGateEntry(import.meta.url, runKernelContractChecks, 'Usage: node scripts/check-kernel-contracts.mjs');
