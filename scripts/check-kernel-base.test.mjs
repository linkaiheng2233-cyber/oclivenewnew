import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { runKernelBaseChecks } from './check-kernel-base.mjs';
import { GateToolchainError, runRequiredCommand } from './lib/gate-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(repoRoot, 'scripts/check-kernel-base.mjs');
const child = (args, env = process.env) => spawnSync(process.execPath, [script, ...args], {
  cwd: repoRoot, env, encoding: 'utf8', windowsHide: true, timeout: 30000,
});

test('the local chain only selects the existing Base/minimal-role checks and two public crates', () => {
  const calls = [];
  runKernelBaseChecks((command, args, options) => {
    calls.push({ command, args, options });
    return { stdout: '', stderr: '' };
  });
  assert.deepEqual(calls.map(call => [call.command, ...call.args]), [
    ['cargo', 'test', '--locked', '-p', 'oclive_kernel_contracts', '--test', 'base_only_fixture'],
    ['cargo', 'test', '--locked', '-p', 'oclive_kernel_types', '--test', 'minimal_role_contract'],
    ['cargo', 'clippy', '--locked', '-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types', '--all-targets', '--', '-D', 'warnings'],
    ['cargo', 'test', '--locked', '-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types', '--doc'],
  ]);
  assert.ok(calls.every(call => call.options.cwd === repoRoot));
});

test('a real native nonzero result stops the same chain before Clippy and doctest', () => {
  const calls = [];
  assert.throws(() => runKernelBaseChecks((command, args) => {
    calls.push([command, ...args]);
    if (calls.length === 2) {
      return runRequiredCommand(process.execPath, ['-e', 'process.stderr.write("native fixture failed"); process.exit(7)']);
    }
    return { stdout: '', stderr: '' };
  }), error => error instanceof GateToolchainError
    && error.exitCode === 7 && error.stderr === 'native fixture failed');
  assert.equal(calls.length, 2);
});

test('the real CLI fails without cargo and never prints PASS or attempts the second step', () => {
  const env = {
    ...Object.fromEntries(Object.entries(process.env).filter(([key]) => key.toUpperCase() !== 'PATH')),
    PATH: '',
  };
  const result = child([], env);
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /\[toolchain:MISSING\]/);
  assert.match(result.stderr, /cargo --version/);
  assert.match(result.stdout, /base_only_fixture/);
  assert.doesNotMatch(result.stdout, /minimal_role_contract|PASS/);
});

test('the real CLI rejects unsupported options before starting checks', () => {
  const result = child(['--skip']);
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /Usage:/);
  assert.doesNotMatch(result.stdout, /cargo|PASS/);
});
