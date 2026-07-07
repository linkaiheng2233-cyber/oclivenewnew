/**
 * Wait for Vite on :1420, then start Tauri without spawning a second Vite.
 */
import { spawn, spawnSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.join(scriptDir, '..')
const probeScript = path.join(scriptDir, 'dev-probe.mjs')
const deadline = Date.now() + Number(process.env.OCLIVE_VITE_WAIT_MS ?? '120000')

function probe() {
  const r = spawnSync(process.execPath, [probeScript], {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: false,
    timeout: 8000,
  })
  return r.status === 0
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function main() {
  while (!probe()) {
    if (Date.now() > deadline) {
      console.error('[tauri-dev-wait-vite] timed out waiting for http://localhost:1420')
      process.exit(1)
    }
    await sleep(400)
  }

  const child = spawn(
    process.execPath,
    [
      path.join(scriptDir, 'with-windows-rc-path.mjs'),
      'tauri',
      'dev',
      '--config',
      'distros/desktop-tauri/tauri.conf.json',
    ],
    {
      cwd: repoRoot,
      stdio: 'inherit',
      shell: false,
      env: {
        ...process.env,
        OCLIVE_TAURI_REUSE_VITE: '1',
      },
    },
  )
  child.on('exit', (code) => process.exit(code ?? 0))
}

void main()
