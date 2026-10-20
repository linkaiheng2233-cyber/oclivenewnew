import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { discoverDocuments, inspectCatalog, queryEntries, reviewDigest } from './cli.mjs';

const script = fileURLToPath(new URL('./cli.mjs', import.meta.url));
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
    ['check', '--mode', 'commit'], ['check', '--mode', 'commit', '--rev', 'HEAD'], ['query', '--limit', '0'],
    ['check', '--scope', 'documents']]) {
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

test('document discovery finds unregistered body text and excludes archive and historical waves', t => {
  const f = fixture(t);
  write(f.root, 'creator-docs/plugin-and-architecture/RPC.md', '# Plugin guide\n\n## Transport\n\nThe searchable provider_marker is only source text.\n');
  write(f.root, 'handoff/archive/old.md', '# provider_marker old archive\n');
  write(f.root, 'handoff/debt-marathon/waves/old.md', '# provider_marker old wave\n');
  f.commit();
  const report = discoverDocuments({ root: f.root });
  const result = queryEntries(report, { query: 'provider_marker' });
  assert.equal(result.matches.length, 1);
  assert.equal(result.matches[0].path, 'creator-docs/plugin-and-architecture/RPC.md');
  assert.equal(result.matches[0].origin, 'discovered');
  assert.equal(result.matches[0].source_oid, git(f.root, 'rev-parse', ':creator-docs/plugin-and-architecture/RPC.md'));
  assert.equal(result.matches[0].observed_oid, result.matches[0].source_oid);
  assert.equal(result.matches[0].matched_line, 5);
  assert.match(result.matches[0].excerpt, /provider_marker/);
  assert.equal(result.matches[0].summary, null);
  assert.equal(result.matches[0].review, null);
  assert.equal(report.excluded_documents, 2);
  assert.ok(!JSON.stringify(result).includes('search_lines'));
  const actual = child(f.root, 'query', '--query', 'provider_marker', '--json');
  assert.equal(actual.status, 0, actual.stderr);
  assert.equal(JSON.parse(actual.stdout).matches[0].matched_line, 5);
});

test('count and exhaustive modes expose the window instead of hiding it', t => {
  const f = fixture(t);
  for (const name of ['a', 'b', 'c']) write(f.root, `human-docs/${name}.md`, `# ${name}\n\nmarker_beta\n`);
  f.commit();
  const report = discoverDocuments({ root: f.root });
  const limited = queryEntries(report, { query: 'marker_beta', select: 'documents', limit: 2 });
  assert.equal(limited.matches.length, 2);
  assert.equal(limited.truncated, true);
  assert.equal(limited.selected_total, 3);
  const counted = queryEntries(report, { query: 'marker_beta', select: 'documents', count: true });
  assert.deepEqual(counted.matches, []);
  assert.equal(counted.selected_total, 3);
  assert.equal(counted.total, 3);
  assert.ok(counted.all_estimate > 0);
  const exhaustive = queryEntries(report, { query: 'marker_beta', select: 'documents', all: true });
  assert.equal(exhaustive.matches.length, 3);
  assert.equal(exhaustive.truncated, false);
  assert.throws(() => queryEntries(report, { query: 'marker_beta', select: 'entries', all: true }), /--all requires/);
  assert.throws(() => queryEntries(report, { query: 'marker_beta', all: true, count: true }), /one of/);
  assert.equal(child(f.root, 'query', '--query', 'marker_beta', '--all', '--select', 'entries').status, 2);
});

test('discovery roots are configurable, missing catalog still queries and bad encoding is skipped', t => {
  const f = fixture(t);
  write(f.root, 'human-docs/note.md', '# Note\n\nroots_marker\n');
  f.commit();
  assert.equal(discoverDocuments({ root: f.root, roots: ['human-docs'] }).coverage, 1);
  assert.equal(discoverDocuments({ root: f.root, roots: ['handoff/source.md'] }).coverage, 1);

  const bad = Buffer.concat([Buffer.from('# Bad\n\n'), Buffer.from([0xff, 0xfe]), Buffer.from('\n'), Buffer.from('encoding_marker\n')]);
  fs.mkdirSync(path.join(f.root, 'human-docs'), { recursive: true });
  fs.writeFileSync(path.join(f.root, 'human-docs/bad.md'), bad);
  f.commit();
  const after = discoverDocuments({ root: f.root });
  const badRow = after.records.find(record => record.path === 'human-docs/bad.md');
  assert.ok(badRow, 'invalid document is still listed');
  assert.ok(badRow.issues.includes('source_encoding'));

  fs.rmSync(path.join(f.root, 'handoff/catalog/records.json'));
  const query = child(f.root, 'query', '--query', 'roots_marker');
  assert.equal(query.status, 0, query.stderr);
  assert.match(query.stdout, /human-docs\/note\.md/);
  const boot = child(f.root, 'boot', '--task', 'documentation');
  assert.equal(boot.status, 0, boot.stderr);
  const check = child(f.root, 'check');
  assert.equal(check.status, 2);
  assert.match(check.stderr, /catalog missing/);
  const badRoots = child(f.root, 'query', '--query', 'roots_marker', '--roots', '../etc');
  assert.equal(badRoots.status, 2);
  assert.match(badRoots.stderr, /invalid --roots/);
});

test('the CLI document window defaults to twenty documents', t => {
  const f = fixture(t);
  for (let index = 0; index < 12; index += 1) write(f.root, `human-docs/n${index}.md`, `# N${index}\n\nwindow_marker\n`);
  f.commit();
  const out = child(f.root, 'query', '--query', 'window_marker', '--json');
  assert.equal(out.status, 0, out.stderr);
  const parsed = JSON.parse(out.stdout);
  assert.equal(parsed.unit, 'documents');
  assert.equal(parsed.matches.length, 12);
});

test('the document window caps curated rows and drops mirrored English twins', () => {
  const record = (id, path, origin) => ({ id, path, purpose: '', tags: [], origin,
    issues: [], source_oid: 'x', observed_oid: 'x', staged: false, unstaged: false,
    entries: [{ key: 'k', label: 'L', line: 1, locator: { kind: 'heading-path', value: [] }, issues: [], summary: null, review: null }] });
  const records = [
    ...[1, 2, 3, 4, 5].map(n => record(`c${n}`, `handoff/c${n}.md`, 'curated')),
    record('d1', 'creator-docs/architecture/a.md', 'discovered'),
    record('d2', 'creator-docs/architecture-en/a.md', 'discovered'),
    record('d3', 'handoff/d3.md', 'discovered'),
  ];
  const report = { mode: 'worktree', commit: 'x', coverage: records.length, scope: '', records };
  const capped = queryEntries(report, { select: 'documents', limit: 4, query: 'L' });
  const cappedPaths = capped.matches.map(row => row.path);
  assert.equal(cappedPaths.filter(item => item.startsWith('handoff/c')).length, 3, 'curated is capped at three');
  assert.ok(cappedPaths.includes('creator-docs/architecture/a.md'), 'discovered fills the reserved slots');
  const all = queryEntries(report, { select: 'documents', all: true, query: 'L' });
  const allPaths = all.matches.map(row => row.path);
  assert.ok(!allPaths.includes('creator-docs/architecture-en/a.md'), 'mirror twin is dropped');
  assert.ok(allPaths.includes('handoff/c5.md'), 'capped curated rows stay enumerable');
});

test('document selection lists one entry per document while entries selection keeps every heading', t => {
  const f = fixture(t);
  write(f.root, 'human-docs/a.md', '# A\n\n## One\nmarker_alpha\n\n## Two\nmarker_alpha\n');
  write(f.root, 'human-docs/b.md', '# B\n\n## One\nmarker_alpha\n');
  f.commit();
  const report = discoverDocuments({ root: f.root });
  const docs = queryEntries(report, { query: 'marker_alpha', select: 'documents', limit: 20 });
  assert.equal(docs.matches.length, 2, 'one row per document');
  assert.equal(new Set(docs.matches.map(row => row.path)).size, 2);
  assert.equal(docs.unit, 'documents');
  assert.equal(docs.total, 3);
  assert.equal(docs.selected_total, 2);
  const entries = queryEntries(report, { query: 'marker_alpha', select: 'entries', limit: 20 });
  assert.equal(entries.matches.length, 3, 'one row per heading');
});

test('long source lines are quoted to 240 code points and marked when truncated', t => {
  const f = fixture(t);
  const longLine = `${'x'.repeat(240)} tail_marker_longline`;
  write(f.root, 'human-docs/long.md', `# Long\n\n${longLine}\n`);
  write(f.root, 'human-docs/short.md', '# Short\n\nshort_marker_line\n');
  f.commit();
  const report = discoverDocuments({ root: f.root });
  const longHit = queryEntries(report, { query: 'tail_marker_longline' }).matches[0];
  assert.equal([...longHit.excerpt].length, 241);
  assert.ok(longHit.excerpt.endsWith('…'), longHit.excerpt.slice(-8));
  assert.ok(!longHit.excerpt.includes('tail_marker_longline'));
  const shortHit = queryEntries(report, { query: 'short_marker_line' }).matches[0];
  assert.ok(!shortHit.excerpt.endsWith('…'));
});

test('discovery keeps dirty current material distinct from a frozen commit and leaves untracked notes out', t => {
  const f = fixture(t);
  write(f.root, 'human-docs/start.md', '# Start\n\nOld material.\n');
  const sha = f.commit();
  write(f.root, 'human-docs/start.md', '# Start\n\nNew material.\n');
  write(f.root, 'human-docs/untracked.md', '# Private local note\n');
  const current = queryEntries(discoverDocuments({ root: f.root }), { query: 'New material' });
  assert.equal(current.matches.length, 1);
  assert.ok(current.matches[0].issues.includes('unstaged_change'));
  const old = queryEntries(discoverDocuments({ root: f.root, mode: 'commit', rev: sha }), { query: 'Old material' });
  assert.equal(old.matches.length, 1);
  assert.deepEqual(old.matches[0].issues, []);
  assert.equal(queryEntries(discoverDocuments({ root: f.root }), { query: 'Private local note' }).matches.length, 0);
});

test('literal CLI options are searchable without being parsed as navigation options', t => {
  const f = fixture(t);
  write(f.root, 'creator-docs/cli/guide.md', '# CLI\n\nUse --memory-example off when preparing this source example.\n');
  f.commit();
  const result = child(f.root, 'query', '--query', '--memory-example', '--json');
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).matches[0].matched_line, 3);
});

test('an unavailable indexed blob fails batch discovery instead of silently omitting the document', t => {
  const f = fixture(t);
  write(f.root, 'human-docs/missing-object.md', '# Material that has no valid indexed object\n');
  git(f.root, 'update-index', '--add', '--cacheinfo', `100644,${'1'.repeat(40)},human-docs/missing-object.md`);
  assert.throws(() => discoverDocuments({ root: f.root }), /invalid or unavailable batch blob/);
  const result = child(f.root, 'query', '--query', 'Material');
  assert.equal(result.status, 2, result.stderr);
  assert.doesNotMatch(result.stdout, /navigation located/);
});
