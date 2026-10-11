import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { runKernelContractChecks } from './check-kernel-contracts.mjs';
import { GateToolchainError, runRequiredCommand } from './lib/gate-toolchain.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts/check-kernel-contracts.mjs');
const child = (args, env = process.env) => spawnSync(process.execPath, [script, ...args], {
  cwd: root, env, encoding: 'utf8', windowsHide: true, timeout: 30000,
});

test('the owner runs full target lint and all tests, not only Base fixtures', () => {
  const calls = [];
  runKernelContractChecks((command, args, options) => {
    calls.push({ command, args, options });
    return { stdout: '', stderr: '' };
  });
  assert.deepEqual(calls.map(call => [call.command, ...call.args]), [
    ['cargo', 'clippy', '--locked', '-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types', '--all-targets', '--all-features', '--no-deps', '--', '-D', 'warnings'],
    ['cargo', 'test', '--locked', '-p', 'oclive_kernel_contracts', '-p', 'oclive_kernel_types'],
  ]);
  assert.ok(calls.every(call => call.options.cwd === root));
  assert.ok(calls.every(call => !call.args.includes('--lib') && !call.args.includes('--test')));
});

test('a native lint failure stops the same owner before any tests', () => {
  let calls = 0;
  assert.throws(() => runKernelContractChecks(() => {
    calls += 1;
    return runRequiredCommand(process.execPath, ['-e', 'process.stderr.write("native lint failed"); process.exit(7)']);
  }), error => error instanceof GateToolchainError && error.exitCode === 7
    && error.stderr === 'native lint failed');
  assert.equal(calls, 1);
});

test('a native test failure remains nonzero after lint succeeds', () => {
  let calls = 0;
  assert.throws(() => runKernelContractChecks(() => {
    calls += 1;
    if (calls === 2)
      return runRequiredCommand(process.execPath, ['-e', 'process.exit(9)']);
    return { stdout: '', stderr: '' };
  }), error => error instanceof GateToolchainError && error.exitCode === 9);
  assert.equal(calls, 2);
});

test('the CLI rejects absent cargo and unsupported arguments without a PASS', () => {
  const env = {
    ...Object.fromEntries(Object.entries(process.env).filter(([key]) => key.toUpperCase() !== 'PATH')),
    PATH: '',
  };
  const missing = child([], env);
  assert.equal(missing.status, 1, missing.stderr);
  assert.match(missing.stderr, /\[toolchain:MISSING\]/);
  assert.doesNotMatch(missing.stdout, /PASS/);
  const unsupported = child(['--skip']);
  assert.equal(unsupported.status, 1, unsupported.stderr);
  assert.match(unsupported.stderr, /Usage:/);
  assert.doesNotMatch(unsupported.stdout, /cargo|PASS/);
});

test('the real workflow topology probe rejects a legacy or broken catalog', () => {
  const workflow = fs.readFileSync(path.join(root, '.github/workflows/ci.yml'), 'utf8');
  const match = workflow.match(/elif ! node -e '([^'\r\n]+)' "\$GITHUB_WORKSPACE\/target\/oclive-ci\/trusted\/data\/ci\/validation-catalog\.v1\.json"; then/);
  assert.ok(match, 'workflow must probe the trusted base, not the PR catalog');
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-contract-owner-'));
  try {
    const file = path.join(directory, 'catalog.json');
    const catalog = JSON.parse(fs.readFileSync(path.join(root, 'data/ci/validation-catalog.v1.json'), 'utf8'));
    const probe = value => {
      fs.writeFileSync(file, typeof value === 'string' ? value : JSON.stringify(value));
      return spawnSync(process.execPath, ['-e', match[1], file], { encoding: 'utf8', windowsHide: true }).status;
    };
    assert.equal(probe(catalog), 0);
    const legacy = structuredClone(catalog);
    legacy.validators = legacy.validators.filter(value => value.id !== 'kernel-contracts');
    assert.equal(probe(legacy), 1);
    const wrongOwner = structuredClone(catalog);
    wrongOwner.validators.find(value => value.id === 'kernel-contracts').workflow_jobs = ['rust'];
    assert.equal(probe(wrongOwner), 1);
    assert.notEqual(probe('{invalid JSON'), 0);
    const resolve = workflow.slice(workflow.indexOf('      - name: Resolve execution policy'), workflow.indexOf('      - name: Upload impact evidence'));
    assert.equal(resolve.match(/--force-full-reason trusted_policy_bootstrap/g)?.length, 2);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('the unchanged gate verifier requires the new owner when selected or full', () => {
  const verify = needs => spawnSync(process.execPath, [
    path.join(root, 'scripts/ci-execution-policy.mjs'), 'verify', '--needs-env', 'CONTRACT_OWNER_NEEDS',
  ], {
    cwd: root, env: { ...process.env, CONTRACT_OWNER_NEEDS: JSON.stringify(needs) },
    encoding: 'utf8', windowsHide: true, timeout: 30000,
  });
  const selected = {
    'ci-impact-plan': { result: 'success', outputs: { run_full: 'false', selected_jobs: '["kernel-contracts","rust"]' } },
    'kernel-contracts': { result: 'success' }, rust: { result: 'success' },
  };
  assert.equal(verify(selected).status, 0);
  for (const result of ['failure', 'skipped', 'cancelled']) {
    const failed = structuredClone(selected);
    failed['kernel-contracts'].result = result;
    const rejected = verify(failed);
    assert.equal(rejected.status, 1, rejected.stdout);
    assert.match(rejected.stderr, /kernel-contracts result mismatch/);
  }
  const omitted = structuredClone(selected);
  delete omitted['kernel-contracts'];
  assert.equal(verify(omitted).status, 1);
  const full = structuredClone(selected);
  full['ci-impact-plan'].outputs.run_full = 'true';
  assert.equal(verify(full).status, 0);
  full['kernel-contracts'].result = 'failure';
  assert.equal(verify(full).status, 1);
  const docs = {
    'ci-impact-plan': { result: 'success', outputs: { run_full: 'false', selected_jobs: '["stale-paths"]' } },
    'kernel-contracts': { result: 'skipped' }, rust: { result: 'skipped' },
    'stale-paths': { result: 'success' },
  };
  assert.equal(verify(docs).status, 0, 'existing docs selection must remain legal');
});
