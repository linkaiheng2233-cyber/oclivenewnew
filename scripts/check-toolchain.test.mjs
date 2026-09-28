import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { inspectProjectToolchain } from './check-toolchain.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const child = (script, args = [], env = process.env) => spawnSync(process.execPath, [path.join(repoRoot, script), ...args], {
  cwd: repoRoot, env, encoding: 'utf8', windowsHide: true, timeout: 60000,
});
const withPath = value => ({
  ...Object.fromEntries(Object.entries(process.env).filter(([key]) => key.toUpperCase() !== 'PATH')),
  PATH: value,
});
function fixture(t) {
  const temporaryRoot = path.resolve(os.tmpdir());
  const prefix = 'oclive-toolchain-doctor-';
  const directory = fs.mkdtempSync(path.join(temporaryRoot, prefix));
  t.after(() => {
    const target = path.resolve(directory);
    assert.equal(path.dirname(target), temporaryRoot);
    assert.ok(path.basename(target).startsWith(prefix));
    fs.rmSync(target, { recursive: true, force: true });
  });
  return directory;
}

test('real ratchets preserve the frozen 3/1 and 75 matching-line baselines', () => {
  const domain = child('scripts/check-domain-layering.mjs');
  const host = child('scripts/check-host-reexport-imports.mjs');
  assert.equal(domain.status, 0, domain.stderr);
  assert.equal(host.status, 0, host.stderr);
  assert.match(domain.stdout, /use imports: 3 \(baseline max 3\)/);
  assert.match(domain.stdout, /FQ refs \(prod\): 1 \(baseline max 1\)/);
  assert.match(host.stdout, /re-export imports: 75 \(baseline max 75\)/);
});

test('both actual CLI gates reject missing rg with repair advice and no bare exception stack', () => {
  for (const script of ['scripts/check-domain-layering.mjs', 'scripts/check-host-reexport-imports.mjs']) {
    const result = child(script, [], withPath(''));
    assert.equal(result.status, 1);
    assert.match(result.stderr, /\[toolchain:MISSING\]/);
    assert.match(result.stderr, /rg --version/);
    assert.doesNotMatch(result.stderr, /node:internal|at (?:main|countDomain|countHost)/);
    assert.doesNotMatch(result.stdout, /ratchet ok/);
  }
});

test('real project doctor reports actual tool usability and executed gates separately', () => {
  const result = child('scripts/check-toolchain.mjs', ['--json']);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  const report = JSON.parse(result.stdout);
  assert.equal(report.scope, 'oclive-static-gates');
  assert.equal(report.ok, true);
  assert.deepEqual(report.tools.map(tool => [tool.id, tool.present, tool.usable, tool.gate_passed]), [
    ['node', true, true, true], ['rg', true, true, true], ['git', true, true, true], ['cargo', true, true, true],
  ]);
  assert.deepEqual(report.gates.map(gate => [gate.id, gate.status, gate.exit_code]), [
    ['domain-layering', 'PASS', 0], ['host-reexport', 'PASS', 0], ['git-worktree', 'PASS', 0], ['cargo-workspace', 'PASS', 0],
  ]);
});

test('missing tools block actual gates and NOT_RUN remains null rather than passing', () => {
  const result = child('scripts/check-toolchain.mjs', ['--json'], withPath(''));
  assert.equal(result.status, 1, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.ok, false);
  assert.equal(report.tools.find(tool => tool.id === 'node').usable, true);
  for (const tool of report.tools.filter(tool => tool.id !== 'node')) {
    assert.equal(tool.present, false);
    assert.equal(tool.usable, false);
    assert.equal(tool.gate_passed, null);
  }
  assert.ok(report.gates.every(gate => gate.status === 'NOT_RUN' && gate.exit_code === null));
});

test('doctor reports present but unusable for an existing invalid native rg executable', t => {
  const directory = fixture(t);
  fs.writeFileSync(path.join(directory, process.platform === 'win32' ? 'rg.exe' : 'rg'), '', { mode: 0o600 });
  const report = inspectProjectToolchain({ env: withPath(directory) });
  const rg = report.tools.find(tool => tool.id === 'rg');
  assert.equal(rg.present, true);
  assert.equal(rg.usable, false);
  assert.equal(rg.gate_passed, null);
  assert.equal(report.ok, false);
});

test('working tools do not make absent project gates pass', t => {
  const directory = fixture(t);
  fs.writeFileSync(path.join(directory, 'package.json'), JSON.stringify({ engines: { node: '>=22' } }));
  const report = inspectProjectToolchain({ repoRoot: directory });
  assert.ok(report.tools.every(tool => tool.present && tool.usable));
  assert.ok(report.tools.every(tool => tool.gate_passed === false));
  assert.ok(report.gates.every(gate => gate.status === 'FAIL'));
  assert.equal(report.ok, false);
});

test('doctor rejects unsupported CLI options instead of pretending they skip checks', () => {
  const result = child('scripts/check-toolchain.mjs', ['--skip']);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Usage:/);
});
