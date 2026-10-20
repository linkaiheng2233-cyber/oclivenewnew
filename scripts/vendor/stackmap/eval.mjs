#!/usr/bin/env node
/**
 * Optional quality check for the documentation navigator.
 *
 * Cases live in a repository-owned JSON file so the same harness can run in any
 * repository: `{ "locate": [{ "q", "expect": [paths] }], "coupling": [terms] }`.
 *
 * It measures:
 *  - locate:   does the expected original appear in the first 8 / 12 / 20 documents?
 *  - coupling: is the returned document set complete against a full-text baseline?
 *
 * Report only by default; `--gate` turns the thresholds into an exit code.
 * Read-only: no writes, no service, no network.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { discoverDocuments, inspectCatalog, queryEntries } from './cli.mjs';

const packageRoot = fileURLToPath(new URL('./', import.meta.url));
const args = process.argv.slice(2);
const value = flag => {
  const index = args.indexOf(flag);
  return index >= 0 ? args[index + 1] : undefined;
};
const gate = args.includes('--gate');
const root = value('--root') ? path.resolve(value('--root')) : packageRoot;

const caseCandidates = [
  value('--cases'),
  path.join(root, 'catalog/eval.cases.json'),
  path.join(root, 'handoff/catalog/eval.cases.json'),
].filter(Boolean);
const casesPath = caseCandidates.find(candidate => fs.existsSync(candidate));
if (!casesPath) {
  console.error(`stackmap eval: no cases file; looked for ${caseCandidates.join(' or ')}`);
  process.exitCode = 2;
} else {
  run(JSON.parse(fs.readFileSync(casesPath, 'utf8')));
}

function run(cases) {
  const locate = Array.isArray(cases.locate) ? cases.locate : [];
  const couplingTerms = Array.isArray(cases.coupling) ? cases.coupling : [];

  let curated = { records: [] };
  try {
    curated = inspectCatalog({ root });
  } catch {
    // No curated catalog is a supported mode; locate cases still run on discovery.
  }

  const discovered = discoverDocuments({ root, roots: curated.roots ?? ['.'] });
  const report = { ...discovered, records: [...curated.records, ...discovered.records] };

  const hits = limit => locate.map(({ q, expect }) => {
    const out = queryEntries(report, { query: q, select: 'documents', limit });
    const paths = out.matches.map(row => row.path);
    return expect.some(item => paths.includes(item));
  });

  /** The navigator drops a mirrored `*-en` document when its translated twin is also listed. */
  const dropMirrors = paths => {
    const present = new Set(paths);
    return paths.filter(item => {
      const match = /(^|\/)([^/]+)-en\//.exec(item);
      return match === null || !present.has(item.replace(`${match[2]}-en/`, `${match[2]}/`));
    });
  };

  const baseline = term => new Set(dropMirrors(discovered.records
    .filter(record => record.entries.some(entry => entry.search_lines?.some(line => line.text.toLowerCase().includes(term.toLowerCase()))))
    .map(record => record.path)));

  const rows = [8, 12, 20].map(limit => {
    const result = hits(limit);
    return { limit, hits: result.filter(Boolean).length, total: result.length };
  });

  console.log(`stackmap eval: ${path.relative(root, casesPath) || casesPath}`);
  console.log('locate recall (documents mode)');
  for (const row of rows) console.log(`  limit ${String(row.limit).padStart(2)}: ${row.hits}/${row.total}`);

  let complete = true;
  if (couplingTerms.length > 0) {
    console.log('\ncoupling completeness (--all vs full-text baseline)');
    for (const term of couplingTerms) {
      const truth = baseline(term);
      const out = queryEntries(report, { query: term, select: 'documents', all: true });
      const paths = out.matches.map(row => row.path);
      const hit = [...truth].filter(item => paths.includes(item)).length;
      const ok = hit === truth.size;
      if (!ok) complete = false;
      console.log(`  ${term.padEnd(18)} baseline=${String(truth.size).padStart(3)} returned=${String(paths.length).padStart(3)} hit=${String(hit).padStart(3)} ${ok ? 'complete' : 'INCOMPLETE'}`);
    }
  }

  const top = rows.find(row => row.limit === 20);
  const locateOk = locate.length === 0 || top.hits === top.total;
  console.log(`\nresult: locate@20 ${top.hits}/${top.total}; coupling ${complete ? 'complete' : 'INCOMPLETE'}`);
  if (gate) process.exitCode = locateOk && complete ? 0 : 1;
}
