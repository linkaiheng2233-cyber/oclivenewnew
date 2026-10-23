#!/usr/bin/env node
// SDK boundary guard: the SDK is a development-time toolkit, never a runtime dependency.
//
// Any Cargo manifest whose runtime dependency name matches *sdk* fails this check. The SDK may
// appear in `dev-dependencies` or repository tooling only. See
// creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md ("SDK boundary: a development-time toolkit").

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SDK_NAME = /(^|[-_])sdk($|[-_])/i;
const RUNTIME_SECTION = /^(dependencies|build-dependencies|target\..+\.(dependencies|build-dependencies))$/;

export function scanManifest(text) {
  const violations = [];
  let section = '';
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.length === 0 || line.startsWith('#')) continue;

    const tableDependency = /^\[(?:target\.[^\]]+\.)?(dependencies|build-dependencies)\.([^\]]+)\]$/.exec(line);
    if (tableDependency) {
      const name = tableDependency[2].trim().replace(/^["']|["']$/g, '');
      if (SDK_NAME.test(name)) violations.push({ name, section: tableDependency[1] });
      continue;
    }

    const header = /^\[([^\]]+)\]$/.exec(line);
    if (header) {
      section = header[1].trim();
      continue;
    }

    if (!RUNTIME_SECTION.test(section)) continue;
    const dependency = /^([A-Za-z0-9_-]+)\s*=/.exec(line);
    if (dependency && SDK_NAME.test(dependency[1])) {
      violations.push({ name: dependency[1], section });
    }
  }
  return violations;
}

function manifests(root) {
  try {
    return execFileSync('git', ['-C', root, 'ls-files'], { encoding: 'utf8' })
      .split('\n')
      .filter(file => file === 'Cargo.toml' || file.endsWith('/Cargo.toml'));
  } catch {
    return [];
  }
}

function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const files = manifests(root);
  if (files.length === 0) {
    console.error('check-sdk-boundary: no Cargo manifests found (is this a git checkout?)');
    process.exitCode = 2;
    return;
  }
  const problems = [];
  for (const file of files) {
    const text = fs.readFileSync(path.join(root, file), 'utf8');
    for (const violation of scanManifest(text)) {
      problems.push(`${file}: ${violation.name} in [${violation.section}]`);
    }
  }
  if (problems.length > 0) {
    for (const problem of problems) console.error(`  - ${problem}`);
    console.error(`check-sdk-boundary: FAIL (${problems.length} runtime SDK dependenc${problems.length === 1 ? 'y' : 'ies'} across ${files.length} manifests)`);
    process.exitCode = 1;
    return;
  }
  console.log(`check-sdk-boundary: ok (${files.length} manifests, no runtime SDK dependency)`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
