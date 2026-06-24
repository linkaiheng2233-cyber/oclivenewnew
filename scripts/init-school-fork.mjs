#!/usr/bin/env node
/**
 * Mark a clone as school/enterprise downstream and wire upstream remote.
 * Run at the school repo root after mirror push.
 *
 * Usage:
 *   node scripts/init-school-fork.mjs --upstream URL [--baseline-tag TAG] [--dry-run]
 */
import { execSync } from 'child_process';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';

const ROOT = join(fileURLToPath(new URL('..', import.meta.url)));

function parseArgs() {
  const out = { upstream: '', baselineTag: 'school-baseline-v0.4.0', dryRun: false };
  const argv = process.argv.slice(2);
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--upstream' && argv[i + 1]) {
      out.upstream = argv[++i];
    } else if (argv[i] === '--baseline-tag' && argv[i + 1]) {
      out.baselineTag = argv[++i];
    } else if (argv[i] === '--dry-run') {
      out.dryRun = true;
    } else if (argv[i] === '--help' || argv[i] === '-h') {
      console.log(`Usage: node scripts/init-school-fork.mjs --upstream URL [--baseline-tag TAG] [--dry-run]`);
      process.exit(0);
    }
  }
  return out;
}

function run(cmd, opts = {}) {
  return execSync(cmd, { cwd: ROOT, encoding: 'utf8', stdio: opts.silent ? 'pipe' : 'inherit' });
}

function runSilent(cmd) {
  return run(cmd, { silent: true }).trim();
}

function remoteUrl(name) {
  try {
    return runSilent(`git remote get-url ${name}`);
  } catch {
    return '';
  }
}

const { upstream, baselineTag, dryRun } = parseArgs();

if (!upstream) {
  console.error('init-school-fork: missing --upstream URL');
  console.error('Example: node scripts/init-school-fork.mjs --upstream https://github.com/linkaiheng2233-cyber/oclivenewnew');
  process.exit(1);
}

let headSha = '';
try {
  headSha = runSilent('git rev-parse HEAD');
} catch {
  console.error('init-school-fork: not a git repository');
  process.exit(1);
}

const originUrl = remoteUrl('origin');
const existingUpstream = remoteUrl('upstream');

if (dryRun) {
  console.log('[dry-run] would configure school fork:');
  console.log('  upstream:', upstream);
  console.log('  baseline-tag:', baselineTag);
  console.log('  HEAD:', headSha);
  console.log('  origin:', originUrl || '(none)');
  process.exit(0);
}

if (existingUpstream) {
  if (existingUpstream !== upstream) {
    console.log(`init-school-fork: updating upstream remote → ${upstream}`);
    run(`git remote set-url upstream ${JSON.stringify(upstream).slice(1, -1)}`);
  } else {
    console.log('init-school-fork: upstream remote already set');
  }
} else if (originUrl === upstream) {
  console.log('init-school-fork: renaming origin → upstream');
  run('git remote rename origin upstream');
} else {
  console.log(`init-school-fork: adding upstream remote → ${upstream}`);
  run(`git remote add upstream ${JSON.stringify(upstream).slice(1, -1)}`);
}

const metaDir = join(ROOT, '.oclive');
mkdirSync(metaDir, { recursive: true });
const meta = {
  kind: 'school-enterprise-fork',
  upstream_url: upstream,
  baseline_tag: baselineTag,
  initialized_at: new Date().toISOString(),
  initialized_from_commit: headSha,
  doc: 'handoff/distros/SCHOOL_ENTERPRISE_FORK.md',
};
const metaPath = join(metaDir, 'school-fork.json');
writeFileSync(metaPath, `${JSON.stringify(meta, null, 2)}\n`, 'utf8');

console.log('');
console.log('init-school-fork: OK');
console.log(`  wrote ${metaPath.replace(/\\/g, '/')}`);
console.log('');
console.log('Next steps (school repo maintainers):');
console.log('  1. Add README banner — see handoff/distros/SCHOOL_ENTERPRISE_FORK.md §3.4');
console.log('  2. Create handoff/distros/SCHOOL_CUSTOMIZATIONS.md for local-only notes');
console.log(`  3. git fetch upstream && git tag -l '${baselineTag}'`);
console.log('  4. node scripts/dimension5-acceptance.mjs --ci');
