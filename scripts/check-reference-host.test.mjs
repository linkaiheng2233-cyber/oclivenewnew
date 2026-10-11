import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { runReferenceHostChecks } from './check-reference-host.mjs';
import { GateToolchainError, runRequiredCommand } from './lib/gate-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(repoRoot, 'scripts/check-reference-host.mjs');
const child = (args, env = process.env) => spawnSync(process.execPath, [script, ...args], {
  cwd: repoRoot, env, encoding: 'utf8', windowsHide: true, timeout: 30000,
});

test('default Host feedback does not replace all-feature workspace or distro checks', () => {
  const calls = [];
  runReferenceHostChecks((command, args, options) => {
    calls.push({ command, args, options });
    return { stdout: '', stderr: '' };
  });
  assert.deepEqual(calls.map(c => [c.command, ...c.args]), [
    ['cargo', 'clippy', '--locked', '-p', 'oclive_kernel_host', '--all-targets', '--no-deps', '--', '-D', 'warnings'],
    ['cargo', 'test', '--locked', '-p', 'oclive_kernel_host'],
  ]);
  assert.ok(calls.every(c => c.options.cwd === repoRoot && c.options.timeout === 20 * 60 * 1000));
  const manifest = fs.readFileSync(path.join(repoRoot, 'kernel/crates/oclive_kernel_host/Cargo.toml'), 'utf8');
  assert.match(manifest, /^default = \[\]$/m, 'scope must be reviewed if default Host features change');
});

test('a native Clippy failure stops the same chain before Host tests', () => {
  let calls = 0;
  assert.throws(() => runReferenceHostChecks(() => {
    calls++;
    return runRequiredCommand(process.execPath, ['-e', 'process.stderr.write("lint failed"); process.exit(7)']);
  }), e => e instanceof GateToolchainError && e.exitCode === 7 && e.stderr === 'lint failed');
  assert.equal(calls, 1);
});

test('a native Host-test failure cannot become a successful chain', () => {
  let calls = 0;
  assert.throws(() => runReferenceHostChecks(() => {
    calls++;
    return calls === 1 ? { stdout: '', stderr: '' }
      : runRequiredCommand(process.execPath, ['-e', 'process.exit(9)']);
  }), e => e instanceof GateToolchainError && e.exitCode === 9);
  assert.equal(calls, 2);
});

test('the real entry fails without Cargo and emits no PASS', () => {
  const env = { ...Object.fromEntries(Object.entries(process.env).filter(([k]) => k.toUpperCase() !== 'PATH')), PATH: '' };
  const result = child([], env);
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /\[toolchain:MISSING\]/);
  assert.doesNotMatch(result.stdout, /PASS|cargo test/);
});

test('unsupported flags cannot silently change the default feature scope', () => {
  const result = child(['--all-features']);
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /Usage:/);
  assert.doesNotMatch(result.stdout, /cargo|PASS/);
});
