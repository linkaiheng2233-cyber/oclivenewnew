/**
 * Stable dev: Vite stays in its own process; Tauri reuses http://localhost:1420.
 * Usage: npm run tauri:dev:split
 */
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { freeVitePortIfBusyUnlessHealthy, probeViteDevOk } from './lib/vite-dev-port.mjs'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.join(scriptDir, '..')
const waitScript = path.join(scriptDir, 'tauri-dev-wait-vite.mjs')

const reuseVite = probeViteDevOk(repoRoot)
/** @type {import('node:child_process').ChildProcess | null} */
let vite = null

if (reuseVite) {
  console.info('[tauri-dev-split] reusing existing Vite on http://localhost:1420/')
}
else {
  freeVitePortIfBusyUnlessHealthy(repoRoot)
  console.info('[tauri-dev-split] starting Vite + Tauri (split processes)')
  vite = spawn('npm', ['run', 'dev:web'], {
    cwd: repoRoot,
    stdio: 'inherit',
    shell: true,
  })
}

const tauri = spawn(process.execPath, [waitScript], {
  cwd: repoRoot,
  stdio: 'inherit',
  shell: false,
})

let exiting = false

function shutdown(signal) {
  if (exiting)
    return
  exiting = true
  if (signal)
    console.info(`[tauri-dev-split] ${signal} — stopping Tauri${vite ? ' and Vite' : ''}`)
  if (vite)
    vite.kill('SIGTERM')
  tauri.kill('SIGTERM')
}

process.on('SIGINT', () => shutdown('SIGINT'))
process.on('SIGTERM', () => shutdown('SIGTERM'))

if (vite) {
  vite.on('exit', (code) => {
    if (exiting)
      return
    exiting = true
    tauri.kill('SIGTERM')
    process.exit(code ?? 0)
  })
}

tauri.on('exit', (code) => {
  if (exiting)
    return
  exiting = true
  if (vite)
    vite.kill('SIGTERM')
  process.exit(code ?? 0)
})
