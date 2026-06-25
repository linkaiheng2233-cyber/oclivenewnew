#!/usr/bin/env node
/**
 * G14/G16 doc gates:
 * 1. Every handoff/ root *.md (excl. archive/ + subdirs) must appear in handoff/README.md.
 * 2. CANONICAL_BLOCKS sentinels must only exist in their SSOT files (handoff/ + creator-docs/).
 *
 * Usage: node scripts/check-doc-registry.mjs
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const handoffDir = path.join(repoRoot, 'handoff');
const readmePath = path.join(handoffDir, 'README.md');

/** Stable table-header sentinels — duplicate only in listed SSOT (posix path from repo root). */
const CANONICAL_BLOCKS = [
  {
    label: '六槽 × backend 24 格矩阵',
    sentinel: '| 槽 | builtin | remote | directory | none |',
    ssot: 'handoff/SLOT_BACKEND_REALITY_MATRIX.md',
  },
  {
    label: '记忆三套存储 Store 表',
    sentinel: '| Store | Location | Purpose |',
    ssot: 'handoff/CHAT_STORAGE_ARCHITECTURE.md',
  },
  {
    label: 'MODULE_MAP 四条铁律表',
    sentinel: '| # | 铁律 | 一句话 |',
    ssot: 'handoff/MODULE_MAP_AND_HANDOFF.md',
  },
  {
    label: 'MODULE_MAP 模块四大类 / 六槽边界',
    sentinel: '| 大类 | 占 `plugin_backends` 六键？ | 编号 | 改动的文档 SSOT |',
    ssot: 'handoff/MODULE_MAP_AND_HANDOFF.md',
  },
  {
    label: 'CHAT_STORAGE hybrid 后端对照表',
    sentinel: '| Config | SQLite chat tables | JSON mirror under `{app_data}/chats/` | Search | Auto cleanup | Memory replay |',
    ssot: 'handoff/CHAT_STORAGE_ARCHITECTURE.md',
  },
];

const SCAN_DIRS = ['handoff', 'creator-docs'];

function listHandoffRootMarkdown() {
  return fs
    .readdirSync(handoffDir)
    .filter((name) => name.endsWith('.md') && name !== 'README.md')
    .sort();
}

function checkRegistry() {
  const readme = fs.readFileSync(readmePath, 'utf8');
  const missing = [];

  for (const file of listHandoffRootMarkdown()) {
    if (!readme.includes(file)) {
      missing.push(file);
    }
  }

  if (missing.length === 0) {
    console.log(`doc registry ok (${listHandoffRootMarkdown().length} handoff root files)`);
    return [];
  }

  const errors = [`${missing.length} handoff root .md not listed in handoff/README.md:`];
  for (const file of missing) {
    errors.push(`  - ${file}`);
    errors.push(
      `    template: | [${file.replace(/\.md$/, '')}](${file}) | <one-line purpose> |`,
    );
  }
  return errors;
}

function walkMarkdownFiles(dirAbs, relPrefix, out) {
  for (const entry of fs.readdirSync(dirAbs, { withFileTypes: true })) {
    if (entry.name === 'archive') continue;
    const abs = path.join(dirAbs, entry.name);
    const rel = relPrefix ? `${relPrefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      walkMarkdownFiles(abs, rel, out);
      continue;
    }
    if (entry.name.endsWith('.md')) {
      out.push({ abs, rel: rel.replace(/\\/g, '/') });
    }
  }
}

function collectScanFiles() {
  const files = [];
  for (const dir of SCAN_DIRS) {
    walkMarkdownFiles(path.join(repoRoot, dir), dir, files);
  }
  return files;
}

function checkCanonicalBlocks() {
  const files = collectScanFiles();
  const errors = [];

  for (const block of CANONICAL_BLOCKS) {
    const hits = [];
    for (const { abs, rel } of files) {
      const content = fs.readFileSync(abs, 'utf8');
      if (content.includes(block.sentinel)) {
        hits.push(rel);
      }
    }
    const foreign = hits.filter((h) => h !== block.ssot);
    if (foreign.length > 0) {
      errors.push(
        `CANONICAL_BLOCK "${block.label}" sentinel found outside SSOT (${block.ssot}):`,
      );
      for (const f of foreign) {
        errors.push(`  - ${f}`);
      }
    }
  }

  if (errors.length === 0) {
    console.log(`canonical blocks ok (${CANONICAL_BLOCKS.length} sentinels)`);
  }
  return errors;
}

function main() {
  const errors = [...checkRegistry(), ...checkCanonicalBlocks()];
  if (errors.length > 0) {
    console.error('check-doc-registry FAIL:');
    for (const e of errors) console.error(e);
    process.exit(1);
  }
}

main();
