import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { collectMarkdownFiles, collectTrackedRoleJsonFiles } from './check-doc-encoding.mjs';

const script = fileURLToPath(new URL('./check-doc-encoding.mjs', import.meta.url));

function removeFixture(dir) {
  const root = path.resolve(os.tmpdir());
  const target = path.resolve(dir);
  assert.ok(target.startsWith(`${root}${path.sep}oclive-doc-`));
  fs.rmSync(target, { recursive: true, force: true });
}

function runFixture(bytes, extension = 'md') {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-doc-encoding-'));
  try {
    const file = path.join(dir, `sample.${extension}`);
    fs.writeFileSync(file, bytes);
    return spawnSync(process.execPath, [script, '--file', file], { encoding: 'utf8' });
  } finally {
    removeFixture(dir);
  }
}

test('accepts normal Chinese and English Markdown', () => {
  const result = runFixture(Buffer.from('# 标题\nEnglish text\n', 'utf8'));
  assert.equal(result.status, 0, result.stderr);
});

test('rejects damaged role JSON bytes', () => {
  const result = runFixture(Buffer.from('{"reply":"???"}\n'), 'json');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /sample\.json: three or more consecutive question marks/);
});

test('scans tracked role JSON without reading untracked runtime chats', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-doc-tree-'));
  try {
    const tracked = path.join(dir, 'distros/chat-pro/roles/demo/config.json');
    const runtime = path.join(dir, 'distros/chat-pro/roles/.oclive_directory_plugin_data/chats/demo/session.json');
    fs.mkdirSync(path.dirname(tracked), { recursive: true });
    fs.mkdirSync(path.dirname(runtime), { recursive: true });
    fs.writeFileSync(tracked, '{"name":"demo"}\n');
    fs.writeFileSync(runtime, '{"reply":"???"}\n');
    execFileSync('git', ['init', '--quiet'], { cwd: dir });
    execFileSync('git', ['-c', 'core.autocrlf=false', 'add', '--', 'distros/chat-pro/roles/demo/config.json'], { cwd: dir });
    assert.deepEqual(collectTrackedRoleJsonFiles(dir), [tracked]);
  } finally {
    removeFixture(dir);
  }
});

for (const [name, bytes, diagnostic] of [
  ['BOM', Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), Buffer.from('标题')]), 'UTF-8 BOM'],
  ['invalid UTF-8', Buffer.from([0x23, 0x20, 0xc3, 0x28]), 'invalid UTF-8'],
  ['replacement character', Buffer.from('# 标题\ufffd\n'), 'replacement character U+FFFD'],
  ['ASCII substitution', Buffer.from('# ???\n'), 'three or more consecutive question marks'],
]) {
  test(`rejects ${name} with a file-specific diagnostic`, () => {
    const result = runFixture(bytes);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /sample\.md:/);
    assert.ok(result.stderr.includes(diagnostic), result.stderr);
  });
}

test('default scan excludes historical archive directories', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-doc-tree-'));
  try {
    for (const root of ['handoff', 'creator-docs', 'human-docs']) {
      fs.mkdirSync(path.join(dir, root, 'archive'), { recursive: true });
      fs.writeFileSync(path.join(dir, root, 'active.md'), '# 标题\n');
      fs.writeFileSync(path.join(dir, root, 'archive', 'history.md'), '# ???\n');
    }
    const files = collectMarkdownFiles(dir);
    assert.equal(files.length, 3);
    assert.ok(files.every((file) => file.endsWith('active.md')));
  } finally {
    removeFixture(dir);
  }
});
