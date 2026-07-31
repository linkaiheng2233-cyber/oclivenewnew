/**
 * OOCP 对齐 HTTP 黑盒：S0–S12 + S15（/chat/stream SSE）；可选 S13/S14（见 ../../creator-docs/testing/OOCP_TEST_SUITE.md）
 * 使用 Node 20+ 内置 fetch，无额外 npm 依赖。
 */

import { existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chatProRolesDir, resolveRepoRoot } from '../../scripts/lib/chat-pro-roles-dir.mjs'

const __dirname = dirname(fileURLToPath(import.meta.url))

function argFlag(name) {
  return process.argv.includes(name)
}

function env(name, fallback) {
  const v = process.env[name]
  return v != null && String(v).trim() !== '' ? String(v).trim() : fallback
}

function ciContext(baseUrl) {
  const githubActions = env('GITHUB_ACTIONS', '').toLowerCase() === 'true'
  const runId = env('GITHUB_RUN_ID', '')
  const sha = env('GITHUB_SHA', '')
  const ref = env('GITHUB_REF', '')
  return {
    generated_at_utc: new Date().toISOString(),
    api_base: baseUrl,
    github_actions: githubActions,
    github_run_id: runId || null,
    github_sha: sha || null,
    github_ref: ref || null,
  }
}

function repoRoot() {
  return resolveRepoRoot()
}

function chatRolesRoot() {
  return chatProRolesDir(repoRoot())
}

function defaultRolePath() {
  const override = process.env.OCLIVE_OOCP_ROLE_PATH
  if (override) return resolve(override)
  return join(chatRolesRoot(), 'mumu')
}

function withApiToken(init = {}) {
  const headers = new Headers(init.headers || {})
  const token = env('OCLIVE_API_TOKEN', '')
  if (token) headers.set('x-oclive-api-token', token)
  return { ...init, headers }
}

function apiFetch(url, init) {
  return fetch(url, withApiToken(init))
}

async function fetchJson(url, init) {
  const res = await apiFetch(url, init)
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
  const badPath = join(chatRolesRoot(), '__oocp_nonexistent_role__')

  return {
    S0: async () => {
      const r = await apiFetch(`${base}/health`)
      const t = await r.text()
      if (!r.ok) throw new Error(`health status ${r.status}`)
      if (t.trim() !== 'ok') throw new Error(`health body expected ok, got ${JSON.stringify(t)}`)
    },
    S0b_health_startup_warnings: async () => {
      const { res, body } = await fetchJson(`${base}/health`, {
        headers: { accept: 'application/json' },
      })
      if (!res.ok) throw new Error(`S0b health status ${res.status}`)
      if (body?.ok !== true) throw new Error(`S0b health ok field ${JSON.stringify(body?.ok)}`)
      if (!Array.isArray(body?.startup_warnings)) {
        throw new Error(`S0b startup_warnings must be array, got ${typeof body?.startup_warnings}`)
      }
    },
    S1: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: '   ' }),
      })
      if (res.status !== 400) throw new Error(`S1 status ${res.status}`)
      if (body?.error?.code !== 'EMPTY_MESSAGE') throw new Error(`S1 code ${JSON.stringify(body)}`)
      if (typeof body?.error?.code !== 'string') {
        throw new Error(`S1 kernel error.code must be string, got ${typeof body?.error?.code}`)
      }
    },
    S12: async () => {
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ role_path: mumu, message: '   ' }),
      })
      if (res.status !== 400) throw new Error(`S12 status ${res.status}`)
      const code = body?.error?.code
      if (typeof code !== 'string') {
        throw new Error(`S12 expected string kernel code, got ${JSON.stringify(code)}`)
      }
      if (Number.isFinite(Number(code)) && String(Number(code)) === String(code)) {
        throw new Error(`S12 kernel code must not be JSON-RPC integer form: ${code}`)
      }
    },
    S13_dual_core_fallback: async () => {
      const dualRole = join(__dirname, 'fixtures', 'dual-core-fallback')
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          role_path: dualRole,
          message: 'OOCP S13 dual-core silent fallback',
        }),
      })
      if (!res.ok) throw new Error(`S13 status ${res.status} ${JSON.stringify(body)}`)
      if (typeof body?.reply !== 'string' || !body.reply.length) {
        throw new Error('S13 missing reply after experimental fallback')
      }
    },
    S14_dual_core_happy_path: async () => {
      const dualRole = join(__dirname, 'fixtures', 'dual-core-success')
      const { res, body } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          role_path: dualRole,
          message: 'OOCP S14 dual-core happy path',
        }),
      })
      if (!res.ok) throw new Error(`S14 status ${res.status} ${JSON.stringify(body)}`)
      if (typeof body?.reply !== 'string' || !body.reply.length) {
        throw new Error('S14 missing reply after experimental happy path')
      }
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
    S16_visual_presentation_fields: async () => {
      const visualDisabledRole = join(__dirname, 'fixtures', 'visual-disabled')
      const { res: disabledRes, body: disabledBody } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          role_path: visualDisabledRole,
          message: 'OOCP S16 visual disabled',
        }),
      })
      if (!disabledRes.ok) {
        throw new Error(`S16 disabled status ${disabledRes.status} ${JSON.stringify(disabledBody)}`)
      }
      if (disabledBody?.visual_state_id != null || disabledBody?.performance_directive != null) {
        throw new Error(`S16 disabled fixture should omit visual fields: ${JSON.stringify(disabledBody).slice(0, 300)}`)
      }
      const catalogRole = join(__dirname, 'fixtures', 'portrait-catalog')
      const { res: catRes, body: catBody } = await fetchJson(`${base}/chat`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          role_path: catalogRole,
          message: 'OOCP S16 catalog visual_state_id',
        }),
      })
      if (!catRes.ok) throw new Error(`S16 catalog status ${catRes.status} ${JSON.stringify(catBody)}`)
      if (typeof catBody?.visual_state_id !== 'string' || !catBody.visual_state_id.length) {
        throw new Error(`S16 catalog missing visual_state_id: ${JSON.stringify(catBody).slice(0, 300)}`)
      }
      if (!catBody?.performance_directive || catBody.performance_directive.kind !== 'image') {
        throw new Error(`S16 catalog missing image performance_directive: ${JSON.stringify(catBody?.performance_directive)}`)
      }
    },
    S15_chat_stream_sse: async () => {
      const res = await apiFetch(`${base}/chat/stream`, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          accept: 'text/event-stream',
        },
        body: JSON.stringify({ role_path: mumu, message: 'OOCP S15 stream' }),
      })
      if (!res.ok) {
        const errText = await res.text()
        throw new Error(`S15 status ${res.status} ${errText.slice(0, 400)}`)
      }
      const text = await res.text()
      const blocks = text.split('\n\n').map((b) => b.trim()).filter(Boolean)
      let sawToken = false
      let donePayload = null
      for (const block of blocks) {
        const lines = block.split('\n')
        let eventName = 'message'
        const dataLines = []
        for (const line of lines) {
          if (line.startsWith('event:')) eventName = line.slice(6).trim()
          else if (line.startsWith('data:')) dataLines.push(line.slice(5).trim())
        }
        if (!dataLines.length) continue
        const data = dataLines.join('\n')
        if (eventName === 'token') {
          sawToken = true
          const parsed = JSON.parse(data)
          if (typeof parsed.token !== 'string' || !parsed.token.length) {
            throw new Error(`S15 token payload invalid: ${data}`)
          }
        } else if (eventName === 'done') {
          donePayload = JSON.parse(data)
        }
      }
      if (!sawToken) throw new Error('S15 missing token event')
      if (!donePayload) throw new Error('S15 missing done event')
      const reply = donePayload?.data?.reply ?? donePayload?.reply
      if (typeof reply !== 'string' || !reply.length) {
        throw new Error(`S15 done missing reply: ${JSON.stringify(donePayload).slice(0, 200)}`)
      }
    },
  }
}

async function main() {
  const base = env('OCLIVE_API_BASE', 'http://127.0.0.1:8420').replace(/\/$/, '')
  const rolePath = defaultRolePath()
  const hasLegacy = existsSync(join(rolePath, 'manifest.json'))
  const hasV2 = existsSync(join(rolePath, 'pipeline.ocblueprint'))
  if (!hasLegacy && !hasV2) {
    throw new Error(
      `role_path invalid (need manifest.json or pipeline.ocblueprint): ${rolePath}`,
    )
  }

  const handlers = await scenarioHandlers(base, rolePath)
  const includeDualCore =
    argFlag('--include-dual-core') || process.env.OCLIVE_OOCP_INCLUDE_DUAL_CORE === '1'
  const includeS13 =
    includeDualCore || argFlag('--include-s13') || process.env.OCLIVE_OOCP_INCLUDE_S13 === '1'
  const includeS14 =
    includeDualCore || argFlag('--include-s14') || process.env.OCLIVE_OOCP_INCLUDE_S14 === '1'
  const order = [
    'S0',
    'S0b_health_startup_warnings',
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
    'S12',
    'S15_chat_stream_sse',
    'S16_visual_presentation_fields',
    ...(includeS13 ? ['S13_dual_core_fallback'] : []),
    ...(includeS14 ? ['S14_dual_core_happy_path'] : []),
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
    ci_context: ciContext(base),
    dual_core: {
      enabled: includeS13 || includeS14,
      include_s13: includeS13,
      include_s14: includeS14,
      scenarios_requested: ['S13_dual_core_fallback', 'S14_dual_core_happy_path'],
      scenarios_executed: order.filter((id) => id.startsWith('S13_') || id.startsWith('S14_')),
    },
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
