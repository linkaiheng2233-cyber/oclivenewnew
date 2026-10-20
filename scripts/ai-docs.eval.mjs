#!/usr/bin/env node
/**
 * Optional quality check for the documentation navigator.
 *
 * It measures two things against this checkout:
 *  - locate:   for a task question, does the expected original appear in the
 *              first 8 / 12 / 20 documents?
 *  - coupling: for a literal marker, is the returned document set complete
 *              compared with a full-text baseline over the same corpus?
 *
 * Report only by default; `--gate` turns the thresholds into an exit code.
 * It never writes, never starts a service and never reads the network.
 */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { discoverDocuments, inspectCatalog, queryEntries } from './ai-docs.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const gate = process.argv.includes('--gate');

const LOCATE = [
  { q: 'build_prompt', expect: ['human-docs/modules/slots/prompt.md', 'handoff/MODULE_MAP_AND_HANDOFF.md'] },
  { q: 'Done-eligible', expect: ['handoff/debt-marathon/AI_AND_PIPELINE_GATES.md', 'handoff/workflows/dev-pipeline/SKILL.md'] },
  { q: 'fallback', expect: ['creator-docs/plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md', 'creator-docs/plugin-and-architecture/PLUGIN_V1.md'] },
  { q: 'sandbox', expect: ['handoff/debt-marathon/long-plans/K-PLUGIN-SEC-01.md', 'creator-docs/security/SUPPLY_CHAIN.md'] },
  { q: 'oclive.toml', expect: ['creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md', 'handoff/THREE_DISTRO_KERNEL_CLOSURE.md'] },
  { q: 'ERROR_CODES', expect: ['creator-docs/getting-started/ERROR_CODES.md', 'creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md'] },
  { q: 'MODULE_MAP', expect: ['handoff/MODULE_MAP_AND_HANDOFF.md'] },
  { q: 'CHAT_STORAGE', expect: ['handoff/CHAT_STORAGE_ARCHITECTURE.md'] },
];

/** Literal markers used to check completeness. Fingerprint-like terms must be fully covered. */
const COUPLING = ['module-compat', 'K-SUPPLY-12', 'check:module-compat'];

let curated = { records: [] };
try {
  curated = inspectCatalog({ root });
} catch {
  // No curated catalog is a supported mode; locate cases still run on discovery.
}

const discovered = discoverDocuments({ root });
const report = { ...discovered, records: [...curated.records, ...discovered.records] };

const locate = limit => LOCATE.map(({ q, expect }) => {
  const out = queryEntries(report, { query: q, select: 'documents', limit });
  const paths = out.matches.map(row => row.path);
  return expect.some(item => paths.includes(item));
});

const baseline = term => new Set(discovered.records
  .filter(record => record.entries.some(entry => entry.search_lines?.some(line => line.text.toLowerCase().includes(term.toLowerCase()))))
  .map(record => record.path));

const coupling = term => {
  const truth = baseline(term);
  const out = queryEntries(report, { query: term, select: 'documents', all: true });
  const paths = out.matches.map(row => row.path);
  const hit = [...truth].filter(item => paths.includes(item)).length;
  return { term, truth: truth.size, returned: paths.length, hit, complete: hit === truth.size };
};

const rows = [8, 12, 20].map(limit => {
  const hits = locate(limit);
  return { limit, hits: hits.filter(Boolean).length, total: hits.length };
});

console.log('locate recall (documents mode)');
for (const row of rows) console.log(`  limit ${String(row.limit).padStart(2)}: ${row.hits}/${row.total}`);

console.log('\ncoupling completeness (--all vs full-text baseline)');
let complete = true;
for (const term of COUPLING) {
  const result = coupling(term);
  if (!result.complete) complete = false;
  console.log(`  ${term.padEnd(18)} baseline=${String(result.truth).padStart(3)} returned=${String(result.returned).padStart(3)} hit=${String(result.hit).padStart(3)} ${result.complete ? 'complete' : 'INCOMPLETE'}`);
}

const top = rows.find(row => row.limit === 20);
const locateOk = top.hits === top.total;
console.log(`\nresult: locate@20 ${top.hits}/${top.total}; coupling ${complete ? 'complete' : 'INCOMPLETE'}`);
if (gate) process.exitCode = locateOk && complete ? 0 : 1;
