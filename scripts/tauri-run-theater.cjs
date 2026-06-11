/**
 * Theater Tauri beforeDev/beforeBuild — sets VITE_OCLIVE_SHELL=theater for Vite.
 */
const path = require('path')
const { spawn, spawnSync } = require('child_process')

const repoRoot = path.join(__dirname, '..')
const script = process.argv[2] === 'build' ? 'build' : 'dev'

process.env.VITE_OCLIVE_SHELL = 'theater'

function freeVitePortIfBusy() {
  if (script !== 'dev') return
  if (process.env.OCLIVE_DEV_FREE_PORT === '0') return
  if (process.platform !== 'win32') return

  const query = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-Command',
      "(Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique) -join ' '",
    ],
    { encoding: 'utf8', shell: false },
  )

  const pidText = (query.stdout || '').trim()
  if (!pidText) return

  const pids = pidText
    .split(/\s+/)
    .map(s => Number(s))
    .filter(n => Number.isInteger(n) && n > 0 && n !== process.pid)

  for (const pid of pids) {
    console.warn(`[tauri-run-theater] port 1420 busy, stopping PID ${pid}`)
    spawnSync('taskkill', ['/PID', String(pid), '/T', '/F'], {
      stdio: 'inherit',
      shell: false,
    })
  }
}

freeVitePortIfBusy()

const child = spawn('npm', ['run', script], {
  cwd: repoRoot,
  stdio: 'inherit',
  shell: true,
  env: { ...process.env, VITE_OCLIVE_SHELL: 'theater' },
})
child.on('exit', code => process.exit(code ?? 0))
