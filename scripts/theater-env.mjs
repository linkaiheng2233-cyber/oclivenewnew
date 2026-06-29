#!/usr/bin/env node
/**
 * Cross-platform theater dev/build launcher.
 * Usage: node scripts/theater-env.mjs dev|build
 */
import { spawn, spawnSync } from 'child_process'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { cargoTargetDir, kernelExeName } from './lib/e2e-binary.mjs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const confPath = path.join(repoRoot, 'distros', 'desktop-tauri', 'tauri.conf.json')
const mode = process.argv[2] === 'build' ? 'build' : 'dev'

function runNodeScript(name) {
  const script = path.join(__dirname, name)
  const r = spawnSync(process.execPath, [script], { cwd: repoRoot, stdio: 'inherit' })
  if (r.status !== 0)
    process.exit(r.status ?? 1)
}

/** Dev spawn uses Env-tier binary (rank 0) so stale bundled resources are not picked first. */
function ensureDevKernelBinary() {
  console.log('[theater-env] building oclive_kernel_server (debug) for :8420...')
  const build = spawnSync('cargo', ['build', '-p', 'oclive_kernel_server'], {
    cwd: repoRoot,
    stdio: 'inherit',
  })
  if (build.status !== 0)
    process.exit(build.status ?? 1)
  const bin = path.join(cargoTargetDir(repoRoot), 'debug', kernelExeName())
  if (!fs.existsSync(bin))
    throw new Error(`kernel binary not found: ${bin}`)
  console.log(`[theater-env] OCLIVE_KERNEL_BINARY=${bin}`)
  return bin
}

function patchTauriRolesForTheater() {
  const original = fs.readFileSync(confPath, 'utf8')
  const parsed = JSON.parse(original)
  const resources = parsed.tauri?.bundle?.resources
  if (!Array.isArray(resources))
    throw new Error('tauri.conf.json: bundle.resources missing')

  const ROLE_PATHS = new Set(['../roles', '../chat-pro/roles', 'resources/roles'])
  const nextResources = resources.filter((entry) => !ROLE_PATHS.has(entry))
  nextResources.unshift('resources/roles')
  if (!nextResources.includes('resources/theater'))
    nextResources.push('resources/theater')

  parsed.tauri.bundle.resources = nextResources
  fs.writeFileSync(confPath, `${JSON.stringify(parsed, null, 2)}\n`, 'utf8')
  return original
}

if (mode === 'build') {
  runNodeScript('filter-theater-roles.mjs')
  runNodeScript('bundle-kernel-for-tauri.mjs')
}

const env = {
  ...process.env,
  OCLIVE_DISTRO_PROFILE: path.join(repoRoot, 'examples', 'distro-profiles', 'theater.oclive.toml'),
  VITE_OCLIVE_SHELL: 'theater',
  OCLIVE_TAURI_SHELL: 'theater',
}

if (mode === 'dev') {
  env.OCLIVE_KERNEL_BINARY = ensureDevKernelBinary()
  env.OCLIVE_DEVELOPER = '1'
  env.OCLIVE_THEATER_CAST_REWRITE_TIMEOUT_SECS = '45'
}

const npmArgs = mode === 'build' ? ['run', 'tauri:build'] : ['run', 'tauri:dev']
let restoreConf = null

if (mode === 'build')
  restoreConf = patchTauriRolesForTheater()

const child = spawn('npm', npmArgs, {
  cwd: repoRoot,
  env,
  stdio: 'inherit',
  shell: true,
})

child.on('exit', (code) => {
  if (restoreConf)
    fs.writeFileSync(confPath, restoreConf, 'utf8')
  process.exit(code ?? 0)
})
