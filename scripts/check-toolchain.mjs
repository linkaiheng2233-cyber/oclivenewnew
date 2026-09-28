#!/usr/bin/env node
/** Read-only OCLive static-gate doctor; this is not compiler/linker or release readiness. */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { findNativeExecutable, runRequiredCommand } from './lib/gate-toolchain.mjs';

const defaultRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const samePath = (left, right) => process.platform === 'win32'
  ? path.resolve(left).toLowerCase() === path.resolve(right).toLowerCase()
  : path.resolve(left) === path.resolve(right);

export function inspectProjectToolchain({ repoRoot = defaultRoot, env = process.env } = {}) {
  const report = {
    schema_version: 1, scope: 'oclive-static-gates', repo_root: repoRoot,
    tools: [], gates: [], ok: false,
  };
  const specifications = [
    ['node', process.execPath, /^v(\d+)\.\d+\.\d+/],
    ['rg', 'rg', /^ripgrep \d+\./],
    ['git', 'git', /^git version \d+\./],
    ['cargo', 'cargo', /^cargo \d+\./],
  ];
  for (const [id, command, versionPattern] of specifications) {
    const tool = {
      id, required: true, path: findNativeExecutable(command, { cwd: repoRoot, env }),
      present: false, usable: false, gate_passed: null, version: null, error: null,
    };
    tool.present = tool.path !== null;
    try {
      const probe = runRequiredCommand(command, ['--version'], { cwd: repoRoot, env });
      tool.version = probe.stdout.trim().split(/\r?\n/)[0];
      if (!versionPattern.test(tool.version)) throw new Error(`Unexpected ${id} version output: ${tool.version}`);
      if (id === 'node') {
        const engine = JSON.parse(fs.readFileSync(path.join(repoRoot, 'package.json'), 'utf8')).engines.node;
        const minimum = /^>=(\d+)$/.exec(engine);
        if (!minimum) throw new Error(`Unsupported Node engine expression ${engine}; update the doctor with the engine contract.`);
        if (Number(versionPattern.exec(tool.version)[1]) < Number(minimum[1])) throw new Error(`Node ${tool.version} does not satisfy ${engine}.`);
      }
      tool.usable = true;
    } catch (error) { tool.error = error.message; }
    report.tools.push(tool);
  }

  function gate(id, dependencies, command, args, validate = () => ({})) {
    const blocked = dependencies.filter(dep => !report.tools.find(tool => tool.id === dep).usable);
    const result = { id, dependencies, command: [command, ...args], status: 'NOT_RUN', exit_code: null, details: null, error: null };
    if (blocked.length) {
      result.error = `Blocked by unusable required tools: ${blocked.join(', ')}`;
    } else {
      try {
        const executed = runRequiredCommand(command, args, { cwd: repoRoot, env });
        result.exit_code = executed.status;
        result.details = validate(executed.stdout);
        result.status = 'PASS';
      } catch (error) {
        result.exit_code = error.exitCode ?? null;
        result.status = 'FAIL';
        result.error = error.message;
      }
    }
    report.gates.push(result);
  }

  gate('domain-layering', ['node', 'rg'], process.execPath, [path.join(repoRoot, 'scripts', 'check-domain-layering.mjs')]);
  gate('host-reexport', ['node', 'rg'], process.execPath, [path.join(repoRoot, 'scripts', 'check-host-reexport-imports.mjs')]);
  gate('git-worktree', ['git'], 'git', ['rev-parse', '--show-toplevel'], output => {
    if (!samePath(output.trim(), repoRoot)) throw new Error('Git worktree root differs from the inspected project root.');
    return { worktree_root: output.trim() };
  });
  gate('cargo-workspace', ['cargo'], 'cargo', ['metadata', '--locked', '--offline', '--no-deps', '--format-version', '1'], output => {
    const metadata = JSON.parse(output);
    if (typeof metadata.workspace_root !== 'string' || !samePath(metadata.workspace_root, repoRoot) || !Array.isArray(metadata.packages)) {
      throw new Error('Cargo metadata does not describe the inspected workspace.');
    }
    return { workspace_root: metadata.workspace_root, packages: metadata.packages.length };
  });
  for (const tool of report.tools) {
    const related = report.gates.filter(result => result.dependencies.includes(tool.id));
    tool.gate_passed = related.every(result => result.status === 'PASS') ? true
      : related.some(result => result.status === 'FAIL') ? false : null;
  }
  report.ok = report.tools.every(tool => tool.usable) && report.gates.every(result => result.status === 'PASS');
  return report;
}

function main() {
  if (process.argv.slice(2).some(arg => arg !== '--json')) {
    console.error('Usage: node scripts/check-toolchain.mjs [--json]');
    process.exitCode = 1;
    return;
  }
  const report = inspectProjectToolchain();
  if (process.argv.includes('--json')) console.log(JSON.stringify(report, null, 2));
  else {
    console.log('[toolchain] scope: OCLive static gates (not build/linker/release readiness)');
    for (const tool of report.tools) {
      console.log(`${tool.id}: present=${tool.present} usable=${tool.usable} gate_passed=${tool.gate_passed} (${tool.version ?? 'no valid version'})`);
      if (tool.error) console.error(tool.error);
    }
    for (const result of report.gates) {
      console.log(`${result.id}: ${result.status}`);
      if (result.error) console.error(result.error);
    }
    console.log(`[toolchain] ${report.ok ? 'PASS' : 'FAIL'}`);
  }
  if (!report.ok) process.exitCode = 1;
}

if (process.argv[1] && samePath(process.argv[1], fileURLToPath(import.meta.url))) main();
