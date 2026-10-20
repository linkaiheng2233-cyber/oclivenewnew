import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { inspectCatalog, queryEntries, reviewDigest } from './ai-docs.mjs';

const script = fileURLToPath(new URL('./ai-docs.mjs', import.meta.url));
const git = (root, ...args) => execFileSync('git', args, {
  cwd: root, encoding: 'utf8', windowsHide: true,
  env: { ...process.env, GIT_OPTIONAL_LOCKS: '0', GIT_TERMINAL_PROMPT: '0' },
}).trim();
const write = (root, name, text) => {
  fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
  fs.writeFileSync(path.join(root, name), text, 'utf8');
};
const child = (root, ...args) => spawnSync(process.execPath, [script, ...args, '--root', root], {
  encoding: 'utf8', windowsHide: true, timeout: 30000,
});

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-ai-docs-'));
  t.after(() => {
    assert.equal(path.dirname(fs.realpathSync(root)), fs.realpathSync(os.tmpdir()));
    assert.ok(path.basename(root).startsWith('oclive-ai-docs-'));
    fs.rmSync(root, { recursive: true, force: true });
  });
  git(root, 'init', '-q');
  git(root, 'config', 'user.name', 'Fixture');
  git(root, 'config', 'user.email', 'fixture@example.invalid');
  git(root, 'config', 'core.autocrlf', 'false');
  git(root, 'config', 'commit.gpgsign', 'false');
  git(root, 'config', 'core.hooksPath', path.join(root, 'no-hooks'));
  write(root, 'handoff/source.md', '# Source\n\n<a id="current"></a>\n## Current\n\nA limited current promise.\n\n## History\n\nOld evidence.\n');
  git(root, 'add', 'handoff/source.md');
  const oid = git(root, 'rev-parse', ':handoff/source.md');
  const entry = { key: 'current', label: 'Find current scope', locator: { kind: 'explicit-anchor', value: 'current' } };
  const record = { id: 'source', path: 'handoff/source.md', purpose: 'Locate scope', tags: ['documentation'],
    source_ref: { scheme: 'git-blob', object_format: 'sha1', oid }, entries: [entry] };
  const catalog = { schema_version: 1, records: [record] };
  const save = () => write(root, 'handoff/catalog/records.json', `${JSON.stringify(catalog, null, 2)}\n`);
  const commit = () => {
    git(root, 'add', '.');
    git(root, 'commit', '-qm', 'fixture snapshot');
    return git(root, 'rev-parse', 'HEAD');
  };
  save();
  const sha = commit();
  return { root, record, entry, catalog, save, commit, sha };
}

test('a real CLI query locates the source; a missing task returns a usable fallback', t => {
  const f = fixture(t);
  const result = child(f.root, 'query', '--task', 'documentation', '--json');
  assert.equal(result.status, 0, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.equal(output.matches[0].path, 'handoff/source.md');
  assert.equal(output.matches[0].line, 3);
  assert.deepEqual(output.matches[0].issues, []);
  const missing = child(f.root, 'query', '--task', 'voice', '--json');
  assert.equal(missing.status, 0, missing.stderr);
  assert.equal(JSON.parse(missing.stdout).matches.length, 0);
  assert.ok(JSON.parse(missing.stdout).fallback.length > 0);
});

test('unstaged and staged changes remain distinct; frozen mode reads its own catalog and source', t => {
  const f = fixture(t);
  write(f.root, 'handoff/source.md', '# Changed\n');
  let report = inspectCatalog({ root: f.root });
  assert.equal(report.records[0].unstaged, true);
  assert.equal(report.records[0].staged, false);
  assert.ok(report.records[0].entries[0].issues.includes('locator_missing'));
  git(f.root, 'add', 'handoff/source.md');
  report = inspectCatalog({ root: f.root });
  assert.equal(report.records[0].unstaged, false);
  assert.equal(report.records[0].staged, true);
  assert.ok(report.records[0].issues.includes('source_version_mismatch'));
  write(f.root, 'handoff/catalog/records.json', '{broken current input');
  const frozen = inspectCatalog({ root: f.root, mode: 'commit', rev: f.sha });
  assert.deepEqual(frozen.records[0].issues, []);
  assert.equal(frozen.records[0].entries[0].line, 3);
});

test('a source or summary change invalidates its scoped AI review', t => {
  const f = fixture(t);
  f.entry.summary = 'A limited current promise.';
  f.entry.review = { source_oid: f.record.source_ref.oid, entry_sha256: reviewDigest(f.entry),
    reviewed_on: '2026-10-20', subject: 'ai', reviewer: 'controller', method: 'summary-to-source', independent: false };
  f.save(); f.commit();
  let row = queryEntries(inspectCatalog({ root: f.root }), { task: 'documentation' }).matches[0];
  assert.equal(row.summary, f.entry.summary);
  const dated = queryEntries(inspectCatalog({ root: f.root }), { now: Date.parse('2026-10-21T00:00:00Z') });
  assert.equal(dated.matches[0].review_age_days, 1);
  f.entry.summary = 'A broader promise that was never reviewed.';
  f.save();
  row = queryEntries(inspectCatalog({ root: f.root }), { task: 'documentation' }).matches[0];
  assert.equal(row.summary, null);
  assert.ok(row.issues.includes('summary_review_stale'));
  f.entry.summary = 'A limited current promise.';
  write(f.root, 'handoff/source.md', '# Source\n\n<a id="current"></a>\n## Current\n\nA revised promise.\n');
  git(f.root, 'add', 'handoff/source.md');
  f.record.source_ref.oid = git(f.root, 'rev-parse', ':handoff/source.md');
  f.save(); f.commit();
  row = queryEntries(inspectCatalog({ root: f.root }), { task: 'documentation' }).matches[0];
  assert.equal(row.summary, null);
  assert.ok(row.issues.includes('summary_review_stale'));
});

test('the reader does not change source bytes, the index or its catalog', t => {
  const f = fixture(t);
  const files = ['handoff/source.md', 'handoff/catalog/records.json', '.git/index'];
  const before = files.map(name => fs.readFileSync(path.join(f.root, name)));
  inspectCatalog({ root: f.root });
  const result = child(f.root, 'boot', '--task', 'documentation');
  assert.equal(result.status, 0, result.stderr);
  files.forEach((name, i) => assert.deepEqual(fs.readFileSync(path.join(f.root, name)), before[i], name));
});

test('CRLF text and LF text compare equally without changing either file or index', t => {
  const f = fixture(t);
  const source = fs.readFileSync(path.join(f.root, 'handoff/source.md'), 'utf8');
  const lf = inspectCatalog({ root: f.root });
  write(f.root, 'handoff/source.md', source.replaceAll('\n', '\r\n'));
  const before = fs.readFileSync(path.join(f.root, 'handoff/source.md'));
  const crlf = inspectCatalog({ root: f.root });
  assert.equal(crlf.records[0].unstaged, false);
  assert.equal(crlf.records[0].observed_oid, lf.records[0].observed_oid);
  assert.deepEqual(crlf.records[0].issues, []);
  assert.deepEqual(fs.readFileSync(path.join(f.root, 'handoff/source.md')), before);
});

test('headings and explicit anchors in fenced examples or comments are never navigation targets', t => {
  const f = fixture(t);
  write(f.root, 'handoff/source.md', [
    '# Source', '````markdown', '<a id="current"></a>', '## Current', '```', '````',
    'prefix <!--', '<a id="current"></a>', '## Current', '-->',
    '~~~', '<!-- <a id="current"></a>', '~~~',
    '<a id="current"></a>', '## Current', 'A limited current promise.',
  ].join('\n'));
  git(f.root, 'add', 'handoff/source.md');
  f.record.source_ref.oid = git(f.root, 'rev-parse', ':handoff/source.md');
  f.record.entries.push({ key: 'by-heading', label: 'Find by exact title path',
    locator: { kind: 'heading-path', value: ['Source', 'Current'] } });
  f.save(); f.commit();
  const rows = inspectCatalog({ root: f.root }).records[0].entries;
  assert.equal(rows[0].line, 14);
  assert.equal(rows[1].line, 15);
  assert.ok(rows.every(row => row.issues.length === 0));
  write(f.root, 'handoff/source.md', '<a id="current"></a>\n<a id="current"></a>\n');
  assert.ok(inspectCatalog({ root: f.root }).records[0].entries[0].issues.includes('locator_ambiguous'));
});

test('deleted or untracked sources, broken locators and unreviewed summaries cannot pass check', t => {
  const f = fixture(t);
  fs.unlinkSync(path.join(f.root, 'handoff/source.md'));
  let result = child(f.root, 'check', '--json');
  assert.equal(result.status, 1, result.stderr);
  assert.ok(JSON.parse(result.stdout).records[0].issues.includes('source_missing'));
  f.record.path = 'handoff/untracked.md';
  f.entry.summary = 'An unreviewed summary';
  write(f.root, f.record.path, '# Other\n');
  f.save();
  result = child(f.root, 'check', '--json');
  assert.equal(result.status, 1, result.stderr);
  const record = JSON.parse(result.stdout).records[0];
  assert.ok(record.issues.includes('source_untracked'));
  assert.ok(record.entries[0].issues.includes('summary_unreviewed'));
  assert.equal(record.entries[0].summary, null);
});

test('unsafe paths, malformed JSON, wrong types and encoding errors fail closed through the actual CLI', t => {
  const f = fixture(t);
  for (const bad of ['../outside.md', 'C:/secret.md', 'handoff/../secret.md', 'handoff\\source.md']) {
    f.record.path = bad; f.save();
    const result = child(f.root, 'check');
    assert.equal(result.status, 2, `${bad}: ${result.stderr}`);
    assert.doesNotMatch(result.stdout, /matched/);
  }
  f.record.path = 'handoff/source.md';
  f.record.source_ref.oid = 'not-an-oid'; f.save();
  assert.equal(child(f.root, 'check').status, 2);
  write(f.root, 'handoff/catalog/records.json', 'PRIVATE_FIXTURE_MARKER: malformed JSON');
  const invalid = child(f.root, 'check');
  assert.equal(invalid.status, 2);
  assert.doesNotMatch(invalid.stderr, /PRIVATE_FIXTURE_MARKER/);
  fs.writeFileSync(path.join(f.root, 'handoff/catalog/records.json'), Buffer.from([0xff]));
  assert.match(child(f.root, 'check').stderr, /invalid UTF-8/);
  f.record.source_ref.oid = git(f.root, 'rev-parse', 'HEAD:handoff/source.md');
  f.catalog.records.push({ ...f.record, id: 'duplicate-path' }); f.save();
  assert.equal(child(f.root, 'check').status, 2);
});

test('a junction or symbolic directory cannot redirect a working source, while old commits stay readable', t => {
  const f = fixture(t);
  write(f.root, 'outside/source.md', '# Not the registered source\n');
  fs.symlinkSync(path.join(f.root, 'outside'), path.join(f.root, 'handoff/linked'), process.platform === 'win32' ? 'junction' : 'dir');
  f.record.path = 'handoff/linked/source.md'; f.save();
  const result = child(f.root, 'check');
  assert.equal(result.status, 2, result.stderr);
  assert.match(result.stderr, /symbolic link or junction rejected/);
  assert.deepEqual(inspectCatalog({ root: f.root, mode: 'commit', rev: f.sha }).records[0].issues, []);
});

test('CLI modes, limits and native exits are explicit; repeated output is deterministic', t => {
  const f = fixture(t);
  for (const args of [[], ['--unknown'], ['check', '--json', '--json'], ['check', '--rev', f.sha],
    ['check', '--mode', 'commit'], ['check', '--mode', 'commit', '--rev', 'HEAD'], ['query', '--limit', '0']]) {
    const result = child(f.root, ...args);
    assert.equal(result.status, 2, JSON.stringify(args));
  }
  assert.equal(child(f.root, 'check', '--mode', 'commit', '--rev', f.sha).status, 0);
  assert.equal(child(f.root, 'query', '--json').stdout, child(f.root, 'query', '--json').stdout);
  const report = inspectCatalog({ root: f.root });
  report.records[0].entries.push({ ...report.records[0].entries[0], key: 'second' });
  const result = queryEntries(report, { limit: 1 });
  assert.equal(result.total, 2);
  assert.equal(result.truncated, true);
});
