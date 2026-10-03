#!/usr/bin/env node
/** Reject known silent text corruption in active docs and tracked role JSON. */
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const activeRoots = ['handoff', 'creator-docs', 'human-docs'];
const roleJsonRoot = 'distros/chat-pro/roles';
const decoder = new TextDecoder('utf-8', { fatal: true });

export function collectMarkdownFiles(root = repoRoot) {
  const files = [];
  function visit(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === 'archive') continue;
      const target = path.join(dir, entry.name);
      if (entry.isSymbolicLink()) {
        throw new Error(`symbolic link in active docs: ${path.relative(root, target)}`);
      }
      if (entry.isDirectory()) visit(target);
      else if (entry.isFile() && entry.name.endsWith('.md')) files.push(target);
    }
  }
  for (const rel of activeRoots) visit(path.join(root, rel));
  return files.sort();
}

export function collectTrackedRoleJsonFiles(root = repoRoot) {
  // Git's index excludes local chat history and other untracked runtime data.
  const listed = execFileSync('git', ['ls-files', '-z', '--', roleJsonRoot], { cwd: root });
  const files = [];
  for (const relative of listed.toString('utf8').split('\0')) {
    if (!relative.startsWith(`${roleJsonRoot}/`) || !relative.endsWith('.json')) continue;
    const segments = relative.split('/');
    if (segments.some((segment) => !segment || segment === '.' || segment === '..')) {
      throw new Error(`invalid tracked role path: ${relative}`);
    }
    let target = root;
    for (const segment of segments) {
      target = path.join(target, segment);
      if (fs.lstatSync(target).isSymbolicLink()) {
        throw new Error(`symbolic link in tracked role JSON: ${relative}`);
      }
    }
    files.push(target);
  }
  return files.sort();
}

export function checkDocumentBytes(bytes) {
  const problems = [];
  if (bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    problems.push('UTF-8 BOM');
  }
  let content;
  try {
    content = decoder.decode(bytes);
  } catch {
    problems.push('invalid UTF-8');
    return problems;
  }
  if (content.includes('\ufffd')) problems.push('replacement character U+FFFD');
  if (/\?{3,}/u.test(content)) problems.push('three or more consecutive question marks');
  return problems;
}

function main(args) {
  const files = [];
  if (args.length === 0) files.push(...collectMarkdownFiles(), ...collectTrackedRoleJsonFiles());
  else {
    for (let i = 0; i < args.length; i += 2) {
      if (args[i] !== '--file' || !args[i + 1]) {
        throw new Error('usage: node scripts/check-doc-encoding.mjs [--file PATH ...]');
      }
      files.push(path.resolve(args[i + 1]));
    }
  }

  const failures = [];
  for (const file of files) {
    const label = path.relative(repoRoot, file) || file;
    for (const problem of checkDocumentBytes(fs.readFileSync(file))) {
      failures.push(`${label}: ${problem}`);
    }
  }
  if (failures.length > 0) {
    console.error(`doc encoding FAIL (${failures.length} findings):`);
    for (const failure of failures) console.error(`  ${failure}`);
    process.exitCode = 1;
  } else {
    console.log(`doc encoding ok (${files.length} active Markdown and tracked role JSON files)`);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`doc encoding FAIL: ${error.message}`);
    process.exitCode = 1;
  }
}
