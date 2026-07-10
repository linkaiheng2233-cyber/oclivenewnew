#!/usr/bin/env node
/**
 * Prepend Windows SDK rc.exe + MSVC link.exe (x64) to PATH for Tauri on Windows.
 * Usage: node scripts/with-windows-rc-path.mjs <command> [args...]
 */
import { spawn, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

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
  const home = process.env.USERPROFILE || process.env.HOME
  if (!home)
    return
  const cargoBin = path.join(home, '.cargo', 'bin')
  if (fs.existsSync(path.join(cargoBin, 'cargo.exe')) || fs.existsSync(path.join(cargoBin, 'cargo')))
    prefix.push(cargoBin)
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
const env = { ...process.env }
const pathPrefix = [path.dirname(process.execPath)]
prependCargoBin(pathPrefix)
const localBin = path.join(repoRoot, 'node_modules', '.bin')
if (fs.existsSync(localBin))
  pathPrefix.push(localBin)
if (msvcBin)
  pathPrefix.push(msvcBin)
if (rcBin)
  pathPrefix.push(rcBin)
if (pathPrefix.length)
  env.PATH = `${pathPrefix.join(path.delimiter)}${path.delimiter}${env.PATH || ''}`

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

const [cmd, ...args] = process.argv.slice(2)
if (!cmd) {
  console.error('[with-windows-rc-path] missing command')
  process.exit(1)
}

const child = spawn(cmd, args, {
  stdio: 'inherit',
  shell: true,
  env,
})
child.on('exit', (code) => process.exit(code ?? 0))
