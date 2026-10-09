#!/usr/bin/env node
/** Source observations, not a gate on abstraction counts. Historical filename retained.
 * Git-tracked working files only; lexical Rust matches are not cfg/macro-resolved impls.
 * Exit 0 means the observation completed. Only input/collection failures exit nonzero.
 * Usage: node scripts/check-abstraction-ratchet.mjs [--json | --self-test]
 */
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CONTRACTS = 'kernel/crates/oclive_kernel_contracts/src/';
const SOURCE = /^(?:kernel\/crates\/|distros\/).*\.(?:rs|ts|vue)$/;
const METHOD = 'git-tracked-lexical-v1';

// Keep line positions while excluding comments and literals. This is a lexical aid,
// not a Rust parser: cfg, aliases, macros, name resolution and reachability stay unknown.
function rustCode(text) {
  const tokens = /\/\/|\/\*|\b(?:br|cr|r)(#{0,255})"|"(?:\\[\s\S]|[^"\\])*"|'(?:\\(?:u\{[\da-fA-F]+\}|x[\da-fA-F]{2}|[\s\S])|[^\\'\r\n])'/gu;
  const chunks = [];
  let cursor = 0;
  for (let match; (match = tokens.exec(text));) {
    const start = match.index;
    let end = tokens.lastIndex;
    if (match[0] === '//') {
      end = text.indexOf('\n', end);
      if (end < 0) end = text.length;
    } else if (match[0] === '/*') {
      const brackets = /\/\*|\*\//g;
      brackets.lastIndex = end;
      let depth = 1;
      for (let bracket; depth > 0 && (bracket = brackets.exec(text));) {
        depth += bracket[0] === '/*' ? 1 : -1;
        end = brackets.lastIndex;
      }
      if (depth !== 0) throw new Error('unterminated Rust block comment');
    } else if (match[1] !== undefined) {
      const closing = `"${match[1]}`;
      const closeAt = text.indexOf(closing, end);
      if (closeAt < 0) throw new Error('unterminated Rust raw string');
      end = closeAt + closing.length;
    }
    chunks.push(text.slice(cursor, start), text.slice(start, end).replace(/[^\r\n]/g, ' '));
    cursor = end;
    tokens.lastIndex = end;
  }
  return chunks.join('') + text.slice(cursor);
}

function location(file, text, offset) {
  return { path: file, line: text.slice(0, offset).split('\n').length };
}

function validatePaths(paths) {
  const unique = new Set();
  for (const file of paths) {
    if (!file || file.includes('\\') || path.posix.isAbsolute(file)
      || file.split('/').some(part => part === '..' || part === '.' || !part)
      || unique.has(file)) throw new Error(`invalid or duplicate tracked path: ${file}`);
    unique.add(file);
  }
}

function collect(root, paths, read) {
  validatePaths(paths);
  const checkedPaths = new Set([root]);
  const readText = read ?? (file => {
    const absolute = path.join(root, file);
    for (let current = absolute; !checkedPaths.has(current); current = path.dirname(current)) {
      if (fs.lstatSync(current).isSymbolicLink()) throw new Error(`symlink input: ${file}`);
      checkedPaths.add(current);
    }
    return new TextDecoder('utf-8', { fatal: true }).decode(fs.readFileSync(absolute));
  });
  const sourcePaths = paths.filter(file => SOURCE.test(file));
  if (!sourcePaths.some(file => file.startsWith(CONTRACTS))) throw new Error('contracts scope is empty');
  const sources = sourcePaths.map(file => {
    try {
      const text = readText(file); // One read per source file; failures must fail collection.
      return { file, text, code: file.endsWith('.rs') ? rustCode(text) : null };
    } catch (error) {
      throw new Error(`${file}: ${error.message}`);
    }
  });
  const traits = sources.filter(source => source.file.startsWith(CONTRACTS)).flatMap(source =>
    [...source.code.matchAll(/\bpub\s+(?:unsafe\s+)?trait\s+([A-Za-z_]\w*)/g)]
      .map(match => ({ name: match[1], ...location(source.file, source.text, match.index) })));
  const observations = traits.map(trait => {
    const pattern = new RegExp(`\\bimpl(?:\\s*<[^;{}]*>)?\\s+(?:[A-Za-z_]\\w*\\s*::\\s*)*${trait.name}\\b(?:\\s*<[^;{}]*>)?\\s+for\\b`, 'g');
    const matches = sources.filter(source => source.code !== null).flatMap(source =>
      [...source.code.matchAll(pattern)].map(match => location(source.file, source.text, match.index)));
    return { ...trait, impl_text_matches: matches.length, matches };
  });
  const largest = sources.map(({ file, text }) => ({ path: file,
    lines: text === '' ? 0 : text.replace(/\r\n/g, '\n').replace(/\n$/, '').split('\n').length }))
    .sort((a, b) => b.lines - a.lines || a.path.localeCompare(b.path))[0];
  return {
    mode: 'OBSERVATION_ONLY', method: METHOD,
    scope: { source: 'Git-tracked working files in kernel/crates and distros (.rs/.ts/.vue)',
      traits: CONTRACTS, scripts: 'Git-tracked files recursively under scripts/',
      untracked_and_ignored: 'excluded', source_files: sources.length },
    metrics: { contract_trait_declarations: traits.length,
      traits_with_one_impl_text_match: observations.filter(trait => trait.impl_text_matches === 1).length,
      max_source_file_lines: largest.lines, scripts_tracked_files: paths.filter(file => file.startsWith('scripts/')).length },
    largest_source: largest, traits: observations,
    limitations: ['Text matches do not resolve cfg, macros, aliases, same-name traits or external implementations.',
      'Zero matches means no matching text in this scope; it does not prove a capability is absent.',
      'Counts include inline tests and enabled/disabled source alike; they do not measure runtime or quality.',
      'Growth or a single implementation is a review signal, not an error or an instruction to refactor.'],
  };
}

function options(args) {
  if (args.length > 1 || args.some(arg => !['--json', '--self-test'].includes(arg))) {
    throw new Error('usage: check-abstraction-ratchet.mjs [--json | --self-test]');
  }
  return args[0];
}

function configuration(value) {
  if (value?.abstraction_observation?.mode !== 'observe-only'
    || value.abstraction_observation.method !== METHOD) throw new Error('invalid observation configuration');
  return value;
}

function selfTest() {
  const contract = `${CONTRACTS}lib.rs`;
  const implementation = 'kernel/crates/example/src/lib.rs';
  const files = new Map([[contract, 'pub trait Port {}\npub trait Missing {}'],
    [implementation, 'impl<T: Bound<Nested<T>>>\n crate::Port\n for Adapter<T> {}']]);
  const observe = () => collect(ROOT, [...files.keys()], file => files.get(file));
  assert.equal(observe().traits[0].impl_text_matches, 1);
  assert.equal(observe().traits[0].matches[0].line, 1);
  assert.equal(observe().traits[1].impl_text_matches, 0);
  files.set(implementation, '// impl Port for Fake {}\n/* nested /* impl Port for Fake {} */ */\nconst S: &str = r##"impl Port for Fake {}"##;\nconst T: &str = "impl Port for Fake {}";\nimpl crate::Port for Real {}');
  assert.equal(observe().traits[0].impl_text_matches, 1);
  assert.equal(observe().traits[0].matches[0].line, 5);
  assert.equal(rustCode("fn f<'a>(x: &'a str) { let c = '中'; }\n").includes("&'a str"), true);
  files.set('distros/test/src/extra.ts', 'line\n'.repeat(5000));
  files.set('scripts/new/check.mjs', '');
  const growth = observe();
  assert.equal(growth.mode, 'OBSERVATION_ONLY');
  assert.equal(growth.metrics.max_source_file_lines, 5000);
  assert.equal(growth.metrics.scripts_tracked_files, 1);
  assert.equal(growth.limitations.some(text => text.startsWith('Zero matches')), true);
  assert.throws(() => collect(ROOT, [], () => ''), /scope is empty/);
  assert.throws(() => collect(ROOT, [contract], () => { throw new Error('read denied'); }), /read denied/);
  assert.throws(() => validatePaths(['../escape.rs']), /invalid/);
  assert.throws(() => validatePaths([contract, contract]), /duplicate/);
  assert.throws(() => rustCode('/* unfinished'), /unterminated/);
  assert.throws(() => options(['--strict']), /usage/);
  assert.throws(() => options(['--json', '--self-test']), /usage/);
  assert.throws(() => configuration({ abstraction_observation: { mode: 'ratchet' } }), /configuration/);
  assert.doesNotThrow(() => configuration({ abstraction_observation: { mode: 'observe-only', method: METHOD } }));
  console.log('[abstraction] self-test PASS (qualified/multiline impl, literals, growth, input and read failures)');
}

try {
  const option = options(process.argv.slice(2));
  if (option === '--self-test') {
    selfTest();
  } else {
    configuration(JSON.parse(fs.readFileSync(path.join(ROOT, 'handoff/LAYERING_BASELINE.json'), 'utf8')));
    const git = args => execFileSync('git', args, { cwd: ROOT, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
    const report = collect(ROOT, git(['ls-files', '--cached', '-z']).split('\0').filter(Boolean));
    report.head = git(['rev-parse', 'HEAD']).trim();
    report.working_tree = 'Working bytes may differ from HEAD; Git-tracked scope is from the current index.';
    if (option === '--json') console.log(JSON.stringify(report, null, 2));
    else {
      console.log(`[abstraction] OBSERVATION ONLY (${report.method}; ${report.scope.source_files} source files)`);
      console.log(JSON.stringify(report.metrics));
      console.log(`largest: ${report.largest_source.path}:${report.largest_source.lines} lines`);
      console.log('Single-match declarations: ' + report.traits.filter(trait => trait.impl_text_matches === 1)
        .map(trait => `${trait.name} (${trait.path}:${trait.line})`).join(', '));
      console.log('No numeric caps. Use --json for match locations, scope and limitations.');
    }
  }
} catch (error) {
  console.error(`[abstraction] collection failed: ${error.message}`);
  process.exitCode = 1;
}
