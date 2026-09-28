import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { countMatchingLines, findNativeExecutable, GateToolchainError, parseRipgrepCounts, runRequiredCommand } from './gate-toolchain.mjs';

function fixture(t) {
  const temporaryRoot = path.resolve(os.tmpdir());
  const prefix = 'oclive-gate-toolchain-';
  const directory = fs.mkdtempSync(path.join(temporaryRoot, prefix));
  t.after(() => {
    const target = path.resolve(directory);
    assert.equal(path.dirname(target), temporaryRoot);
    assert.ok(path.basename(target).startsWith(prefix));
    fs.rmSync(target, { recursive: true, force: true });
  });
  return directory;
}

function withPath(value) {
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => key.toUpperCase() !== 'PATH'));
  return { ...env, PATH: value };
}

test('real ripgrep counts matching lines, excludes requested files and retains the Rust glob', t => {
  const directory = fixture(t);
  const first = path.join(directory, 'first.rs');
  const excluded = path.join(directory, 'excluded.rs');
  fs.writeFileSync(first, 'needle needle\nneedle\nno match\n');
  fs.writeFileSync(excluded, 'needle\n');
  fs.writeFileSync(path.join(directory, 'ignored.txt'), 'needle\n');
  assert.equal(countMatchingLines('needle', [directory], { cwd: directory }), 3);
  assert.equal(countMatchingLines('needle', [directory], { cwd: directory, excludeFiles: [excluded] }), 2);
});

test('real ripgrep exit 1 means zero matches', t => {
  const directory = fixture(t);
  fs.writeFileSync(path.join(directory, 'empty.rs'), 'not this one\n');
  assert.equal(countMatchingLines('absent', [directory], { cwd: directory }), 0);
});

test('real ripgrep regex and missing-target errors are failures, never zero', t => {
  const directory = fixture(t);
  fs.writeFileSync(path.join(directory, 'target.rs'), 'content\n');
  for (const [pattern, target] of [['[', directory], ['content', path.join(directory, 'missing')]]) {
    assert.throws(() => countMatchingLines(pattern, [target], { cwd: directory }), error =>
      error instanceof GateToolchainError && error.code === 'EXIT' && error.exitCode === 2 && error.stderr.length > 0);
  }
});

test('missing PATH tool fails with actionable diagnostics without changing the parent PATH', () => {
  const parentPath = Object.entries(process.env).filter(([key]) => key.toUpperCase() === 'PATH');
  assert.throws(() => countMatchingLines('anything', ['.'], { env: withPath('') }), error =>
    error instanceof GateToolchainError && error.code === 'MISSING' && /rg --version/.test(error.message));
  assert.deepEqual(Object.entries(process.env).filter(([key]) => key.toUpperCase() === 'PATH'), parentPath);
});

test('an existing unusable native executable is not treated as an absent tool or success', t => {
  const directory = fixture(t);
  const name = process.platform === 'win32' ? 'rg.exe' : 'rg';
  const executable = path.join(directory, name);
  fs.writeFileSync(executable, '', { mode: 0o600 });
  const env = withPath(directory);
  assert.equal(findNativeExecutable('rg', { env }), executable);
  assert.throws(() => runRequiredCommand('rg', ['--version'], { env }), error =>
    error instanceof GateToolchainError && error.code === 'EXECUTION');
});

test('native nonzero exit preserves its exit code and stderr', () => {
  assert.throws(() => runRequiredCommand(process.execPath, ['-e', 'process.stderr.write("injected failure");process.exit(7)']), error =>
    error instanceof GateToolchainError && error.code === 'EXIT' && error.exitCode === 7 && error.stderr === 'injected failure');
});

test('native timeout cannot pass as an empty successful command', () => {
  assert.throws(() => runRequiredCommand(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { timeout: 100 }), error =>
    error instanceof GateToolchainError && error.code === 'TIMEOUT');
});

test('NUL records preserve Windows drive colons and filename newlines', () => {
  assert.equal(parseRipgrepCounts('E:\\repo\\source.rs\0' + '3\nname\nwith:colon.rs\0' + '2\n'), 5);
});

test('Windows file exclusion is path-aware and case-insensitive', () => {
  const output = 'E:\\repo\\src\\domain\\mod.rs\0' + '9\nE:\\repo\\src\\consumer.rs\0' + '2\n';
  assert.equal(parseRipgrepCounts(output, { excludeFiles: ['e:/repo/src/domain/mod.rs'] }), 2);
});

test('malformed or overflowing count output fails closed', () => {
  for (const output of ['file.rs:4\n', 'file.rs\0oops\n', 'file.rs\0' + '0\n', '\0' + '1\n', 'file.rs\0' + '1', 'file.rs\0' + '9007199254740992\n', 'a.rs\0' + '9007199254740991\nb.rs\0' + '1\n']) {
    assert.throws(() => parseRipgrepCounts(output), error => error instanceof GateToolchainError && error.code === 'OUTPUT');
  }
});
