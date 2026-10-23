import assert from 'node:assert/strict';
import test from 'node:test';
import { scanManifest } from './check-sdk-boundary.mjs';

test('allows an SDK crate in dev-dependencies', () => {
  const manifest = [
    '[package]',
    'name = "demo"',
    '',
    '[dependencies]',
    'oclive_kernel_types = "0.2"',
    '',
    '[dev-dependencies]',
    'oclive-sdk = { path = "../sdk" }',
  ].join('\n');
  assert.deepEqual(scanManifest(manifest), []);
});

test('rejects an SDK crate in runtime dependencies', () => {
  const manifest = '[dependencies]\noclive-sdk = "0.1"\n';
  assert.deepEqual(scanManifest(manifest), [{ name: 'oclive-sdk', section: 'dependencies' }]);
});

test('rejects an SDK build dependency and target-specific dependency', () => {
  const build = scanManifest('[build-dependencies]\noclive_kernel_sdk = { path = "sdk" }\n');
  assert.equal(build.length, 1);
  const target = scanManifest('[target.\'cfg(unix)\'.dependencies]\noclive-sdk = "0.1"\n');
  assert.equal(target.length, 1);
});

test('rejects the table form of a dependency', () => {
  const table = scanManifest('[dependencies.oclive-sdk]\npath = "../sdk"\n');
  assert.deepEqual(table, [{ name: 'oclive-sdk', section: 'dependencies' }]);
});

test('ignores unrelated crates and comments', () => {
  const manifest = [
    '# oclive-sdk = "0.1" (planned, must stay dev-only)',
    '[dependencies]',
    'tokio = "1"',
    'oclive_kernel_contracts = { path = "contracts" }',
  ].join('\n');
  assert.deepEqual(scanManifest(manifest), []);
});
