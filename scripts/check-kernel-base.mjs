#!/usr/bin/env node
/** Existing public Base/minimal-role checks; does not prepare or start a Host. */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runGateEntry, runRequiredCommand } from './lib/gate-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const commands = [
  ['test', '--locked', '-p', 'oclive_kernel_contracts', '--test', 'base_only_fixture'],
  ['test', '--locked', '-p', 'oclive_kernel_types', '--test', 'minimal_role_contract'],
  ['clippy', '--locked', '-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types', '--all-targets', '--', '-D', 'warnings'],
  ['test', '--locked', '-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types', '--doc'],
];

// The runner seam is only for this fixed chain's stop-on-failure regression.
export function runKernelBaseChecks(runCommand = runRequiredCommand) {
  for (const args of commands) {
    console.log(`[kernel-base] cargo ${args.join(' ')}`);
    const result = runCommand('cargo', args, { cwd: repoRoot, timeout: 20 * 60 * 1000 });
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
  }
  console.log('[kernel-base] PASS (Base/minimal-role checks only; Host and distro checks not run)');
}

runGateEntry(import.meta.url, runKernelBaseChecks, 'Usage: node scripts/check-kernel-base.mjs');
