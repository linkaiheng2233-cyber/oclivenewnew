import assert from 'node:assert/strict';
import test from 'node:test';
import { applyHashes, applyIndexHashes, includeKind, selectorsOverlap, validateScheme } from './check-ci-schemes.mjs';

const ctx = {
  validators: new Set(['dimension5-acceptance', 'stale-paths']),
  overrides: [
    { id: 'control', selectors: [{ kind: 'exact', value: 'data/ci/impact-map.v1.json' }, { kind: 'prefix', value: 'kernel/crates/oclive_ci_plan' }] },
  ],
  hashOf: () => 'hash',
  exists: () => true,
};

const base = {
  id: 'demo',
  title: 'Demo',
  status: 'draft',
  includes: ['scripts/demo.mjs'],
  jobs: ['dimension5-acceptance'],
  derived_from: { impact_map: 'hash', catalog: 'hash', planner: 'hash' },
  author: 'ai',
  created_at: '2026-10-22T16:45:00+08:00',
  rationale: 'test',
};

test('a clean scheme has no problems', () => {
  assert.deepEqual(validateScheme(base, ctx, '`demo`'), []);
});

test('unknown jobs and missing index rows are rejected', () => {
  const problems = validateScheme({ ...base, jobs: ['nope'] }, ctx, '');
  assert.ok(problems.some(item => item.includes('unknown job nope')));
  assert.ok(problems.some(item => item.includes('INDEX.md')));
});

test('includes may not overlap a control-plane override', () => {
  const problems = validateScheme({ ...base, includes: ['data/ci/impact-map.v1.json'] }, ctx, '`demo`');
  assert.ok(problems.some(item => item.includes('risk override control')));
});

test('include shapes and stale derivation are detected', () => {
  assert.equal(includeKind('scripts/vendor/**').kind, 'dir');
  assert.equal(includeKind('scripts/*.mjs').kind, 'unsupported');
  assert.ok(selectorsOverlap(ctx.overrides[0], 'kernel/crates/oclive_ci_plan/**'));
  const problems = validateScheme({ ...base, derived_from: { impact_map: 'old', catalog: 'hash', planner: 'hash' } }, ctx, '`demo`');
  assert.ok(problems.some(item => item.includes('stale derived_from.impact_map')));
});

test('applyHashes replaces the three derived hashes', () => {
  const text = JSON.stringify({
    derived_from: {
      impact_map: '1'.repeat(40),
      catalog: '2'.repeat(40),
      planner: '3'.repeat(40),
    },
  }, null, 2);
  const out = applyHashes(text, { impact_map: 'a'.repeat(40), catalog: 'b'.repeat(40), planner: 'c'.repeat(40) });
  assert.ok(out.includes('a'.repeat(40)) && out.includes('b'.repeat(40)) && out.includes('c'.repeat(40)));
});

test('applyIndexHashes rewrites a scheme row', () => {
  const index = '| `docs` | active | x | y | ai | 2026-10-23 | `11111111` / `22222222` / `33333333` |\n';
  const out = applyIndexHashes(index, [{ id: 'docs' }], { impact_map: 'a'.repeat(40), catalog: 'b'.repeat(40), planner: 'c'.repeat(40) });
  assert.ok(out.includes('`aaaaaaaa` / `bbbbbbbb` / `cccccccc`'));
});
