#!/usr/bin/env node
/**
 * Prepend Windows SDK rc.exe + MSVC link.exe (x64) to PATH for Tauri on Windows.
 * Usage: node scripts/with-windows-rc-path.mjs <command> [args...]
 */
import { spawn, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { resolveChatProDevRuntimeEnv } from './lib/dev-performance-runtime.mjs'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const [cmd, ...args] = process.argv.slice(2)

if (!cmd) {
  console.error('[with-windows-rc-path] missing command')
  process.exit(1)
}

function isChatProDevLaunch(command, commandArgs) {
  if (path.basename(command).toLowerCase() === 'tauri' && commandArgs[0] === 'dev')
    return true
  return commandArgs.some(arg => path.basename(arg).toLowerCase() === 'tauri-dev-split.mjs')
}

function findWindowsRcBinDir() {
  if (process.platform !== 'win32')
    return null
  const pf = process.env['ProgramFiles(x86)']
  if (!pf)
    return null
  const binRoot = path.join(pf, 'Windows Kits', '10', 'bin')
  if (!fs.existsSync(binRoot))
    return null
  const versions = fs
    .readdirSync(binRoot, { withFileTypes: true })
    .filter((e) => e.isDirectory() && /^\d/.test(e.name))
    .map((e) => e.name)
    .sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
  for (const ver of versions) {
    const x64 = path.join(binRoot, ver, 'x64')
    if (fs.existsSync(path.join(x64, 'rc.exe')))
      return x64
  }
  return null
}

function findMsvcLinkBinDir() {
  if (process.platform !== 'win32')
    return null
  const pf = process.env['ProgramFiles(x86)']
  if (!pf)
    return null
  const vswhere = path.join(pf, 'Microsoft Visual Studio', 'Installer', 'vswhere.exe')
  if (!fs.existsSync(vswhere))
    return null
  const r = spawnSync(vswhere, [
    '-latest',
    '-products', '*',
    '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
    '-property', 'installationPath',
  ], { encoding: 'utf8' })
  const installPath = (r.stdout || '').trim()
  if (!installPath)
    return null
  const msvcRoot = path.join(installPath, 'VC', 'Tools', 'MSVC')
  if (!fs.existsSync(msvcRoot))
    return null
  const versions = fs
    .readdirSync(msvcRoot, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
  for (const ver of versions) {
    const linkDir = path.join(msvcRoot, ver, 'bin', 'Hostx64', 'x64')
    if (fs.existsSync(path.join(linkDir, 'link.exe')))
      return linkDir
  }
  return null
}

function prependCargoBin(prefix) {
  const candidates = []
  if (process.env.CARGO_HOME)
    candidates.push(process.env.CARGO_HOME)
  const home = process.env.USERPROFILE || process.env.HOME
  if (home)
    candidates.push(path.join(home, '.cargo'))
  for (const cargoHome of candidates) {
    const cargoBin = path.join(cargoHome, 'bin')
    if (fs.existsSync(path.join(cargoBin, 'cargo.exe')) || fs.existsSync(path.join(cargoBin, 'cargo'))) {
      prefix.push(cargoBin)
      return
    }
  }
}

function pathHasExecutable(pathEnv, name) {
  for (const dir of (pathEnv || '').split(path.delimiter)) {
    if (!dir)
      continue
    try {
      if (fs.existsSync(path.join(dir, name)))
        return true
    }
    catch {
      // ignore unreadable dirs
    }
  }
  return false
}

const rcBin = findWindowsRcBinDir()
const msvcBin = findMsvcLinkBinDir()
const devRuntime = isChatProDevLaunch(cmd, args)
  ? resolveChatProDevRuntimeEnv(repoRoot)
  : { env: { ...process.env }, inferredRuntimePath: null }
const env = devRuntime.env
if (devRuntime.inferredRuntimePath) {
  console.info(
    `[with-windows-rc-path] using workspace llama-server: ${devRuntime.inferredRuntimePath}`,
  )
}
const pathPrefix = [path.dirname(process.execPath)]
prependCargoBin(pathPrefix)
const localBin = path.join(repoRoot, 'node_modules', '.bin')
if (fs.existsSync(localBin))
  pathPrefix.push(localBin)
if (msvcBin)
  pathPrefix.push(msvcBin)
if (rcBin)
  pathPrefix.push(rcBin)
if (pathPrefix.length) {
  // Windows env blocks may hold PATH under arbitrary casing. Normalize to one
  // key without clobbering the inherited toolchain path.
  const pathKeys = Object.keys(env).filter(key => key.toLowerCase() === 'path')
  const existingPath = pathKeys
    .map(key => env[key])
    .find(value => typeof value === 'string' && value.length > 0) ?? ''
  for (const key of pathKeys)
    delete env[key]
  env.PATH = `${pathPrefix.join(path.delimiter)}${path.delimiter}${existingPath}`
}

if (process.platform === 'win32') {
  if (!rcBin) {
    console.error(
      '[with-windows-rc-path] rc.exe not found. Install Windows SDK, e.g.:\n'
      + '  winget install Microsoft.WindowsSDK.10.0.26100',
    )
    process.exit(1)
  }
  if (!pathHasExecutable(env.PATH, 'link.exe')) {
    console.error(
      '[with-windows-rc-path] link.exe (MSVC) not found. Install Visual Studio Build Tools:\n'
      + '  winget install Microsoft.VisualStudio.2022.BuildTools --override '
      + '"--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"\n'
      + 'See human-docs/10_SETUP_WINDOWS.md',
    )
    process.exit(1)
  }
}

const child = spawn(cmd, args, {
  stdio: 'inherit',
  shell: true,
  env,
})
child.on('exit', (code) => process.exit(code ?? 0))
