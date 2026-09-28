import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const remediation = {
  rg: 'Install ripgrep or expose its native executable on PATH; verify rg --version, then rerun npm run check:toolchain.',
  git: 'Expose Git on PATH; verify git --version, then rerun npm run check:toolchain.',
  cargo: 'Expose the Rust toolchain on PATH; verify cargo --version, then rerun npm run check:toolchain.',
  node: 'Use the Node version required by package.json; verify node --version, then rerun npm run check:toolchain.',
};

export class GateToolchainError extends Error {
  constructor(code, command, message, result = {}) {
    const tool = path.basename(command).replace(/\.exe$/i, '');
    const detail = result.stderr?.trim() || result.stdout?.trim();
    super(`[toolchain:${code}] ${message}${detail ? `\n${detail.slice(0, 4000)}` : ''}\n${remediation[tool] ?? 'Verify the command and rerun the failed gate.'}`);
    this.name = 'GateToolchainError';
    this.code = code;
    this.command = command;
    this.exitCode = result.status ?? null;
    this.signal = result.signal ?? null;
    this.stdout = result.stdout ?? '';
    this.stderr = result.stderr ?? '';
  }
}

// Only native programs are used here. No shell, npm shim, installation or PATH mutation.
export function findNativeExecutable(command, { env = process.env, cwd = process.cwd() } = {}) {
  const isFile = candidate => {
    try { return fs.statSync(candidate).isFile(); } catch { return false; }
  };
  if (path.isAbsolute(command) || command.includes('/') || command.includes('\\')) {
    const candidate = path.resolve(cwd, command);
    return isFile(candidate) ? candidate : null;
  }
  const pathKey = Object.keys(env).find(key => process.platform === 'win32' ? key.toUpperCase() === 'PATH' : key === 'PATH');
  const directories = (env[pathKey] ?? '').split(path.delimiter).filter(Boolean);
  const names = process.platform === 'win32' && !/\.(exe|com)$/i.test(command)
    ? [`${command}.exe`, `${command}.com`]
    : [command];
  for (const directory of directories) {
    for (const name of names) {
      const candidate = path.resolve(cwd, directory.replace(/^"(.*)"$/, '$1'), name);
      if (isFile(candidate)) return candidate;
    }
  }
  return null;
}

export function runRequiredCommand(command, args, {
  cwd = process.cwd(), env = process.env, acceptedExitCodes = [0], timeout = 30000,
} = {}) {
  const executable = findNativeExecutable(command, { cwd, env });
  if (!executable) {
    throw new GateToolchainError('MISSING', command, `Required executable ${command} was not found on PATH.`);
  }
  const result = spawnSync(executable, args, {
    cwd, env, encoding: 'utf8', shell: false, windowsHide: true,
    timeout, maxBuffer: 8 * 1024 * 1024,
  });
  if (result.error || result.signal || !acceptedExitCodes.includes(result.status)) {
    const code = result.error?.code === 'ETIMEDOUT' ? 'TIMEOUT' : result.error ? 'EXECUTION' : result.signal ? 'SIGNAL' : 'EXIT';
    throw new GateToolchainError(code, command,
      `${command} failed (${result.error?.code ?? result.signal ?? `exit ${result.status}`}): ${args.join(' ')}`, result);
  }
  return { ...result, executable };
}

function fileIdentity(file, cwd) {
  if (/^(?:[a-z]:[\\/]|\\\\)/i.test(file)) return path.win32.normalize(file).toLowerCase();
  const resolved = path.resolve(cwd, file);
  return process.platform === 'win32' ? resolved.toLowerCase() : resolved;
}

// rg --count counts matching lines, not occurrences. --null keeps drive colons
// and even newlines in file names separate from the numeric count.
export function parseRipgrepCounts(output, { cwd = process.cwd(), excludeFiles = [] } = {}) {
  const excluded = new Set(excludeFiles.map(file => fileIdentity(file, cwd)));
  let offset = 0;
  let total = 0;
  while (offset < output.length) {
    const separator = output.indexOf('\0', offset);
    const file = output.slice(offset, separator);
    const count = separator >= 0 ? /^([1-9]\d*)\r?\n/.exec(output.slice(separator + 1)) : null;
    const number = count ? Number(count[1]) : NaN;
    if (separator < 0 || !file || !Number.isSafeInteger(number)) {
      throw new GateToolchainError('OUTPUT', 'rg', 'Malformed rg --count --null output; refusing to treat it as zero matches.');
    }
    if (!excluded.has(fileIdentity(file, cwd))) {
      total += number;
      if (!Number.isSafeInteger(total)) throw new GateToolchainError('OUTPUT', 'rg', 'Ripgrep count exceeds the safe integer range.');
    }
    offset = separator + 1 + count[0].length;
  }
  return total;
}

export function countMatchingLines(pattern, targets, { cwd = process.cwd(), env = process.env, excludeFiles = [] } = {}) {
  const result = runRequiredCommand('rg', [
    '--glob', '*.rs', '--count', '--with-filename', '--null', '--', pattern, ...targets,
  ], { cwd, env, acceptedExitCodes: [0, 1] });
  if (result.status === 1 && result.stdout === '') return 0;
  if (result.status !== 0 || !result.stdout) {
    throw new GateToolchainError('OUTPUT', 'rg', 'Ripgrep exit status contradicts its count output.', result);
  }
  return parseRipgrepCounts(result.stdout, { cwd, excludeFiles });
}

export function runGate(main) {
  try { main(); } catch (error) {
    if (!(error instanceof GateToolchainError)) throw error;
    console.error(error.message);
    process.exitCode = 1;
  }
}
