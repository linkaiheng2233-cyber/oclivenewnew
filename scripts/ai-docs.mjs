#!/usr/bin/env node
/** Optional, read-only navigation for eight pilot documents; never a rule or acceptance authority. */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkDocumentBytes } from './check-doc-encoding.mjs';

const catalogPath = 'handoff/catalog/records.json';
const defaultRoot = fileURLToPath(new URL('../', import.meta.url));
const fallback = 'Read handoff/AI_READING_INDEX.md and handoff/README.md; missing coverage is not missing implementation or policy.';
const usage = 'Usage: node scripts/ai-docs.mjs <query|check|boot> [--task TAG] [--query TEXT] [--limit 1..20] [--mode worktree|commit] [--rev FULL_SHA] [--root PATH] [--json]';
const normalize = text => text.replace(/\r\n/g, '\n');
const nonempty = value => typeof value === 'string' && value.trim().length > 0;
const identifier = value => typeof value === 'string' && /^[a-z][a-z0-9-]*$/.test(value);
const hash = text => createHash('sha256').update(text).digest('hex');

function requireValue(ok, message) {
  if (!ok) throw new Error(message);
}

function keys(value, required, optional, label) {
  requireValue(value !== null && typeof value === 'object' && !Array.isArray(value), `${label}: expected object`);
  requireValue(required.every(key => Object.hasOwn(value, key)), `${label}: missing required field`);
  requireValue(Object.keys(value).every(key => [...required, ...optional].includes(key)), `${label}: unknown field`);
}

function relativePath(value) {
  requireValue(typeof value === 'string' && value.length > 0
    && !/[\\:\x00-\x1f\x7f]/u.test(value)
    && value.split('/').every(part => part && part !== '.' && part !== '..'), 'unsafe repository path');
  return value;
}

function textOf(bytes, label) {
  const problems = checkDocumentBytes(bytes);
  requireValue(problems.length === 0, `${label}: ${problems.join(', ')}`);
  const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  requireValue(!/[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f]/u.test(text), `${label}: terminal control character`);
  return normalize(text);
}

function workingFile(root, relative) {
  let target = root;
  const parts = relativePath(relative).split('/');
  for (let i = 0; i < parts.length; i += 1) {
    target = path.join(target, parts[i]);
    let stat;
    try { stat = fs.lstatSync(target); } catch (error) {
      if (error.code === 'ENOENT') return null;
      throw error;
    }
    requireValue(!stat.isSymbolicLink(), `${relative}: symbolic link or junction rejected`);
    requireValue(i === parts.length - 1 ? stat.isFile() : stat.isDirectory(), `${relative}: expected regular path`);
  }
  return fs.readFileSync(target);
}

function repository(root) {
  root = fs.realpathSync(root);
  function git(args) {
    const result = spawnSync('git', ['--no-pager', '--no-optional-locks', '--literal-pathspecs',
      '-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false', ...args], {
      cwd: root, windowsHide: true, timeout: 15000, maxBuffer: 16 * 1024 * 1024,
      env: { ...process.env, GIT_OPTIONAL_LOCKS: '0', GIT_TERMINAL_PROMPT: '0', GIT_NO_LAZY_FETCH: '1', GIT_NO_REPLACE_OBJECTS: '1', GIT_PAGER: 'cat' },
    });
    requireValue(!result.error && result.status === 0, `Git read failed (${args[0]})`);
    return result.stdout;
  }
  const top = git(['rev-parse', '--show-toplevel']).toString('utf8').trim();
  requireValue(fs.realpathSync(top) === root, '--root must be the repository checkout root');
  const format = git(['rev-parse', '--show-object-format']).toString('utf8').trim();
  requireValue(['sha1', 'sha256'].includes(format), 'unsupported Git object format');
  const oidPattern = new RegExp(`^[a-f0-9]{${format === 'sha1' ? 40 : 64}}$`);
  const head = git(['rev-parse', '--verify', 'HEAD']).toString('utf8').trim();
  const blobs = new Map();
  function blob(oid) {
    if (!blobs.has(oid)) blobs.set(oid, git(['cat-file', 'blob', oid]));
    return blobs.get(oid);
  }
  function treeFile(rev, relative) {
    relativePath(relative);
    const listing = git(['ls-tree', '-z', rev, '--', relative]).toString('utf8').split('\0').filter(Boolean);
    if (listing.length === 0) return null;
    requireValue(listing.length === 1, `${relative}: ambiguous tree path`);
    const match = /^(100644|100755) blob ([a-f0-9]+)\t(.*)$/u.exec(listing[0]);
    requireValue(match && match[3] === relative, `${relative}: non-regular Git source rejected`);
    return { oid: match[2], bytes: blob(match[2]) };
  }
  return { root, git, format, oidPattern, head, blob, treeFile };
}

function validateCatalog(catalog, repo) {
  keys(catalog, ['schema_version', 'records'], [], 'catalog');
  requireValue(catalog.schema_version === 1, 'unsupported catalog schema');
  requireValue(Array.isArray(catalog.records) && catalog.records.length > 0 && catalog.records.length <= 8,
    'pilot catalog must contain one to eight records');
  const ids = new Set();
  const paths = new Set();
  for (const record of catalog.records) {
    keys(record, ['id', 'path', 'purpose', 'tags', 'source_ref', 'entries'], ['superseded_by'], 'record');
    requireValue(identifier(record.id) && !ids.has(record.id), 'duplicate or invalid record id');
    ids.add(record.id);
    relativePath(record.path);
    requireValue(/^(?:(?:handoff|creator-docs(?:-en)?|human-docs(?:-en)?)\/.+\.md|AGENTS\.md|README\.md)$/u.test(record.path)
      && !paths.has(record.path.toLowerCase()), 'duplicate or unsupported document path');
    paths.add(record.path.toLowerCase());
    requireValue(nonempty(record.purpose), `${record.id}: missing purpose`);
    requireValue(Array.isArray(record.tags) && record.tags.length > 0 && record.tags.every(identifier)
      && new Set(record.tags).size === record.tags.length, `${record.id}: invalid tags`);
    keys(record.source_ref, ['scheme', 'object_format', 'oid'], [], 'source_ref');
    requireValue(record.source_ref.scheme === 'git-blob' && record.source_ref.object_format === repo.format
      && repo.oidPattern.test(record.source_ref.oid), `${record.id}: invalid Git source reference`);
    requireValue(Array.isArray(record.entries) && record.entries.length > 0, `${record.id}: entries required`);
    const entryIds = new Set();
    for (const entry of record.entries) {
      keys(entry, ['key', 'label', 'locator'], ['summary', 'review'], 'entry');
      requireValue(identifier(entry.key) && !entryIds.has(entry.key) && nonempty(entry.label), `${record.id}: invalid or duplicate entry`);
      entryIds.add(entry.key);
      keys(entry.locator, ['kind', 'value'], [], 'locator');
      requireValue((entry.locator.kind === 'explicit-anchor' && identifier(entry.locator.value))
        || (entry.locator.kind === 'heading-path' && Array.isArray(entry.locator.value)
          && entry.locator.value.length > 0 && entry.locator.value.every(nonempty)), `${record.id}: invalid locator`);
      requireValue(entry.summary === undefined || nonempty(entry.summary), `${record.id}: empty summary`);
      if (entry.review !== undefined && entry.review !== null) {
        requireValue(entry.summary !== undefined, `${record.id}: review without summary`);
        keys(entry.review, ['source_oid', 'entry_sha256', 'reviewed_on', 'subject', 'reviewer', 'method', 'independent'], [], 'review');
        const review = entry.review;
        requireValue(repo.oidPattern.test(review.source_oid) && /^[a-f0-9]{64}$/.test(review.entry_sha256), 'invalid review input reference');
        requireValue(/^\d{4}-\d{2}-\d{2}$/.test(review.reviewed_on)
          && !Number.isNaN(Date.parse(`${review.reviewed_on}T00:00:00Z`))
          && new Date(`${review.reviewed_on}T00:00:00Z`).toISOString().slice(0, 10) === review.reviewed_on, 'invalid review date');
        requireValue(['ai', 'human'].includes(review.subject) && nonempty(review.reviewer)
          && review.method === 'summary-to-source' && typeof review.independent === 'boolean', 'invalid scoped review');
      }
    }
    if (record.superseded_by !== undefined) {
      requireValue(Array.isArray(record.superseded_by) && record.superseded_by.every(identifier), 'invalid superseded_by');
    }
  }
  for (const record of catalog.records) {
    requireValue((record.superseded_by ?? []).every(id => ids.has(id) && id !== record.id), 'unknown superseded_by target');
  }
}

/** The digest binds this entry's identity, navigation text, locator and optional summary, not a semantic verdict. */
export function reviewDigest(entry) {
  return hash(JSON.stringify([entry.key, entry.label, entry.locator.kind, entry.locator.value, entry.summary ?? null]));
}

function locate(text, locator) {
  const headings = [];
  const anchors = [];
  const stack = [];
  let fence = null;
  let comment = false;
  for (const [index, line] of text.split('\n').entries()) {
    if (comment) { if (line.includes('-->')) comment = false; continue; }
    if (fence) {
      if (new RegExp(`^ {0,3}${fence.char}{${fence.length},}[ \\t]*$`).test(line)) fence = null;
      continue;
    }
    const opening = /^ {0,3}(`{3,}|~{3,})/u.exec(line);
    if (opening) { fence = { char: opening[1][0], length: opening[1].length }; continue; }
    if (line.includes('<!--')) { comment = !line.includes('-->', line.indexOf('<!--') + 4); continue; }
    const anchor = /^ {0,3}<a\s+id=(["'])([^"']+)\1\s*>\s*<\/a>\s*$/u.exec(line);
    if (anchor) anchors.push({ value: anchor[2], line: index + 1 });
    const heading = /^ {0,3}(#{1,6})\s+(.+?)\s*$/u.exec(line);
    if (heading) {
      const level = heading[1].length;
      while (stack.length && stack.at(-1).level >= level) stack.pop();
      stack.push({ level, text: heading[2].replace(/\s+#+\s*$/u, '') });
      headings.push({ value: stack.map(item => item.text), line: index + 1 });
    }
  }
  const matches = (locator.kind === 'explicit-anchor' ? anchors : headings)
    .filter(item => JSON.stringify(item.value) === JSON.stringify(locator.value));
  return { line: matches.length === 1 ? matches[0].line : null,
    issues: matches.length === 0 ? ['locator_missing'] : matches.length > 1 ? ['locator_ambiguous'] : [] };
}

/** Read one checkout or one complete commit; never mix current records with historical sources. */
export function inspectCatalog({ root = defaultRoot, mode = 'worktree', rev } = {}) {
  requireValue(['worktree', 'commit'].includes(mode), 'invalid mode');
  const repo = repository(root);
  requireValue(mode === 'commit' ? typeof rev === 'string' && repo.oidPattern.test(rev) : rev === undefined,
    'commit mode requires a full commit SHA; worktree mode rejects --rev');
  let commit = repo.head;
  if (mode === 'commit') {
    commit = repo.git(['rev-parse', '--verify', `${rev}^{commit}`]).toString('utf8').trim();
    requireValue(commit === rev, 'revision must be a commit, not a tag or tree');
  }
  const bytes = mode === 'commit' ? repo.treeFile(commit, catalogPath)?.bytes : workingFile(repo.root, catalogPath);
  requireValue(bytes, `${catalogPath}: catalog missing; use the existing reading index`);
  const catalogText = textOf(bytes, catalogPath);
  let catalog;
  try { catalog = JSON.parse(catalogText); }
  catch { throw new Error(`${catalogPath}: invalid JSON`); }
  validateCatalog(catalog, repo);
  const index = new Map();
  if (mode === 'worktree') {
    for (const row of repo.git(['ls-files', '--stage', '-z', '--', ...catalog.records.map(record => record.path)]).toString('utf8').split('\0').filter(Boolean)) {
      const match = /^(\d+) ([a-f0-9]+) ([0-3])\t(.*)$/u.exec(row);
      requireValue(match, 'invalid Git index listing');
      const entries = index.get(match[4]) ?? [];
      entries.push({ mode: match[1], oid: match[2], stage: Number(match[3]) });
      index.set(match[4], entries);
    }
  }
  const records = catalog.records.map(record => {
    const headFile = repo.treeFile(commit, record.path);
    const issues = [];
    let staged = false;
    let unstaged = false;
    let source = headFile?.bytes ?? null;
    let oid = headFile?.oid ?? null;
    if (mode === 'worktree') {
      source = workingFile(repo.root, record.path);
      const entries = index.get(record.path) ?? [];
      const indexed = entries.length === 1 && entries[0].stage === 0 ? entries[0] : null;
      if (entries.some(item => item.stage !== 0)) issues.push('unmerged_index');
      if (indexed) requireValue(['100644', '100755'].includes(indexed.mode), `${record.path}: non-regular index entry rejected`);
      oid = indexed?.oid ?? null;
      staged = (headFile?.oid ?? null) !== oid;
      unstaged = source !== null && indexed !== null
        ? textOf(source, record.path) !== textOf(repo.blob(indexed.oid), record.path)
        : source !== null || indexed !== null;
      if (!indexed && source) issues.push('source_untracked');
      if (staged) issues.push('staged_change');
      if (unstaged) issues.push('unstaged_change');
    }
    if (source === null) issues.push('source_missing');
    if (oid !== record.source_ref.oid) issues.push('source_version_mismatch');
    const text = source === null ? '' : textOf(source, record.path);
    const entries = record.entries.map(entry => {
      const location = locate(text, entry.locator);
      const entryIssues = [...location.issues];
      let summary = null;
      if (entry.summary !== undefined) {
        if (!entry.review) entryIssues.push('summary_unreviewed');
        else if (entry.review.source_oid !== record.source_ref.oid
          || entry.review.entry_sha256 !== reviewDigest(entry)) entryIssues.push('summary_review_stale');
        else if (issues.length === 0 && entryIssues.length === 0) summary = entry.summary;
      }
      return { key: entry.key, label: entry.label, line: location.line, locator: entry.locator,
        summary, review: entry.review ?? null, issues: entryIssues };
    });
    return { id: record.id, path: record.path, purpose: record.purpose, tags: record.tags,
      source_oid: record.source_ref.oid, observed_oid: oid, staged, unstaged, issues, entries };
  });
  return { mode, commit, coverage: records.length, scope: 'pilot-only; no whole-repository coverage or semantic acceptance',
    catalog_sha256: hash(bytes), fallback, records };
}

export function queryEntries(report, { task = '', query = '', limit = 8, now = Date.now() } = {}) {
  requireValue(Number.isInteger(limit) && limit >= 1 && limit <= 20, 'invalid result limit');
  requireValue(Number.isFinite(now), 'invalid observation clock');
  const rows = report.records.flatMap(record => record.entries.map(entry => ({
    id: record.id, path: record.path, purpose: record.purpose, tags: record.tags, ...entry,
    review_age_days: entry.review ? Math.floor((now - Date.parse(`${entry.review.reviewed_on}T00:00:00Z`)) / 86400000) : null,
    issues: [...record.issues, ...entry.issues],
  }))).filter(row => (!task || row.tags.includes(task))
    && (!query || `${row.id} ${row.key} ${row.path} ${row.purpose} ${row.tags.join(' ')} ${row.label}`.toLowerCase().includes(query.toLowerCase())));
  rows.sort((a, b) => `${a.id}/${a.key}` < `${b.id}/${b.key}` ? -1 : `${a.id}/${a.key}` > `${b.id}/${b.key}` ? 1 : 0);
  return { mode: report.mode, commit: report.commit, coverage: report.coverage, scope: report.scope,
    total: rows.length, truncated: rows.length > limit, matches: rows.slice(0, limit), fallback };
}

function parseArgs(args) {
  requireValue(['query', 'check', 'boot'].includes(args[0]), usage);
  const options = { command: args[0] };
  const allowed = new Set(['--root', '--task', '--query', '--limit', '--mode', '--rev']);
  for (let i = 1; i < args.length; i += 1) {
    const flag = args[i];
    const key = flag.slice(2);
    requireValue(!Object.hasOwn(options, key), `duplicate option: ${flag}`);
    if (flag === '--json') options.json = true;
    else {
      requireValue(allowed.has(flag) && nonempty(args[i + 1]) && !args[i + 1].startsWith('--'), usage);
      options[key] = args[++i];
    }
  }
  if (options.command === 'check') requireValue(!options.task && !options.query && !options.limit, usage);
  if (options.limit !== undefined) {
    requireValue(/^(?:[1-9]|1[0-9]|20)$/.test(options.limit), usage);
    options.limit = Number(options.limit);
  }
  return options;
}

function render(output, command) {
  const safe = value => JSON.stringify(value);
  const lines = [`AI docs (${command}): ${output.mode} @ ${output.commit}`,
    `Coverage: ${output.coverage} pilot documents; version matching is not semantic or runtime acceptance.`];
  if (command === 'check') {
    for (const record of output.records) {
      const issues = [...record.issues, ...record.entries.flatMap(entry => entry.issues.map(issue => `${entry.key}:${issue}`))];
      lines.push(`${safe(record.path)}: ${issues.length ? issues.join(', ') : 'reference and locator matched'}`);
    }
  } else {
    if (command === 'boot') lines.push('Rules stay in AGENTS.md and linked SSOT; open the relevant original before deciding a change.');
    for (const row of output.matches) {
      lines.push(`${safe(row.label)} -> ${safe(row.path)}:${row.line ?? '?'} [${row.issues.join(', ') || 'navigation matched'}]`);
      if (row.summary !== null) lines.push(`  Scoped summary (not acceptance): ${safe(row.summary)}`);
      if (row.review) lines.push(`  Review: ${safe(row.review.subject)} / ${safe(row.review.reviewer)}; independent=${row.review.independent}; date=${row.review.reviewed_on}; age_hint=${row.review_age_days}d (not validity)`);
    }
    if (output.truncated) lines.push(`Showing ${output.matches.length}/${output.total}; narrow --task or --query, or use --limit up to 20.`);
  }
  lines.push(`Fallback: ${output.fallback}`);
  return lines.join('\n');
}

export function main(args) {
  const options = parseArgs(args);
  const report = inspectCatalog(options);
  const output = options.command === 'check' ? report : queryEntries(report, options);
  console.log(options.json ? JSON.stringify(output, null, 2) : render(output, options.command));
  return options.command === 'check'
    ? (report.records.some(record => record.issues.length || record.entries.some(entry => entry.issues.length)) ? 1 : 0)
    : (output.matches.some(row => row.issues.length) ? 1 : 0);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error(`ai-docs: ${error.message}`); process.exitCode = 2; }
}
