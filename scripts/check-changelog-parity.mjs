#!/usr/bin/env node
/**
 * K-DOC-02: [Unreleased] section parity between CHANGELOG.md and CHANGELOG.en.md.
 * Requires matching ### subsection titles and bullet counts per subsection.
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

function extractUnreleasedSections(markdown) {
  const unreleasedRe = /^## \[Unreleased\]\s*$/m;
  const nextVersionRe = /^## \[(?!Unreleased)[^\]]+\]/m;
  const start = markdown.search(unreleasedRe);
  if (start < 0) {
    throw new Error('Missing ## [Unreleased] section');
  }
  const afterHeader = markdown.indexOf('\n', start) + 1;
  const rest = markdown.slice(afterHeader);
  const endMatch = rest.match(nextVersionRe);
  const body = endMatch ? rest.slice(0, endMatch.index) : rest;

  const sections = new Map();
  let current = null;
  let bullets = [];

  for (const line of body.split('\n')) {
    const h3 = line.match(/^### (.+)\s*$/);
    if (h3) {
      if (current) {
        sections.set(current, bullets.length);
      }
      current = h3[1].trim();
      bullets = [];
      continue;
    }
    if (current && /^- /.test(line)) {
      bullets.push(line);
    }
  }
  if (current) {
    sections.set(current, bullets.length);
  }
  return sections;
}

function compare(chPath, enPath) {
  const zh = fs.readFileSync(chPath, 'utf8');
  const en = fs.readFileSync(enPath, 'utf8');
  const zhSections = extractUnreleasedSections(zh);
  const enSections = extractUnreleasedSections(en);

  const zhKeys = [...zhSections.keys()].sort();
  const enKeys = [...enSections.keys()].sort();
  const errors = [];

  if (zhKeys.join('|') !== enKeys.join('|')) {
    errors.push(
      `subsection title mismatch:\n  zh: ${zhKeys.join(', ')}\n  en: ${enKeys.join(', ')}`,
    );
  }

  for (const key of zhKeys) {
    if (!enSections.has(key)) continue;
    const zhCount = zhSections.get(key);
    const enCount = enSections.get(key);
    if (zhCount !== enCount) {
      errors.push(
        `### ${key}: bullet count zh=${zhCount} en=${enCount}`,
      );
    }
  }

  return errors;
}

function main() {
  const chPath = path.join(repoRoot, 'CHANGELOG.md');
  const enPath = path.join(repoRoot, 'CHANGELOG.en.md');
  const errors = compare(chPath, enPath);
  if (errors.length > 0) {
    console.error('CHANGELOG [Unreleased] parity FAIL:');
    for (const e of errors) console.error(`  - ${e}`);
    process.exit(1);
  }
  console.log('CHANGELOG [Unreleased] parity ok');
}

main();
