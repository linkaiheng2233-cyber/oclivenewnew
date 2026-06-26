/**
 * Measure time-to-first-token (TTFT) for OCLive HTTP chat APIs.
 * Usage: node scripts/measure-ttft.mjs [--base URL] [--role-path PATH] [--runs N] [--message TEXT]
 *        [--profile desktop|desktop-latency]  (expects matching OCLIVE_DISTRO_PROFILE on API process)
 */
import { existsSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { performance } from 'node:perf_hooks'
import { chatProRolesDir, resolveRepoRoot } from './lib/chat-pro-roles-dir.mjs'

const PROFILE_PATHS = {
  desktop: 'examples/distro-profiles/desktop.oclive.toml',
  'desktop-latency': 'examples/distro-profiles/desktop-latency.oclive.toml',
}

const PROFILE_DISTRO_IDS = {
  desktop: 'desktop',
  'desktop-latency': 'desktop-latency',
}

function arg(name, fallback) {
  const i = process.argv.indexOf(name)
  if (i === -1 || i + 1 >= process.argv.length) return fallback
  return process.argv[i + 1]
}

function stats(values) {
  const sorted = [...values].sort((a, b) => a - b)
  const n = sorted.length
  const p = (q) => sorted[Math.min(n - 1, Math.max(0, Math.floor(q * (n - 1))))]
  const mean = sorted.reduce((a, b) => a + b, 0) / n
  return {
    n,
    min: sorted[0],
    p50: p(0.5),
    p95: p(0.95),
    max: sorted[n - 1],
    mean,
  }
}

function parseSseBlock(block) {
  let eventName = 'message'
  const dataLines = []
  for (const line of block.split('\n')) {
    if (line.startsWith('event:')) eventName = line.slice(6).trim()
    else if (line.startsWith('data:')) dataLines.push(line.slice(5).trim())
  }
  return { eventName, data: dataLines.join('\n') }
}

async function prepareCoPresent(base, rolePath, sceneId) {
  const roleId = rolePath.split(/[/\\]/).filter(Boolean).pop() ?? 'mumu'
  const loadRes = await fetch(`${base}/role/load`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ role_id: roleId }),
  })
  if (!loadRes.ok) {
    throw new Error(`role/load ${loadRes.status}: ${(await loadRes.text()).slice(0, 200)}`)
  }
  const switchRes = await fetch(`${base}/scene/switch`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ role_id: roleId, scene_id: sceneId, together: true }),
  })
  if (!switchRes.ok) {
    throw new Error(`scene/switch ${switchRes.status}: ${(await switchRes.text()).slice(0, 200)}`)
  }
  const presenceRes = await fetch(`${base}/scene/user_presence`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ role_id: roleId, scene_id: sceneId }),
  })
  if (!presenceRes.ok) {
    throw new Error(
      `scene/user_presence ${presenceRes.status}: ${(await presenceRes.text()).slice(0, 200)}`,
    )
  }
}

async function measureStreamOnce(base, rolePath, message, sceneId) {
  const t0 = performance.now()
  const res = await fetch(`${base}/chat/stream`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      accept: 'text/event-stream',
    },
    body: JSON.stringify({ role_path: rolePath, message, scene_id: sceneId }),
  })
  const headersMs = performance.now() - t0
  if (!res.ok) {
    const errText = await res.text()
    throw new Error(`stream HTTP ${res.status}: ${errText.slice(0, 400)}`)
  }

  const reader = res.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let ttftMs = null
  let totalMs = null
  let firstToken = ''

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    let sep
    while ((sep = buffer.indexOf('\n\n')) !== -1) {
      const block = buffer.slice(0, sep)
      buffer = buffer.slice(sep + 2)
      const { eventName, data } = parseSseBlock(block)
      if (!data) continue
      if (eventName === 'token' && ttftMs == null) {
        ttftMs = performance.now() - t0
        try {
          firstToken = JSON.parse(data).token ?? ''
        } catch {
          firstToken = data
        }
      } else if (eventName === 'done') {
        totalMs = performance.now() - t0
      }
    }
  }
  if (totalMs == null) totalMs = performance.now() - t0
  return { headersMs, ttftMs, totalMs, firstToken }
}

async function measureBlockingOnce(base, rolePath, message, sceneId) {
  const t0 = performance.now()
  const res = await fetch(`${base}/chat`, {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify({ role_path: rolePath, message, scene_id: sceneId }),
  })
  const fullMs = performance.now() - t0
  if (!res.ok) {
    const errText = await res.text()
    throw new Error(`chat HTTP ${res.status}: ${errText.slice(0, 400)}`)
  }
  const body = await res.json()
  const data = body?.data ?? body
  const reply = data?.reply ?? ''
  const promptEvalMs = data?.llmPromptEvalMs ?? data?.llm_prompt_eval_ms ?? null
  return { fullMs, replyLen: typeof reply === 'string' ? reply.length : 0, promptEvalMs }
}

async function measureOllamaDirectOnce(model, prompt) {
  const t0 = performance.now()
  const res = await fetch('http://127.0.0.1:11434/api/generate', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model, prompt, stream: true }),
  })
  if (!res.ok) throw new Error(`ollama HTTP ${res.status}: ${(await res.text()).slice(0, 200)}`)
  const reader = res.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let ttftMs = null
  let totalMs = null
  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    let nl
    while ((nl = buffer.indexOf('\n')) !== -1) {
      const line = buffer.slice(0, nl).trim()
      buffer = buffer.slice(nl + 1)
      if (!line) continue
      try {
        const json = JSON.parse(line)
        if (json.response && ttftMs == null) {
          ttftMs = performance.now() - t0
        }
        if (json.done) totalMs = performance.now() - t0
      } catch {
        /* skip */
      }
    }
  }
  if (totalMs == null) totalMs = performance.now() - t0
  return { ttftMs, totalMs }
}

async function runLabel(label, fn, runs) {
  const rows = []
  for (let i = 0; i < runs; i++) {
    rows.push(await fn(i))
  }
  return { label, rows }
}

function printStreamReport(report) {
  const ttfts = report.rows.map((r) => r.ttftMs).filter((v) => v != null)
  const totals = report.rows.map((r) => r.totalMs)
  console.log(`\n=== ${report.label} (/chat/stream) ===`)
  if (!ttfts.length) {
    console.log('  NO TOKEN EVENTS — check LLM backend / logs')
    return
  }
  const s = stats(ttfts)
  const t = stats(totals)
  console.log(`  TTFT ms:  min=${s.min.toFixed(0)} p50=${s.p50.toFixed(0)} p95=${s.p95.toFixed(0)} max=${s.max.toFixed(0)} mean=${s.mean.toFixed(0)}`)
  console.log(`  Total ms: min=${t.min.toFixed(0)} p50=${t.p50.toFixed(0)} p95=${t.p95.toFixed(0)} max=${t.max.toFixed(0)}`)
  console.log(`  First tokens: ${report.rows.map((r) => JSON.stringify(r.firstToken.slice(0, 12))).join(', ')}`)
}

function printBlockingReport(report) {
  const full = report.rows.map((r) => r.fullMs)
  const s = stats(full)
  console.log(`\n=== ${report.label} (/chat blocking — 用户可见延迟) ===`)
  console.log(`  Full reply ms: min=${s.min.toFixed(0)} p50=${s.p50.toFixed(0)} p95=${s.p95.toFixed(0)} max=${s.max.toFixed(0)} mean=${s.mean.toFixed(0)}`)
}

function printDeepPrefillReport(rows) {
  const evals = rows.map((r) => r.promptEvalMs).filter((v) => v != null)
  if (!evals.length) {
    console.log('\n=== Deep prefill (prompt_eval_ms) ===')
    console.log('  NO prompt_eval_ms — set OCLIVE_BENCH_TELEMETRY=1 on API + prompt_prefix_cache=true')
    return
  }
  const round1 = evals[0]
  const rest = evals.slice(1)
  const sRest = rest.length ? stats(rest) : null
  console.log('\n=== Deep prefill (prompt_eval_ms via /chat blocking) ===')
  evals.forEach((v, i) => console.log(`  Round ${i + 1}: ${v.toFixed(0)} ms`))
  if (sRest) {
    console.log(
      `  Round 2–${evals.length} p50: ${sRest.p50.toFixed(0)} ms (vs round1 ${round1.toFixed(0)} ms)`,
    )
    const pass = sRest.p50 < round1
    console.log(`  T3 gate (round2+ p50 < round1): ${pass ? 'PASS' : 'FAIL'}`)
  }
}

async function main() {
  const base = arg('--base', 'http://127.0.0.1:8420').replace(/\/$/, '')
  const runs = Number(arg('--runs', '5'))
  const deepOnly = process.argv.includes('--deep-only')
  const deepMulti = process.argv.includes('--deep-multi')
  const benchTelemetry = process.argv.includes('--bench-telemetry')
  const profileKey = arg('--profile', 'desktop-latency')
  const repoRoot = resolveRepoRoot()
  const profileRel = PROFILE_PATHS[profileKey]
  if (!profileRel) {
    throw new Error(`unknown --profile ${profileKey}; use desktop or desktop-latency`)
  }
  const profilePath = resolve(repoRoot, profileRel)
  const expectedDistroId = PROFILE_DISTRO_IDS[profileKey]
  const message = deepOnly || deepMulti
    ? arg(
        '--message',
        '我今天心情特别不好，想了很久要不要和你说…（认真）这件对我来说很重要，你别敷衍我，能多陪我说说话吗？我们好好聊聊。',
      )
    : arg('--message', '你好，一句话自我介绍。')
  const sceneId = arg('--scene-id', 'home')
  const rolePath = resolve(arg('--role-path', join(chatProRolesDir(resolveRepoRoot()), 'mumu')))
  const ollamaModel = arg('--ollama-model', 'qwen2.5:7b')
  const skipDirect = process.argv.includes('--skip-direct')
  const skipSetup = process.argv.includes('--skip-setup')

  if (!existsSync(join(rolePath, 'manifest.json')) && !existsSync(join(rolePath, 'pipeline.ocblueprint'))) {
    throw new Error(`role path invalid: ${rolePath}`)
  }

  const health = await fetch(`${base}/health`, { headers: { accept: 'application/json' } })
  if (!health.ok) throw new Error(`health ${health.status}`)
  const healthBody = await health.json()
  console.log(`API: ${base}`)
  console.log(`Profile: ${profileKey} → ${profileRel}`)
  console.log(`  Set on API process: OCLIVE_DISTRO_PROFILE=${profilePath}`)
  const actualDistroId = healthBody.distro_id ?? healthBody.active_profile_summary?.distro_id ?? null
  if (actualDistroId && actualDistroId !== expectedDistroId) {
    console.warn(
      `  WARN: health distro_id=${actualDistroId} (expected ${expectedDistroId} for --profile ${profileKey})`,
    )
  } else if (actualDistroId) {
    console.log(`  Health distro_id: ${actualDistroId} ✓`)
  }
  console.log(`Role: ${rolePath}`)
  console.log(`Runs: ${runs} · Scene: ${sceneId} · Message: ${message}`)
  if (deepOnly) console.log('Mode: --deep-only (Turn Thinking Deep trigger)')
  if (deepMulti) {
    console.log('Mode: --deep-multi (5-round Deep prefill via prompt_eval_ms)')
    console.log('  API env: OCLIVE_BENCH_TELEMETRY=1 · profile with prompt_prefix_cache=true')
  }
  if (benchTelemetry) console.log('Hint: also set OCLIVE_BENCH_TELEMETRY=1 on the API process')
  console.log(`Kernel: runtime=${healthBody.runtime_api_version ?? '?'} warnings=${(healthBody.startup_warnings ?? []).length}`)

  if (!skipSetup) {
    await prepareCoPresent(base, rolePath, sceneId)
    console.log(`Co-present setup: role+user scene=${sceneId} (together)`)
  }

  if (deepMulti) {
    const deepRuns = Number(arg('--runs', '5'))
    const rows = []
    for (let i = 0; i < deepRuns; i++) {
      rows.push(
        await measureBlockingOnce(
          base,
          rolePath,
          `${message} [deep-${i + 1}]`,
          sceneId,
        ),
      )
    }
    printDeepPrefillReport(rows)
    return
  }

  let settings = null
  try {
    const sRes = await fetch(`${base}/llm/user_settings?roleId=mumu`)
    if (sRes.ok) settings = await sRes.json()
  } catch {
    /* optional */
  }
  if (settings) {
    console.log(`LLM provider (DB): ${settings.provider ?? '?'} · ollamaModel=${settings.ollamaModel ?? settings.ollama_model ?? '?'}`)
  }

  const streamReport = await runLabel('OCLive stream', (i) =>
    measureStreamOnce(base, rolePath, `${message} [${i + 1}]`, sceneId), runs)
  printStreamReport(streamReport)

  const blockReport = await runLabel('OCLive blocking', (i) =>
    measureBlockingOnce(base, rolePath, `${message} [b${i + 1}]`, sceneId), Math.min(3, runs))
  printBlockingReport(blockReport)

  if (!skipDirect) {
    const direct = await runLabel(`Ollama direct (${ollamaModel})`, () =>
      measureOllamaDirectOnce(ollamaModel, message), Math.min(3, runs))
    const ttfts = direct.rows.map((r) => r.ttftMs).filter((v) => v != null)
    if (ttfts.length) {
      const s = stats(ttfts)
      console.log(`\n=== ${direct.label} (对照：极短 prompt) ===`)
      console.log(`  TTFT ms: min=${s.min.toFixed(0)} p50=${s.p50.toFixed(0)} p95=${s.p95.toFixed(0)} max=${s.max.toFixed(0)}`)
    }
  }

  const targetMs = 1000
  const streamP50 = stats(streamReport.rows.map((r) => r.ttftMs).filter(Boolean)).p50
  console.log(`\n--- vs 1s 目标 (/chat/stream TTFT p50) ---`)
  console.log(`  ${streamP50 <= targetMs ? 'PASS' : 'FAIL'}: ${streamP50.toFixed(0)} ms (p50) ${streamP50 <= targetMs ? '≤' : '>'} ${targetMs} ms`)
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
