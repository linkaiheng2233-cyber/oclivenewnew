#!/usr/bin/env node
/**
 * Validate data/ci/schemes/*.json against the CI data they claim to summarise.
 *
 * Phase A checks the static contract only:
 *  - required fields and a matching INDEX.md row;
 *  - every include resolves to a tracked path (exact file or `<dir>/**`);
 *  - every job exists in the validation catalog;
 *  - no include overlaps a risk-override selector;
 *  - derived_from hashes match the current blobs.
 *
 * Planner equality (scheme.jobs === planner output for the include union) is a Phase B
 * check that runs against the trusted-base planner; see data/ci/schemes/README.md.
 */
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const defaultRoot = fileURLToPath(new URL('../', import.meta.url));
const SCHEME_DIR = 'data/ci/schemes';

export function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

export function loadContext(root = defaultRoot) {
  const catalog = readJson(path.join(root, 'data/ci/validation-catalog.v1.json'));
  const impactMap = readJson(path.join(root, 'data/ci/impact-map.v1.json'));
  return {
    root,
    validators: new Set((catalog.validators ?? []).map(item => item.id)),
    overrides: impactMap.risk_overrides ?? [],
    hashOf: repoPath => execFileSync('git', ['-C', root, 'rev-parse', `HEAD:${repoPath}`], { encoding: 'utf8' }).trim(),
    exists: repoPath => fs.existsSync(path.join(root, repoPath)),
  };
}

/** `<dir>/**` means everything under dir; every other include must be an exact path. */
export function includeKind(pattern) {
  if (pattern.endsWith('/**')) return { kind: 'dir', dir: pattern.slice(0, -3) };
  if (pattern.includes('*')) return { kind: 'unsupported' };
  return { kind: 'file', file: pattern };
}

export function selectorsOverlap(override, include) {
  const shape = includeKind(include);
  if (shape.kind === 'unsupported') return true;
  for (const selector of override.selectors ?? []) {
    const value = selector.value;
    if (selector.kind === 'exact') {
      if (shape.kind === 'file' && shape.file === value) return true;
      if (shape.kind === 'dir' && value.startsWith(`${shape.dir}/`)) return true;
    } else if (selector.kind === 'prefix') {
      if (shape.kind === 'file' && (shape.file === value || shape.file.startsWith(`${value}/`))) return true;
      if (shape.kind === 'dir' && (value === shape.dir || value.startsWith(`${shape.dir}/`) || shape.dir.startsWith(`${value}/`))) return true;
    }
  }
  return false;
}

export function validateScheme(scheme, ctx, indexText = '') {
  const problems = [];
  const fail = message => problems.push(`${scheme.id ?? '<no id>'}: ${message}`);
  for (const field of ['id', 'title', 'status', 'includes', 'jobs', 'derived_from', 'author', 'created_at', 'rationale']) {
    if (scheme[field] === undefined) fail(`missing field ${field}`);
  }
  if (!['draft', 'active'].includes(scheme.status)) fail(`status must be draft or active, got ${scheme.status}`);
  if (!Array.isArray(scheme.includes) || scheme.includes.length === 0) fail('includes must be a non-empty array');
  if (!Array.isArray(scheme.jobs) || scheme.jobs.length === 0) fail('jobs must be a non-empty array');
  if (!indexText.includes(`\`${scheme.id}\``)) fail('missing INDEX.md row');

  for (const include of scheme.includes ?? []) {
    const shape = includeKind(include);
    if (shape.kind === 'unsupported') { fail(`unsupported include pattern ${include}`); continue; }
    const target = shape.kind === 'file' ? shape.file : shape.dir;
    if (!ctx.exists(target)) fail(`include does not exist: ${include}`);
    for (const override of ctx.overrides) {
      if (selectorsOverlap(override, include)) fail(`include overlaps risk override ${override.id}: ${include}`);
    }
  }
  for (const job of scheme.jobs ?? []) {
    if (!ctx.validators.has(job)) fail(`unknown job ${job}`);
  }
  const expected = {
    impact_map: 'data/ci/impact-map.v1.json',
    catalog: 'data/ci/validation-catalog.v1.json',
    planner: 'kernel/crates/oclive_ci_plan',
  };
  for (const [key, repoPath] of Object.entries(expected)) {
    let actual;
    try { actual = ctx.hashOf(repoPath); } catch { fail(`cannot read ${repoPath}`); continue; }
    if (scheme.derived_from?.[key] !== actual) fail(`stale derived_from.${key}: ${scheme.derived_from?.[key]} != ${actual} (run: node scripts/check-ci-schemes.mjs --fix-hashes)`);
  }
  return problems;
}

export function validateAll(root = defaultRoot) {
  const dir = path.join(root, SCHEME_DIR);
  const indexText = fs.existsSync(path.join(dir, 'INDEX.md')) ? fs.readFileSync(path.join(dir, 'INDEX.md'), 'utf8') : '';
  const ctx = loadContext(root);
  const problems = [];
  const schemes = fs.readdirSync(dir).filter(name => name.endsWith('.json'));
  for (const name of schemes) {
    const scheme = readJson(path.join(dir, name));
    if (`${scheme.id}.json` !== name) problems.push(`${name}: id must match the file name`);
    problems.push(...validateScheme(scheme, ctx, indexText));
  }
  return { problems, schemes };
}

export function applyHashes(text, hashes) {
  let out = text;
  for (const [key, value] of Object.entries(hashes)) {
    const pattern = new RegExp(`("${key}"\\s*:\\s*")[0-9a-f]{40}(")`);
    if (!pattern.test(out)) throw new Error(`missing derived_from.${key}`);
    out = out.replace(pattern, `$1${value}$2`);
  }
  return out;
}

export function applyIndexHashes(indexText, schemes, hashes) {
  let out = indexText;
  for (const scheme of schemes) {
    const short = [hashes.impact_map.slice(0, 8), hashes.catalog.slice(0, 8), hashes.planner.slice(0, 8)];
    const pattern = new RegExp(`(\\| \`${scheme.id}\` \\|[^\\n]*?\`)[0-9a-f]{8}(\` / \`)[0-9a-f]{8}(\` / \`)[0-9a-f]{8}(\` \\|)`);
    if (!pattern.test(out)) throw new Error(`missing INDEX row for ${scheme.id}`);
    out = out.replace(pattern, `$1${short[0]}$2${short[1]}$3${short[2]}$4`);
  }
  return out;
}

function fixHashes(root) {
  const git = args => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim();
  git(['add', '-A']);
  const tree = git(['write-tree']);
  const hashes = {
    impact_map: git(['rev-parse', `${tree}:data/ci/impact-map.v1.json`]),
    catalog: git(['rev-parse', `${tree}:data/ci/validation-catalog.v1.json`]),
    planner: git(['rev-parse', `${tree}:kernel/crates/oclive_ci_plan`]),
  };
  const dir = path.join(root, SCHEME_DIR);
  const schemes = [];
  for (const name of fs.readdirSync(dir).filter(item => item.endsWith('.json'))) {
    const file = path.join(dir, name);
    const scheme = readJson(file);
    fs.writeFileSync(file, applyHashes(fs.readFileSync(file, 'utf8'), hashes), 'utf8');
    schemes.push(scheme);
  }
  const indexPath = path.join(dir, 'INDEX.md');
  if (fs.existsSync(indexPath)) {
    fs.writeFileSync(indexPath, applyIndexHashes(fs.readFileSync(indexPath, 'utf8'), schemes, hashes), 'utf8');
  }
  console.log(`check-ci-schemes: rewrote ${schemes.length} schemes (impact_map ${hashes.impact_map.slice(0, 8)}, catalog ${hashes.catalog.slice(0, 8)}, planner ${hashes.planner.slice(0, 8)})`);
}

function main() {
  const args = process.argv.slice(2);
  const fix = args.includes('--fix-hashes');
  const positional = args.filter(arg => arg !== '--fix-hashes');
  const root = positional[0] ? path.resolve(positional[0]) : defaultRoot;
  if (fix) {
    fixHashes(root);
    return;
  }
  let result;
  try {
    result = validateAll(root);
  } catch (error) {
    console.error(`check-ci-schemes: ${error.message}`);
    process.exitCode = 2;
    return;
  }
  if (result.problems.length > 0) {
    for (const problem of result.problems) console.error(`  - ${problem}`);
    console.error(`check-ci-schemes: FAIL (${result.problems.length} problems, ${result.schemes.length} schemes)`);
    process.exitCode = 1;
    return;
  }
  console.log(`check-ci-schemes: ok (${result.schemes.length} schemes)`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
