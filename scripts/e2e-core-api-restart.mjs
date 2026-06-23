/**
 * A1.1 PoC — core HTTP API path with process restart (no Ollama; MOCK_LLM).
 * Flow: start --api → GET /health → POST /chat → SIGTERM → start again → health → chat → stop.
 *
 * Env:
 *   OCLIVE_E2E_BINARY — path to oclivenewnew-tauri (default: cargo metadata target-dir + debug/oclivenewnew-tauri[.exe])
 *   OCLIVE_ROLES_DIR  — roles root (default: <repo>/roles)
 *   OCLIVE_E2E_PORT   — listen port (default: 9843)
 *   GITHUB_WORKSPACE  — set in CI to repo root
 */

import { spawn, execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))

function repoRoot() {
  const g = process.env.GITHUB_WORKSPACE
  if (g && existsSync(join(g, 'roles'))) return resolve(g)
  return resolve(__dirname, '..')
}

function targetDirFromCargo() {
  const root = repoRoot()
  const cwd = join(root, 'distros', 'desktop-tauri')
  const out = execFileSync('cargo', ['metadata', '--format-version=1', '--no-deps'], {
    cwd,
    encoding: 'utf8',
  })
  return JSON.parse(out).target_directory
}

function defaultBinary() {
  const profile = process.env.OCLIVE_E2E_PROFILE === 'release' ? 'release' : 'debug'
  const base = targetDirFromCargo()
  const name = process.platform === 'win32' ? 'oclivenewnew-tauri.exe' : 'oclivenewnew-tauri'
  return join(base, profile, name)
}

function rolePathDefault() {
  const o = process.env.OCLIVE_OOCP_ROLE_PATH
  if (o) return resolve(o)
  return join(repoRoot(), 'roles', 'mumu')
}

async function sleep(ms) {
  await new Promise((r) => setTimeout(r, ms))
}

async function waitHealth(base, timeoutMs) {
  const deadline = Date.now() + timeoutMs
  let lastErr = ''
  while (Date.now() < deadline) {
    try {
      const r = await fetch(`${base}/health`)
      const t = await r.text()
      if (r.ok && t.trim() === 'ok') return
      lastErr = `status ${r.status} body ${JSON.stringify(t)}`
    } catch (e) {
      lastErr = e instanceof Error ? e.message : String(e)
    }
    await sleep(200)
  }
  throw new Error(`health not ready: ${lastErr}`)
}

async function postChat(base, rolePath, message) {
  const res = await fetch(`${base}/chat`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ role_path: rolePath, message, session_id: null }),
  })
  const text = await res.text()
  let body
  try {
    body = text ? JSON.parse(text) : null
  } catch {
    body = { _raw: text }
  }
  const summary = typeof body === 'object' && body !== null ? JSON.stringify(body) : String(body)
  if (!res.ok) throw new Error(`POST /chat ${res.status}: ${summary}`)
  if (typeof body?.reply !== 'string' || !body.reply.length) {
    throw new Error(`missing reply: ${summary}`)
  }
}

function startApi(port) {
  const bin = process.env.OCLIVE_E2E_BINARY || defaultBinary()
  if (!existsSync(bin)) {
    throw new Error(`binary not found: ${bin} (run cargo build -p oclivenewnew-tauri or set OCLIVE_E2E_BINARY)`)
  }
  const env = {
    ...process.env,
    OCLIVE_ROLES_DIR: process.env.OCLIVE_ROLES_DIR || join(repoRoot(), 'roles'),
    OCLIVE_HTTP_API_MOCK_LLM: process.env.OCLIVE_HTTP_API_MOCK_LLM || '1',
  }
  const child = spawn(bin, ['--api', '--port', String(port)], {
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  child.stderr?.on('data', (d) => {
    if (process.env.OCLIVE_E2E_VERBOSE) process.stderr.write(d)
  })
  child.stdout?.on('data', (d) => {
    if (process.env.OCLIVE_E2E_VERBOSE) process.stdout.write(d)
  })
  return child
}

async function killAndDrain(child) {
  if (!child || child.killed) return
  await new Promise((resolveK) => {
    child.once('exit', () => resolveK())
    try {
      child.kill(process.platform === 'win32' ? undefined : 'SIGTERM')
    } catch {
      resolveK()
      return
    }
    setTimeout(() => {
      try {
        if (!child.killed) child.kill('SIGKILL')
      } catch {
        /* ignore */
      }
    }, 8000)
  })
}

async function runCycle(port, rolePath, label) {
  process.stderr.write(`[e2e-core-restart] ${label}: starting API on port ${port}\n`)
  const child = startApi(port)
  const base = `http://127.0.0.1:${port}`
  try {
    await waitHealth(base, 60_000)
    await postChat(base, rolePath, `e2e restart probe (${label})`)
    process.stderr.write(`[e2e-core-restart] ${label}: ok\n`)
  } finally {
    await killAndDrain(child)
    await sleep(1500)
  }
}

async function main() {
  const port = Number(process.env.OCLIVE_E2E_PORT || '9843')
  if (!Number.isFinite(port) || port <= 0) throw new Error('bad OCLIVE_E2E_PORT')
  const rolePath = rolePathDefault()
  if (!existsSync(join(rolePath, 'manifest.json'))) {
    throw new Error(`role_path has no manifest: ${rolePath}`)
  }

  await runCycle(port, rolePath, 'cycle-1')
  await runCycle(port, rolePath, 'cycle-2')

  process.stderr.write('[e2e-core-restart] all cycles passed\n')
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
