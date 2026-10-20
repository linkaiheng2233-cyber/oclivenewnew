#!/usr/bin/env node
/** Optional, read-only document navigation; never a rule or acceptance authority. */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const bytesDecoder = new TextDecoder('utf-8', { fatal: true });
function checkDocumentBytes(bytes) {
  const problems = [];
  if (bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) problems.push('UTF-8 BOM');
  let content;
  try { content = bytesDecoder.decode(bytes); } catch { problems.push('invalid UTF-8'); return problems; }
  if (content.includes('\ufffd')) problems.push('replacement character U+FFFD');
  if (/\?{3,}/u.test(content)) problems.push('three or more consecutive question marks');
  return problems;
}

const catalogCandidates = ['handoff/catalog/records.json', 'catalog/records.json'];
const defaultRoot = fileURLToPath(new URL('./', import.meta.url));
const fallback = 'Read the repository reading index; missing coverage is not missing implementation or policy.';
const usage = 'Usage: stackmap <query|check|boot> [--scope catalog|documents] [--select documents|entries] [--task TAG] [--query TEXT] [--limit 1..20] [--all] [--count] [--catalog PATH] [--mode worktree|commit] [--rev FULL_SHA] [--root PATH] [--roots a,b] [--json]';
const normalize = text => text.replace(/\r\n/g, '\n');
const nonempty = value => typeof value === 'string' && value.trim().length > 0;
const identifier = value => typeof value === 'string' && /^[a-z][a-z0-9-]*$/.test(value);
const hash = text => createHash('sha256').update(text).digest('hex');
const compare = (a, b) => a < b ? -1 : a > b ? 1 : 0;
/** Curated records lead the document window, but must not consume it: three slots are reserved. */
const curatedDocumentCap = 3;
const excerptMax = 240;
/** Quote at most `excerptMax` code points; mark truncation so a cut sentence cannot read as complete. */
const excerptOf = text => {
  const chars = [...text.trim()];
  return chars.slice(0, excerptMax).join('') + (chars.length > excerptMax ? '…' : '');
};
/** Rough output size of listing these rows, so a caller can price an exhaustive read before asking for it. */
const estimateRows = rows => {
  let cjk = 0, other = 0;
  for (const row of rows) {
    for (const ch of `${row.label} ${row.path}`) {
      if (ch >= '\u3400' && ch <= '\u9fff') cjk += 1; else other += 1;
    }
    other += 40;
  }
  return Math.round(cjk + other / 3.5);
};

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
  function git(args, input) {
    const result = spawnSync('git', ['--no-pager', '--no-optional-locks', '--literal-pathspecs',
      '-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false', ...args], {
      cwd: root, input, windowsHide: true, timeout: 15000, maxBuffer: 16 * 1024 * 1024,
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
  function prefetch(oids) {
    const unique = [...new Set(oids)].filter(oid => !blobs.has(oid));
    if (unique.length === 0) return;
    requireValue(unique.every(oid => oidPattern.test(oid)), 'invalid batch object id');
    const bytes = git(['cat-file', '--batch'], `${unique.join('\n')}\n`);
    let offset = 0;
    for (const oid of unique) {
      const newline = bytes.indexOf(10, offset);
      const header = bytes.subarray(offset, newline).toString('ascii');
      const match = /^([a-f0-9]+) blob (\d+)$/u.exec(header);
      requireValue(newline >= offset && match && match[1] === oid, 'invalid or unavailable batch blob');
      const end = newline + 1 + Number(match[2]);
      requireValue(Number.isSafeInteger(end) && bytes[end] === 10, 'incomplete batch blob');
      blobs.set(oid, bytes.subarray(newline + 1, end));
      offset = end + 1;
    }
    requireValue(offset === bytes.length, 'unexpected trailing batch output');
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
  return { root, git, format, oidPattern, head, blob, prefetch, treeFile };
}

function validateCatalog(catalog, repo) {
  keys(catalog, ['schema_version', 'records'], ['roots'], 'catalog');
  requireValue(catalog.schema_version === 1, 'unsupported catalog schema');
  requireValue(Array.isArray(catalog.records) && catalog.records.length > 0, 'catalog records required');
  if (catalog.roots !== undefined) {
    requireValue(Array.isArray(catalog.roots) && catalog.roots.length > 0
      && catalog.roots.every(item => item === '.' || (nonempty(item) && !/[\\:\x00-\x1f\x7f]/u.test(item))), 'invalid catalog roots');
  }
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

function scanMarkdown(text) {
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
  return { anchors, headings };
}

function locate(text, locator) {
  const { anchors, headings } = scanMarkdown(text);
  const matches = (locator.kind === 'explicit-anchor' ? anchors : headings)
    .filter(item => JSON.stringify(item.value) === JSON.stringify(locator.value));
  return { line: matches.length === 1 ? matches[0].line : null,
    issues: matches.length === 0 ? ['locator_missing'] : matches.length > 1 ? ['locator_ambiguous'] : [] };
}

/** Read one checkout or one complete commit; never mix current records with historical sources. */
export function inspectCatalog({ root = defaultRoot, mode = 'worktree', rev, catalog } = {}) {
  requireValue(['worktree', 'commit'].includes(mode), 'invalid mode');
  const repo = repository(root);
  requireValue(mode === 'commit' ? typeof rev === 'string' && repo.oidPattern.test(rev) : rev === undefined,
    'commit mode requires a full commit SHA; worktree mode rejects --rev');
  let commit = repo.head;
  if (mode === 'commit') {
    commit = repo.git(['rev-parse', '--verify', `${rev}^{commit}`]).toString('utf8').trim();
    requireValue(commit === rev, 'revision must be a commit, not a tag or tree');
  }
  const candidates = catalog ? [catalog] : catalogCandidates;
  let catalogPath = null;
  let bytes = null;
  for (const candidate of candidates) {
    relativePath(candidate);
    const found = mode === 'commit' ? repo.treeFile(commit, candidate)?.bytes : workingFile(repo.root, candidate);
    if (found) { catalogPath = candidate; bytes = found; break; }
  }
  requireValue(bytes, `${candidates.join(' or ')}: catalog missing; use the existing reading index`);
  const catalogText = textOf(bytes, catalogPath);
  let parsedCatalog;
  try { parsedCatalog = JSON.parse(catalogText); }
  catch { throw new Error(`${catalogPath}: invalid JSON`); }
  validateCatalog(parsedCatalog, repo);
  const index = new Map();
  if (mode === 'worktree') {
    for (const row of repo.git(['ls-files', '--stage', '-z', '--', ...parsedCatalog.records.map(record => record.path)]).toString('utf8').split('\0').filter(Boolean)) {
      const match = /^(\d+) ([a-f0-9]+) ([0-3])\t(.*)$/u.exec(row);
      requireValue(match, 'invalid Git index listing');
      const entries = index.get(match[4]) ?? [];
      entries.push({ mode: match[1], oid: match[2], stage: Number(match[3]) });
      index.set(match[4], entries);
    }
  }
  const records = parsedCatalog.records.map(record => {
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
  return { mode, commit, coverage: records.length, scope: 'curated catalog; no semantic or runtime acceptance',
    roots: parsedCatalog.roots ?? null, catalog_path: catalogPath, catalog_sha256: hash(bytes), fallback, records };
}

/** Derive navigation at read time. No generated card, summary or review is persisted. */
export function discoverDocuments({ root = defaultRoot, mode = 'worktree', rev, roots = ['.'] } = {}) {
  requireValue(['worktree', 'commit'].includes(mode), 'invalid mode');
  const repo = repository(root);
  requireValue(mode === 'commit' ? typeof rev === 'string' && repo.oidPattern.test(rev) : rev === undefined,
    'commit mode requires a full commit SHA; worktree mode rejects --rev');
  const commit = mode === 'commit' ? repo.git(['rev-parse', '--verify', `${rev}^{commit}`]).toString('utf8').trim() : repo.head;
  requireValue(mode !== 'commit' || commit === rev, 'revision must be a commit');
  requireValue(Array.isArray(roots) && roots.length > 0
    && roots.every(item => item === '.' || (nonempty(item) && !/[\\:\x00-\x1f\x7f]/u.test(item))), 'invalid discovery roots');
  const head = new Map();
  for (const row of repo.git(['ls-tree', '-r', '-z', commit, '--', ...roots]).toString('utf8').split('\0').filter(Boolean)) {
    const match = /^(\d+) (?:blob|commit) ([a-f0-9]+)\t(.*)$/u.exec(row);
    requireValue(match, 'invalid Git tree listing');
    if (match[3].endsWith('.md')) head.set(match[3], { mode: match[1], oid: match[2] });
  }
  const index = new Map();
  if (mode === 'worktree') {
    for (const row of repo.git(['ls-files', '--stage', '-z', '--', ...roots]).toString('utf8').split('\0').filter(Boolean)) {
      const match = /^(\d+) ([a-f0-9]+) ([0-3])\t(.*)$/u.exec(row);
      requireValue(match, 'invalid Git index listing');
      if (!match[4].endsWith('.md')) continue;
      const values = index.get(match[4]) ?? [];
      values.push({ mode: match[1], oid: match[2], stage: Number(match[3]) });
      index.set(match[4], values);
    }
  }
  const listed = [...new Set([...head.keys(), ...index.keys()])].sort();
  const selected = listed.filter(name => !name.split('/').includes('archive')
    && !name.startsWith('handoff/debt-marathon/waves/'));
  const sources = selected.map(name => {
    relativePath(name);
    const entries = index.get(name) ?? [];
    const file = mode === 'commit' ? head.get(name) : entries.find(entry => entry.stage === 0);
    if (file) requireValue(['100644', '100755'].includes(file.mode), `${name}: non-regular document rejected`);
    return { name, entries, file };
  });
  repo.prefetch(sources.flatMap(source => source.file ? [source.file.oid] : []));
  const records = sources.map(({ name, entries, file }) => {
    const bytes = mode === 'commit' ? (file ? repo.blob(file.oid) : null) : workingFile(repo.root, name);
    const issues = [];
    let text = '';
    let indexedText = null;
    if (bytes !== null) {
      try { text = textOf(bytes, name); }
      catch { issues.push('source_encoding'); }
    }
    if (file && !issues.includes('source_encoding')) {
      try { indexedText = textOf(repo.blob(file.oid), name); }
      catch { indexedText = null; }
    }
    const staged = mode === 'worktree' && (head.get(name)?.oid ?? null) !== (file?.oid ?? null);
    const unstaged = mode === 'worktree'
      && (file ? bytes === null || indexedText === null || text !== indexedText : bytes !== null);
    if (entries.some(entry => entry.stage !== 0)) issues.push('unmerged_index');
    if (bytes === null) issues.push('source_missing');
    if (staged) issues.push('staged_change');
    if (unstaged) issues.push('unstaged_change');
    const lines = text.split('\n');
    const headings = scanMarkdown(text).headings;
    if (headings.length === 0 || headings[0].line > 1) headings.unshift({ line: 1, value: [name] });
    return { id: `document:${name}`, path: name, purpose: headings[0].value.at(-1),
      tags: [...new Set([name.split('/')[0], ...name.toLowerCase().split(/[^a-z0-9]+/u).filter(identifier)])],
      origin: 'discovered', source_oid: file?.oid ?? null, staged, unstaged, issues,
      entries: headings.map((heading, i) => ({ key: `line-${heading.line}`, label: heading.value.join(' / '),
        line: heading.line, locator: { kind: 'observed-heading', value: heading.value }, summary: null, review: null, issues: [],
        search_lines: lines.slice(heading.line - 1, (headings[i + 1]?.line ?? lines.length + 1) - 1)
          .map((line, j) => ({ line: heading.line + j, text: line })),
      })),
    };
  });
  return { mode, commit, coverage: records.length, excluded_documents: listed.length - selected.length,
    scope: `tracked Markdown under ${roots.join(', ')}; archive and historical waves excluded; document status is not inferred`,
    fallback, records };
}

export function queryEntries(report, { task = '', query = '', limit = 8, select = 'entries', all = false, count = false, now = Date.now() } = {}) {
  requireValue(Number.isInteger(limit) && limit >= 1 && limit <= 20, 'invalid result limit');
  requireValue(['documents', 'entries'].includes(select), 'invalid selection unit');
  requireValue(!(all && count), 'use one of --all or --count');
  requireValue(!all || select === 'documents', '--all requires --select documents');
  requireValue(Number.isFinite(now), 'invalid observation clock');
  const rows = report.records.flatMap(record => record.entries.flatMap(entry => {
    if (task && !record.tags.includes(task)) return [];
    const haystack = `${record.id} ${entry.key} ${record.path} ${record.purpose} ${record.tags.join(' ')} ${entry.label}`;
    const match = query ? entry.search_lines?.find(line => line.text.toLowerCase().includes(query.toLowerCase())) : null;
    if (query && !haystack.toLowerCase().includes(query.toLowerCase()) && !match) return [];
    const { search_lines, ...navigation } = entry;
    return [{ id: record.id, path: record.path, purpose: record.purpose, tags: record.tags, ...navigation,
      origin: record.origin ?? 'curated', reference: record.origin === 'discovered' ? 'read-time-only; not pre-reviewed' : 'recorded-source-reference',
      source_oid: record.source_oid,
      observed_oid: record.origin === 'discovered' ? record.source_oid : record.observed_oid,
      staged: record.staged, unstaged: record.unstaged,
      matched_line: match?.line ?? null, excerpt: match ? excerptOf(match.text) : null,
      review_age_days: entry.review ? Math.floor((now - Date.parse(`${entry.review.reviewed_on}T00:00:00Z`)) / 86400000) : null,
      issues: [...record.issues, ...entry.issues],
    }];
  }));
  rows.sort((a, b) => (a.origin === 'curated' ? 0 : 1) - (b.origin === 'curated' ? 0 : 1)
    || compare(a.path, b.path) || a.line - b.line || compare(a.key, b.key));
  const documents = rows.filter((row, index) => rows.findIndex(other => other.path === row.path) === index);
  const isMirrored = row => {
    const match = /(^|\/)([^/]+)-en\//.exec(row.path);
    return match !== null && documents.some(other => other.path === row.path.replace(`${match[2]}-en/`, `${match[2]}/`));
  };
  const kept = documents.filter(row => !isMirrored(row));
  const curated = kept.filter(row => row.origin === 'curated');
  const selected = select === 'entries' ? rows
    : [...curated.slice(0, curatedDocumentCap), ...kept.filter(row => row.origin !== 'curated'), ...curated.slice(curatedDocumentCap)];
  const listed = count ? [] : (all ? selected : selected.slice(0, limit));
  return { mode: report.mode, commit: report.commit, coverage: report.coverage, scope: report.scope,
    excluded_documents: report.excluded_documents ?? 0, unit: select,
    total: rows.length, selected_total: selected.length,
    all_estimate: estimateRows(selected), count_mode: count,
    truncated: !count && !all && selected.length > limit, matches: listed, fallback };
}

function parseArgs(args) {
  requireValue(['query', 'check', 'boot'].includes(args[0]), usage);
  const options = { command: args[0] };
  const allowed = new Set(['--root', '--task', '--query', '--limit', '--mode', '--rev', '--scope', '--select', '--roots', '--catalog']);
  for (let i = 1; i < args.length; i += 1) {
    const flag = args[i];
    const key = flag.slice(2);
    requireValue(!Object.hasOwn(options, key), `duplicate option: ${flag}`);
    if (flag === '--json') options.json = true;
    else if (flag === '--all' || flag === '--count') options[key] = true;
    else {
      requireValue(allowed.has(flag) && nonempty(args[i + 1])
        && (flag === '--query' || !args[i + 1].startsWith('--')), usage);
      options[key] = args[++i];
    }
  }
  if (options.command === 'check') requireValue(!options.task && !options.query && !options.limit && !options.select && !options.all && !options.count && !options.roots, usage);
  options.scope ??= options.command === 'query' ? 'documents' : 'catalog';
  options.select ??= 'documents';
  if (options.roots !== undefined) {
    options.roots = options.roots.split(',').map(item => item.trim()).filter(Boolean);
    requireValue(options.roots.length > 0
      && options.roots.every(item => item === '.' || item.split('/').every(part => part !== '' && part !== '.' && part !== '..')),
    'invalid --roots path: use repository-relative paths without . or ..');
  }
  requireValue(['documents', 'entries'].includes(options.select), usage);
  requireValue(!(options.all && options.count), 'use one of --all or --count');
  requireValue(!options.all || options.select === 'documents', '--all requires --select documents');
  requireValue(['catalog', 'documents'].includes(options.scope)
    && (options.command !== 'check' || options.scope === 'catalog'), usage);
  if (options.limit !== undefined) {
    requireValue(/^(?:[1-9]|1[0-9]|20)$/.test(options.limit), usage);
    options.limit = Number(options.limit);
  } else if (options.command !== 'check') {
    options.limit = options.select === 'documents' ? 20 : 8;
  }
  return options;
}

function render(output, command) {
  const safe = value => JSON.stringify(value);
  const lines = [`stackmap (${command}): ${output.mode} @ ${output.commit}`,
    `Coverage: ${output.coverage} documents; version matching is not semantic or runtime acceptance.`, `Scope: ${output.scope}`];
  if (output.excluded_documents) lines.push(`Excluded from discovery: ${output.excluded_documents} archive / historical wave documents.`);
  if (command === 'check') {
    for (const record of output.records) {
      const issues = [...record.issues, ...record.entries.flatMap(entry => entry.issues.map(issue => `${entry.key}:${issue}`))];
      lines.push(`${safe(record.path)}: ${issues.length ? issues.join(', ') : 'reference and locator matched'}`);
    }
  } else {
    if (command === 'boot') lines.push('Rules stay in AGENTS.md and linked SSOT; open the relevant original before deciding a change.');
    const byDocument = output.unit === 'documents';
    if (output.count_mode) {
      lines.push(`${output.selected_total} ${output.unit} match from ${output.total} entries; use --all to list them (~${output.all_estimate} tokens).`);
    } else {
      for (const row of output.matches) {
        lines.push(`${safe(row.label)} -> ${safe(row.path)}:${row.line ?? '?'} [${row.origin}; ${row.issues.join(', ') || 'navigation located'}]`);
        if (!byDocument && row.excerpt !== null) lines.push(`  Source line ${row.matched_line} (unreviewed quote, max 240 characters): ${safe(row.excerpt)}`);
        if (row.summary !== null) lines.push(`  Scoped summary (not acceptance): ${safe(row.summary)}`);
        if (row.review) lines.push(`  Review: ${safe(row.review.subject)} / ${safe(row.review.reviewer)}; independent=${row.review.independent}; date=${row.review.reviewed_on}; age_hint=${row.review_age_days}d (not validity)`);
      }
      if (output.truncated) {
        lines.push(byDocument
          ? `Showing ${output.matches.length}/${output.selected_total} documents from ${output.total} matching entries; use --all to enumerate ${output.selected_total} documents (~${output.all_estimate} tokens), or narrow --task/--query.`
          : `Showing ${output.matches.length}/${output.total}; narrow --task or --query, or use --limit up to 20.`);
      }
    }
  }
  lines.push(`Fallback: ${output.fallback}`);
  return lines.join('\n');
}

export function main(args) {
  const options = parseArgs(args);
  let curated = null;
  try {
    curated = inspectCatalog(options);
  } catch (error) {
    if (options.command === 'check' || !String(error.message).includes('catalog missing')) throw error;
  }
  if (!curated && options.scope === 'catalog' && options.command === 'boot') options.scope = 'documents';
  if (!curated && options.scope === 'catalog') {
    throw new Error(`${catalogCandidates.join(' or ')}: catalog missing; use --scope documents or the existing reading index`);
  }
  let report = curated;
  if (options.scope === 'documents') {
    const discovered = discoverDocuments({ ...options, roots: options.roots ?? curated?.roots ?? ['.'] });
    requireValue(!curated || discovered.commit === curated.commit, 'HEAD changed while reading; retry or use commit mode');
    report = { ...discovered, records: [...(curated?.records ?? []), ...discovered.records] };
  }
  const output = options.command === 'check' ? report : queryEntries(report, options);
  console.log(options.json ? JSON.stringify(output, null, 2) : render(output, options.command));
  return options.command === 'check'
    ? (report.records.some(record => record.issues.length || record.entries.some(entry => entry.issues.length)) ? 1 : 0)
    : (output.matches.some(row => row.issues.length) ? 1 : 0);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error(`stackmap: ${error.message}`); process.exitCode = 2; }
}
