/**
 * Shared Vite dev port helpers for tauri:dev / tauri:dev:split.
 */
import { spawnSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptsDir = path.dirname(fileURLToPath(import.meta.url))

export function probeViteDevOk(repoRoot) {
  const probeScript = path.join(scriptsDir, '..', 'dev-probe.mjs')
  const r = spawnSync(process.execPath, [probeScript], {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: false,
    timeout: 8000,
  })
  return r.status === 0
}

/** Kill stale listeners on :1420 when probe fails (Windows only). */
export function freeVitePortIfBusyUnlessHealthy(repoRoot) {
  if (process.env.OCLIVE_DEV_FREE_PORT === '0')
    return
  if (probeViteDevOk(repoRoot))
    return
  if (process.platform !== 'win32')
    return

  const query = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-Command',
      '(Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique) -join \' \'',
    ],
    { encoding: 'utf8', shell: false },
  )

  const pidText = (query.stdout || '').trim()
  if (!pidText)
    return

  const pids = pidText
    .split(/\s+/)
    .map(s => Number(s))
    .filter(n => Number.isInteger(n) && n > 0 && n !== process.pid)

  for (const pid of pids) {
    console.warn(`[vite-dev-port] port 1420 busy but probe failed — stopping PID ${pid}`)
    spawnSync('taskkill', ['/PID', String(pid), '/T', '/F'], {
      stdio: 'inherit',
      shell: false,
    })
  }
}
