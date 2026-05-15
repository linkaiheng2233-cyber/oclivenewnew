/**
 * OOCP 对齐 HTTP 黑盒：S0–S11（见 ../../creator-docs/testing/OOCP_TEST_SUITE.md）
 * 使用 Node 20+ 内置 fetch，无额外 npm 依赖。
 */

import { existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))

function argFlag(name) {
  return process.argv.includes(name)
}

function env(name, fallback) {
  const v = process.env[name]
  return v != null && String(v).trim() !== '' ? String(v).trim() : fallback
}

function repoRoot() {
  const envRoot = process.env.GITHUB_WORKSPACE
  if (envRoot && existsSync(join(envRoot, 'roles'))) return resolve(envRoot)
  return resolve(__dirname, '..', '..')
}

function defaultRolePath() {
  const override = process.env.OCLIVE_OOCP_ROLE_PATH
  if (override) return resolve(override)
  return join(repoRoot(), 'roles', 'mumu')
}

async function fetchJson(url, init) {
  const res = await fetch(url, init)
  const text = await res.text()
  let body
  try {
    body = text ? JSON.parse(text) : null
  } catch {
    body = { _raw: text }
  }
  return { res, body, text }
}

async function scenarioHandlers(base, rolePath) {
  const mumu = rolePath
  const badPath = join(repoRoot(), 'roles', '__oocp_nonexistent_role__')

  return {
    S0: async () => {
      const r = await fetch(`${base}/health`)
      const t = await r.text()
      if (!r.ok) throw new Error(`health status ${r.status}`)
      if (t.trim() !== 'ok') throw new Error(`health body expected ok, got ${JSON.stringify(t)}`)
    },
    S1: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: '   ' }),
      })
      if (res.status !== 400) throw new Error(`S1 status ${res.status}`)
      if (body?.error?.code !== 'EMPTY_MESSAGE') throw new Error(`S1 code ${JSON.stringify(body)}`)
    },
    S2: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: badPath.toString(), message: 'hi' }),
      })
      if (res.status !== 400) throw new Error(`S2 status ${res.status}`)
      const code = body?.error?.code
      if (code !== 'INVALID_ROLE_PATH' && code !== 'ROLE_NOT_FOUND') {
        throw new Error(`S2 unexpected code ${code}`)
      }
    },
    S3: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: '', message: 'hi' }),
      })
      if (res.status !== 400) throw new Error(`S3 status ${res.status}`)
      if (!body?.error?.code) throw new Error('S3 missing error code')
    },
    S4: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: '你好，OOCP 黑盒' }),
      })
      if (!res.ok) throw new Error(`S4 status ${res.status} ${JSON.stringify(body)}`)
      if (typeof body?.reply !== 'string' || !body.reply.length) {
        throw new Error('S4 missing reply')
      }
    },
    S5: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: 'ping' }),
      })
      if (!res.ok) throw new Error(`S5 status ${res.status}`)
      const ps = body?.personality_source
      if (ps !== 'vector' && ps !== 'profile') throw new Error(`S5 personality_source ${ps}`)
    },
    S6: async () => {
      const sid = 'oocp-sess-echo'
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: 'echo session', session_id: sid }),
      })
      if (!res.ok) throw new Error(`S6 status ${res.status}`)
      if (body?.session_id !== sid) throw new Error(`S6 session echo ${JSON.stringify(body?.session_id)}`)
    },
    S7: async () => {
      const sceneId = 'home'
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: 'scene explicit', scene_id: sceneId }),
      })
      if (!res.ok) throw new Error(`S7 status ${res.status}`)
      if (body?.scene_id !== sceneId) throw new Error(`S7 scene_id ${body?.scene_id}`)
    },
    S8: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: '你好 😀 中文 + emoji' }),
      })
      if (!res.ok) throw new Error(`S8 status ${res.status}`)
      if (typeof body?.reply !== 'string') throw new Error('S8 reply')
    },
    S9: async () => {
      const long = 'x'.repeat(400)
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: long }),
      })
      if (!res.ok) throw new Error(`S9 status ${res.status}`)
      if (typeof body?.reply !== 'string') throw new Error('S9 reply')
    },
    S10: async () => {
      const sid = 'oocp-sess-chain'
      const a = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: 'first turn', session_id: sid }),
      })
      if (!a.res.ok) throw new Error(`S10a ${a.res.status}`)
      const b = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: 'second turn', session_id: sid }),
      })
      if (!b.res.ok) throw new Error(`S10b ${b.res.status}`)
      if (typeof b.body?.reply !== 'string') throw new Error('S10 reply')
    },
    S11: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: 'schema fields' }),
      })
      if (!res.ok) throw new Error(`S11 status ${res.status}`)
      if (body?.api_version !== 1) throw new Error(`S11 api_version ${body?.api_version}`)
      if (typeof body?.schema !== 'number') throw new Error(`S11 schema ${body?.schema}`)
      if (typeof body?.timestamp !== 'number') throw new Error(`S11 timestamp`)
    },
  }
}

async function main() {
  const base = env('OCLIVE_API_BASE', 'http://127.0.0.1:8420').replace(/\/$/, '')
  const rolePath = defaultRolePath()
  if (!existsSync(join(rolePath, 'manifest.json'))) {
    throw new Error(`role_path invalid (no manifest): ${rolePath}`)
  }

  const handlers = await scenarioHandlers(base, rolePath)
  const order = [
    'S0',
    'S1',
    'S2',
    'S3',
    'S4',
    'S5',
    'S6',
    'S7',
    'S8',
    'S9',
    'S10',
    'S11',
  ]

  const scenarios = []
  let failed = false
  for (const id of order) {
    try {
      await handlers[id]()
      scenarios.push({ id, ok: true, detail: '' })
    } catch (e) {
      failed = true
      const detail = e instanceof Error ? e.message : String(e)
      scenarios.push({ id, ok: false, detail })
    }
  }

  const report = {
    schema: 'oclive.protocol_conformance_report.v1',
    passed: !failed,
    base_url: base,
    scenarios,
  }

  if (argFlag('--json')) {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`)
  } else {
    for (const s of scenarios) {
      const mark = s.ok ? '✓' : '✗'
      console.log(`${mark} ${s.id}${s.detail ? `: ${s.detail}` : ''}`)
    }
  }

  if (failed) process.exit(1)
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
