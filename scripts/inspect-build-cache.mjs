#!/usr/bin/env node
/** Bounded metadata-only cache inspection; it never grants or performs cleanup. */
import fs from 'node:fs';
import path from 'node:path';
import { performance } from 'node:perf_hooks';
import { fileURLToPath } from 'node:url';

const DEFAULT_MAX_ENTRIES = 200000;
const DEFAULT_MAX_DURATION_MS = 30000;
const samePath = (left, right) => process.platform === 'win32'
  ? path.resolve(left).toLowerCase() === path.resolve(right).toLowerCase()
  : path.resolve(left) === path.resolve(right);

function emptyReport(root = null) {
  return {
    schema_version: 1, scope: 'regular-file-path-logical-metadata', root,
    complete: false, stop_reason: null, entries_examined: 0, directories: 0,
    regular_file_paths: 0, hardlinked_file_paths: 0,
    skipped_symbolic_links: 0, skipped_other_entries: 0,
    path_logical_bytes: '0', physical_bytes: null, reclaimable_bytes: null,
    retention_assessed: false, atomic_snapshot: false,
    logical_limit_bytes: null, alert: null, elapsed_ms: 0, errors: [],
  };
}

function positiveInteger(value, label) {
  if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`${label} must be a positive safe integer`);
  return value;
}

function checkedRoot(input) {
  if (typeof input !== 'string' || !input.trim()) throw new Error('explicit --root PATH is required');
  const root = path.resolve(input);
  const volume = path.parse(root).root;
  if (samePath(root, volume)) throw new Error('filesystem root cannot be inspected');
  let ancestor = volume;
  for (const segment of path.relative(volume, root).split(path.sep)) {
    ancestor = path.join(ancestor, segment);
    if (fs.lstatSync(ancestor).isSymbolicLink()) throw new Error(`symbolic link in cache root or ancestor: ${ancestor}`);
  }
  if (!fs.lstatSync(root).isDirectory()) throw new Error('cache root must be an existing directory');
  return fs.realpathSync(root);
}

export function inspectBuildCache({
  root: input, maxEntries = DEFAULT_MAX_ENTRIES, maxDurationMs = DEFAULT_MAX_DURATION_MS,
  maxLogicalBytes = null, clock = () => performance.now(),
} = {}) {
  positiveInteger(maxEntries, 'max entries');
  positiveInteger(maxDurationMs, 'max duration');
  if (maxLogicalBytes !== null && (typeof maxLogicalBytes !== 'bigint' || maxLogicalBytes < 0n)) {
    throw new Error('max logical bytes must be a nonnegative bigint');
  }
  const root = checkedRoot(input);
  const report = emptyReport(root);
  report.max_entries = maxEntries;
  report.max_duration_ms = maxDurationMs;
  report.logical_limit_bytes = maxLogicalBytes?.toString() ?? null;
  report.directories = 1;
  const start = clock();
  let logicalBytes = 0n;
  let activePath = root;
  const directories = [];
  try {
    directories.push(fs.opendirSync(root));
    while (directories.length) {
      if (clock() - start > maxDurationMs) { report.stop_reason = 'time_budget'; break; }
      const directory = directories.at(-1);
      const entry = directory.readSync();
      if (entry === null) { directory.closeSync(); directories.pop(); continue; }
      if (report.entries_examined >= maxEntries) { report.stop_reason = 'entry_budget'; break; }
      activePath = path.join(directory.path, entry.name);
      const metadata = fs.lstatSync(activePath, { bigint: true });
      report.entries_examined += 1;
      if (metadata.isSymbolicLink()) report.skipped_symbolic_links += 1;
      else if (metadata.isDirectory()) {
        if (!samePath(fs.realpathSync(activePath), activePath)) throw new Error(`directory path changed or resolves through a link: ${activePath}`);
        directories.push(fs.opendirSync(activePath));
        report.directories += 1;
      } else if (metadata.isFile()) {
        report.regular_file_paths += 1;
        if (metadata.nlink > 1n) report.hardlinked_file_paths += 1;
        logicalBytes += metadata.size;
      } else report.skipped_other_entries += 1;
    }
  } catch (error) {
    report.stop_reason = 'metadata_error';
    report.errors.push({ path: activePath, code: error.code ?? null, message: error.message });
  } finally {
    for (const directory of directories.reverse()) {
      try { directory.closeSync(); }
      catch (error) {
        report.stop_reason ??= 'metadata_error';
        report.errors.push({ path: directory.path, code: error.code ?? null, message: error.message });
      }
    }
  }
  report.elapsed_ms = Math.max(0, clock() - start);
  // Include the final read/handle-close window before giving a complete verdict.
  if (report.stop_reason === null && report.elapsed_ms > maxDurationMs) report.stop_reason = 'time_budget';
  report.path_logical_bytes = logicalBytes.toString();
  report.complete = report.stop_reason === null;
  report.alert = report.complete ? maxLogicalBytes !== null && logicalBytes > maxLogicalBytes : null;
  return report;
}

function optionsFromArgs(args) {
  const allowed = new Set(['--root', '--max-entries', '--max-seconds', '--max-logical-bytes']);
  const values = new Map();
  for (let index = 0; index < args.length; index += 2) {
    const flag = args[index];
    if (!allowed.has(flag) || values.has(flag)) throw new Error(`unknown or duplicate option: ${flag}`);
    const value = args[index + 1];
    if (!value || value.startsWith('--')) throw new Error(`missing value for ${flag}`);
    values.set(flag, value);
  }
  const integer = (flag, fallback) => {
    if (!values.has(flag)) return fallback;
    const text = values.get(flag);
    if (!/^\d+$/.test(text)) throw new Error(`${flag} must be an unsigned integer`);
    return positiveInteger(Number(text), flag);
  };
  const maxEntries = integer('--max-entries', DEFAULT_MAX_ENTRIES);
  const maxDurationMs = integer('--max-seconds', DEFAULT_MAX_DURATION_MS / 1000) * 1000;
  positiveInteger(maxDurationMs, 'max duration');
  let maxLogicalBytes = null;
  if (values.has('--max-logical-bytes')) {
    const text = values.get('--max-logical-bytes');
    if (!/^\d+$/.test(text)) throw new Error('--max-logical-bytes must be an unsigned integer');
    maxLogicalBytes = BigInt(text);
  }
  return { root: values.get('--root'), maxEntries, maxDurationMs, maxLogicalBytes };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  let report;
  try { report = inspectBuildCache(optionsFromArgs(process.argv.slice(2))); }
  catch (error) {
    report = emptyReport();
    report.stop_reason = 'invalid_input';
    report.errors.push({ code: error.code ?? null, message: error.message });
  }
  console.log(JSON.stringify(report, null, 2));
  process.exitCode = !report.complete ? 2 : report.alert ? 1 : 0;
}
