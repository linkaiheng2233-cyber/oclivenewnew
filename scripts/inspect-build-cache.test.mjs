import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { inspectBuildCache } from './inspect-build-cache.mjs';

const script = fileURLToPath(new URL('./inspect-build-cache.mjs', import.meta.url));
function fixture(t) {
  const temporaryRoot = fs.realpathSync(os.tmpdir());
  const prefix = 'oclive-cache-inspector-';
  const root = fs.mkdtempSync(path.join(temporaryRoot, prefix));
  t.after(() => {
    const target = path.resolve(root);
    assert.equal(path.dirname(target), temporaryRoot);
    assert.ok(path.basename(target).startsWith(prefix));
    assert.equal(fs.realpathSync(target).toLowerCase(), target.toLowerCase());
    fs.rmSync(target, { recursive: true, force: true });
  });
  return root;
}
const cli = args => spawnSync(process.execPath, [script, ...args], {
  encoding: 'utf8', windowsHide: true, timeout: 15000,
});
function reportOf(result, exit) {
  assert.equal(result.status, exit, result.stderr || result.stdout);
  return JSON.parse(result.stdout);
}

test('nested and empty files report logical size while physical and retention facts stay unknown', t => {
  const root = fixture(t);
  fs.mkdirSync(path.join(root, 'deps'));
  fs.writeFileSync(path.join(root, 'empty'), '');
  fs.writeFileSync(path.join(root, 'deps', 'one.rlib'), '12345');
  const report = inspectBuildCache({ root });
  assert.equal(report.complete, true);
  assert.equal(report.regular_file_paths, 2);
  assert.equal(report.directories, 2);
  assert.equal(report.path_logical_bytes, '5');
  assert.equal(report.physical_bytes, null);
  assert.equal(report.reclaimable_bytes, null);
  assert.equal(report.retention_assessed, false);
  assert.equal(report.atomic_snapshot, false);
  assert.equal(fs.readFileSync(path.join(root, 'deps', 'one.rlib'), 'utf8'), '12345');
});

test('hardlinks are counted by path and never as reclaimable objects', t => {
  const root = fixture(t);
  const original = path.join(root, 'one');
  fs.writeFileSync(original, '12345');
  fs.linkSync(original, path.join(root, 'two'));
  const report = inspectBuildCache({ root });
  assert.equal(report.complete, true);
  assert.equal(report.regular_file_paths, 2);
  assert.equal(report.hardlinked_file_paths, 2);
  assert.equal(report.path_logical_bytes, '10');
  assert.equal(report.physical_bytes, null);
  assert.equal(report.reclaimable_bytes, null);
});

test('child junctions are excluded and linked roots or ancestors are refused', t => {
  const parent = fixture(t);
  const root = path.join(parent, 'cache');
  const outside = path.join(parent, 'outside');
  fs.mkdirSync(root);
  fs.mkdirSync(outside);
  fs.writeFileSync(path.join(outside, 'untouched'), 'outside-cache');
  const link = path.join(root, 'linked');
  fs.symlinkSync(outside, link, 'junction');
  const report = inspectBuildCache({ root });
  assert.equal(report.complete, true);
  assert.equal(report.skipped_symbolic_links, 1);
  assert.equal(report.regular_file_paths, 0);
  assert.equal(report.path_logical_bytes, '0');
  assert.throws(() => inspectBuildCache({ root: link }), /symbolic link/i);
  assert.throws(() => inspectBuildCache({ root: path.join(link, 'nested') }), /symbolic link/i);
  assert.equal(fs.readFileSync(path.join(outside, 'untouched'), 'utf8'), 'outside-cache');
});

test('entry budget makes evidence incomplete and disables alert assessment', t => {
  const root = fixture(t);
  fs.writeFileSync(path.join(root, 'one'), '12345');
  fs.writeFileSync(path.join(root, 'two'), '12345');
  const report = inspectBuildCache({ root, maxEntries: 1, maxLogicalBytes: 1n });
  assert.equal(report.complete, false);
  assert.equal(report.stop_reason, 'entry_budget');
  assert.equal(report.entries_examined, 1);
  assert.equal(report.alert, null);
  assert.equal(inspectBuildCache({ root, maxEntries: 2 }).complete, true);
});

test('time budget fails closed using a deterministic clock without sleeping', t => {
  const root = fixture(t);
  let time = 0;
  const report = inspectBuildCache({ root, maxDurationMs: 1, clock: () => { time += 2; return time; } });
  assert.equal(report.complete, false);
  assert.equal(report.stop_reason, 'time_budget');
  assert.equal(report.alert, null);
});

test('the final read window is checked before reporting a complete scan', t => {
  const root = fixture(t);
  const samples = [0, 0, 2];
  const report = inspectBuildCache({ root, maxDurationMs: 1, clock: () => samples.shift() });
  assert.equal(report.complete, false);
  assert.equal(report.stop_reason, 'time_budget');
  assert.equal(report.entries_examined, 0);
  assert.equal(report.alert, null);
});

test('an injected metadata access failure returns incomplete evidence and preserves the file', t => {
  const root = fixture(t);
  const target = path.join(root, 'one');
  fs.writeFileSync(target, 'unchanged');
  const original = fs.lstatSync;
  let report;
  try {
    fs.lstatSync = (input, options) => {
      if (options?.bigint && path.resolve(input) === target) {
        const failure = new Error('injected metadata access denied');
        failure.code = 'EACCES';
        throw failure;
      }
      return original(input, options);
    };
    report = inspectBuildCache({ root });
  } finally { fs.lstatSync = original; }
  assert.equal(report.complete, false);
  assert.equal(report.stop_reason, 'metadata_error');
  assert.equal(report.errors[0].code, 'EACCES');
  assert.equal(report.alert, null);
  assert.equal(fs.readFileSync(target, 'utf8'), 'unchanged');
});

test('real CLI emits one JSON report and preserves contents and modification times', t => {
  const root = fixture(t);
  const target = path.join(root, 'one');
  fs.writeFileSync(target, 'abc');
  const before = fs.statSync(target);
  const report = reportOf(cli(['--root', root]), 0);
  assert.equal(report.path_logical_bytes, '3');
  assert.equal(report.complete, true);
  assert.equal(report.alert, false);
  assert.equal(fs.statSync(target).mtimeMs, before.mtimeMs);
  assert.equal(fs.readFileSync(target, 'utf8'), 'abc');
});

test('explicit logical ceiling allows equality and alerts only above it', t => {
  const root = fixture(t);
  fs.writeFileSync(path.join(root, 'one'), '12345');
  const equal = reportOf(cli(['--root', root, '--max-logical-bytes', '5']), 0);
  const exceeded = reportOf(cli(['--root', root, '--max-logical-bytes', '4']), 1);
  assert.equal(equal.alert, false);
  assert.equal(exceeded.alert, true);
  assert.equal(exceeded.complete, true);
});

test('real CLI never returns green for an incomplete traversal', t => {
  const root = fixture(t);
  fs.writeFileSync(path.join(root, 'one'), '12345');
  fs.writeFileSync(path.join(root, 'two'), '12345');
  const report = reportOf(cli(['--root', root, '--max-entries', '1', '--max-logical-bytes', '1']), 2);
  assert.equal(report.complete, false);
  assert.equal(report.stop_reason, 'entry_budget');
  assert.equal(report.alert, null);
});

test('missing, destructive, duplicate and invalid arguments refuse scanning', t => {
  const root = fixture(t);
  const attempts = [
    [], ['--delete', root], ['--root', root, '--output', 'anything'],
    ['--root', root, '--max-entries', '0'], ['--root', root, '--max-seconds', '-1'],
    ['--root', root, '--max-seconds', '1.5'], ['--root', root, '--max-logical-bytes', '-1'],
    ['--root', root, '--root', root], ['--root'],
  ];
  for (const args of attempts) {
    const report = reportOf(cli(args), 2);
    assert.equal(report.complete, false);
    assert.equal(report.stop_reason, 'invalid_input');
    assert.equal(report.entries_examined, 0);
  }
  assert.deepEqual(fs.readdirSync(root), []);
});

test('file, missing and filesystem-root inputs are refused', t => {
  const root = fixture(t);
  const file = path.join(root, 'file');
  fs.writeFileSync(file, 'unchanged');
  for (const target of [file, path.join(root, 'missing'), path.parse(root).root]) {
    const report = reportOf(cli(['--root', target]), 2);
    assert.equal(report.complete, false);
    assert.equal(report.entries_examined, 0);
  }
  assert.equal(fs.readFileSync(file, 'utf8'), 'unchanged');
});
