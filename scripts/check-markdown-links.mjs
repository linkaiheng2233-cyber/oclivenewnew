#!/usr/bin/env node
/**
 * Validate local Markdown links relative to the file that contains them.
 *
 * The default scope is the human module start packs plus the critical AI/SSOT
 * anchors: these are high-traffic surfaces and should never send a contributor
 * to a missing SSOT.
 * Additional files/directories may be passed as positional arguments.
 *
 * Usage:
 *   node scripts/check-markdown-links.mjs
 *   node scripts/check-markdown-links.mjs human-docs creator-docs
 *   node scripts/check-markdown-links.mjs --self-test
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const DEFAULT_TARGETS = [
  'human-docs/modules',
  'human-docs-en/modules',
  'AGENTS.md',
  'handoff/AI_READING_INDEX.md',
  'handoff/AI_CHANGE_BOUNDARIES.md',
  'handoff/MODULE_MAP_AND_HANDOFF.md',
  'handoff/BUS_FACTOR_NOTES.md',
  'handoff/README.md',
  'creator-docs/NAMING_CONVENTIONS.md',
  'creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md',
];
const SKIPPED_SCHEMES = /^(?:https?:|mailto:|data:|app:|vscode:|file:)/i;

function listMarkdownFiles(targets, root = repoRoot) {
  const files = [];

  function visit(absolutePath) {
    if (!fs.existsSync(absolutePath)) {
      throw new Error(`scan target does not exist: ${path.relative(root, absolutePath)}`);
    }
    const stat = fs.statSync(absolutePath);
    if (stat.isFile()) {
      if (absolutePath.endsWith('.md')) files.push(absolutePath);
      return;
    }
    for (const entry of fs.readdirSync(absolutePath, { withFileTypes: true })) {
      if (
        entry.name === 'archive' ||
        entry.name === 'node_modules' ||
        entry.name === 'target' ||
        entry.name === 'dist' ||
        entry.name === '.git' ||
        entry.name.startsWith('.venv')
      ) continue;
      visit(path.join(absolutePath, entry.name));
    }
  }

  for (const target of targets) visit(path.resolve(root, target));
  return files.sort();
}

function stripFencedCode(markdown) {
  return markdown.replace(/^\s*(```|~~~)[^\n]*\n[\s\S]*?^\s*\1\s*$/gm, '');
}

function linkDestination(rawDestination) {
  const raw = rawDestination.trim();
  if (!raw) return '';
  if (raw.startsWith('<')) {
    const end = raw.indexOf('>');
    return end === -1 ? raw : raw.slice(1, end);
  }
  return raw.split(/\s+["']/u, 1)[0];
}

function localLinkErrors(file, root = repoRoot) {
  const markdown = stripFencedCode(fs.readFileSync(file, 'utf8'));
  const errors = [];
  const linkPattern = /!?\[[^\]]*\]\(([^)]*)\)/gu;

  for (const match of markdown.matchAll(linkPattern)) {
    let destination = linkDestination(match[1]);
    if (!destination || destination.startsWith('#') || SKIPPED_SCHEMES.test(destination)) {
      continue;
    }
    destination = destination.split('#', 1)[0].split('?', 1)[0];
    if (!destination) continue;

    try {
      destination = decodeURIComponent(destination);
    } catch {
      errors.push(`${path.relative(root, file)}: invalid URL encoding in ${destination}`);
      continue;
    }

    const resolved = path.resolve(path.dirname(file), destination);
    if (!fs.existsSync(resolved)) {
      errors.push(
        `${path.relative(root, file)}: ${destination} -> ${path.relative(root, resolved)} (missing)`,
      );
    }
  }
  return errors;
}

function checkTargets(targets, root = repoRoot) {
  const files = listMarkdownFiles(targets, root);
  const errors = files.flatMap((file) => localLinkErrors(file, root));
  return { files, errors };
}

function selfTest() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-md-links-'));
  try {
    fs.mkdirSync(path.join(tmp, 'docs'));
    fs.writeFileSync(path.join(tmp, 'target.md'), '# target\n');
    fs.writeFileSync(
      path.join(tmp, 'docs', 'ok.md'),
      '[relative](../target.md#section)\n[external](https://example.com)\n',
    );
    let result = checkTargets(['docs'], tmp);
    if (result.errors.length !== 0) {
      throw new Error(`valid fixture rejected: ${result.errors.join('; ')}`);
    }

    fs.writeFileSync(path.join(tmp, 'docs', 'broken.md'), '[missing](../missing.md)\n');
    result = checkTargets(['docs'], tmp);
    if (result.errors.length !== 1 || !result.errors[0].includes('missing.md')) {
      throw new Error('missing-link fixture was not detected exactly once');
    }
    console.log('check-markdown-links self-test: PASS');
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

const args = process.argv.slice(2);
if (args.includes('--self-test')) {
  selfTest();
} else {
  const targets = args.length > 0 ? args : DEFAULT_TARGETS;
  const { files, errors } = checkTargets(targets);
  if (errors.length > 0) {
    console.error(`check-markdown-links: FAIL (${errors.length} broken local links)`);
    for (const error of errors) console.error(`  - ${error}`);
    process.exit(1);
  }
  console.log(`check-markdown-links: PASS (${files.length} Markdown files)`);
}
