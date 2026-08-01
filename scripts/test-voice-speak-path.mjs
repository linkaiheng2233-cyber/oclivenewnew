/**
 * Smoke-test voice speak path: probe → warm (optional) → speak.
 * Usage:
 *   node scripts/test-voice-speak-path.mjs [--rpc-url URL] [--skip-warm] [--text TEXT]
 *   node scripts/test-voice-speak-path.mjs --probe-only
 *   node scripts/test-voice-speak-path.mjs --profile local-gpt-sovits-http
 *   node scripts/test-voice-speak-path.mjs --role-path distros/chat-pro/roles/mumu
 *   node scripts/test-voice-speak-path.mjs --role-path distros/chat-pro/roles/mumu --runs 10 --stream-only
 *
 * RPC URL resolution: --rpc-url > OCLIVE_VOICE_RPC_URL > spawn rpc_server.mjs (ephemeral port).
 */
import { spawn } from 'node:child_process'
import { performance } from 'node:perf_hooks'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rpcScript = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr/rpc_server.mjs',
)

async function waitForRpcUrl(child, timeoutMs = 20_000) {
  return new Promise((resolve, reject) => {
    let buf = ''
    const timer = setTimeout(() => reject(new Error('rpc_server startup timeout')), timeoutMs)
    child.stdout.on('data', (chunk) => {
      buf += chunk.toString()
      const m = buf.match(/OCLIVE_READY (http:\/\/[^\s]+)/)
      if (m) {
        clearTimeout(timer)
        resolve(m[1])
      }
    })
    child.stderr.on('data', (chunk) => { buf += chunk.toString() })
    child.on('exit', (code) => {
      if (!buf.includes('OCLIVE_READY')) {
        clearTimeout(timer)
        reject(new Error(`rpc_server exited ${code}: ${buf.slice(-500)}`))
      }
    })
  })
}

async function resolveRpcUrl(cliUrl) {
  const envUrl = process.env.OCLIVE_VOICE_RPC_URL?.trim()
  if (cliUrl && cliUrl !== 'http://127.0.0.1:62658') return cliUrl
  if (envUrl) return envUrl.replace(/\/+$/, '')
  const child = spawn(process.execPath, [rpcScript], {
    cwd: repoRoot,
    env: process.env,
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  const url = await waitForRpcUrl(child)
  child.unref?.()
  child.stdout.unref?.()
  child.stderr.unref?.()
  process.on('exit', () => child.kill('SIGTERM'))
  return url
}

function arg(name, fallback) {
  const i = process.argv.indexOf(name)
  if (i === -1 || i + 1 >= process.argv.length)
    return fallback
  return process.argv[i + 1]
}

async function rpcCall(base, method, params) {
  const t0 = performance.now()
  const res = await fetch(`${base}/rpc`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-oclive-remote-protocol': 'oclive-remote-jsonrpc-v1',
    },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
    signal: AbortSignal.timeout(120_000),
  })
  const body = await res.json()
  return { ms: Math.round(performance.now() - t0), body }
}

async function streamSpeak(endpoint, text, directive = {}) {
  const t0 = performance.now()
  let ttfc = null
  let chunks = 0
  let doneMeta = null
  let firstChunkMeta = null
  const res = await fetch(`${endpoint.replace(/\/+$/, '')}/synthesize/stream`, {
    method: 'POST',
    headers: { 'content-type': 'application/json; charset=utf-8' },
    body: JSON.stringify({
      text,
      emo_text: directive.emo_text || '用自然平静的语气',
      ref_audio: directive.ref_audio || '',
      ref_text: directive.ref_text || '',
      speed: directive.speed || 1,
    }),
    signal: AbortSignal.timeout(45_000),
  })
  if (!res.ok || !res.body)
    return { ok: false, ms: Math.round(performance.now() - t0), reason: `http_${res.status}` }
  const reader = res.body.getReader()
  const dec = new TextDecoder()
  let buf = ''
  while (true) {
    const { done, value } = await reader.read()
    if (done)
      break
    buf += dec.decode(value, { stream: true })
    let idx
    while ((idx = buf.indexOf('\n')) >= 0) {
      const line = buf.slice(0, idx).trim()
      buf = buf.slice(idx + 1)
      if (!line)
        continue
      const ev = JSON.parse(line)
      if (ev.event === 'chunk' && ttfc === null) {
        ttfc = Math.round(performance.now() - t0)
        firstChunkMeta = ev
      }
      if (ev.event === 'chunk')
        chunks += 1
      if (ev.event === 'done')
        doneMeta = ev
    }
  }
  const timings = doneMeta?.timings_ms || firstChunkMeta?.timings_ms || null
  const serverPayloadReady = timings?.server_payload_ready
  return {
    ok: chunks > 0,
    ms: Math.round(performance.now() - t0),
    ttfc_ms: ttfc,
    chunks,
    sidecar_ttfc_ms: doneMeta?.ttfc_ms,
    sidecar_total_ms: doneMeta?.elapsed_ms,
    stream_mode: doneMeta?.stream_mode,
    timings_schema_version:
      doneMeta?.timings_schema_version || firstChunkMeta?.timings_schema_version,
    timings_ms: timings,
    prompt_cache_hit:
      doneMeta?.prompt_cache_hit ?? firstChunkMeta?.prompt_cache_hit ?? null,
    client_delivery_overhead_ms:
      Number.isFinite(ttfc) && Number.isFinite(serverPayloadReady)
        ? Math.max(0, ttfc - serverPayloadReady)
        : null,
  }
}

function positiveIntArg(name, fallback) {
  const parsed = Number.parseInt(arg(name, String(fallback)), 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback
}

function percentile(values, fraction) {
  if (values.length === 0)
    return null
  const sorted = [...values].sort((a, b) => a - b)
  const index = Math.max(0, Math.ceil(sorted.length * fraction) - 1)
  return sorted[index]
}

function distribution(values) {
  const numeric = values.filter(value => Number.isFinite(value))
  return {
    samples: numeric,
    p50: percentile(numeric, 0.5),
    p95: percentile(numeric, 0.95),
    max: numeric.length ? Math.max(...numeric) : null,
  }
}

const cliRpcUrl = arg('--rpc-url', null)
const text = arg('--text', '你好呀，')
const profile = arg('--profile', 'bundled-cosyvoice2-zh')
const rolePathArg = arg('--role-path', '')
const emotion = arg('--emotion', 'neutral')
const runs = positiveIntArg('--runs', 1)
const warmupRuns = positiveIntArg('--warmup-runs', runs > 1 ? 1 : 0)
const maxTtfcMs = Number.parseInt(arg('--max-ttfc-ms', '0'), 10)
const skipWarm = process.argv.includes('--skip-warm')
const probeOnly = process.argv.includes('--probe-only')
const streamOnly = process.argv.includes('--stream-only')
const rpcUrl = await resolveRpcUrl(cliRpcUrl)
const rolePath = rolePathArg
  ? path.resolve(repoRoot, rolePathArg).replace(/\\/g, '/')
  : ''

const MULTI_ENGINE_PROFILES = [
  'bundled-cosyvoice2-zh',
  'local-cosyvoice-http',
  'local-gpt-sovits-http',
  'local-qwen3-tts-http',
  'edge-tts-zh',
  'cloud-tts-openai',
  'local-fish-speech-http',
  'local-indextts-http',
]

console.log('voice speak path smoke', {
  rpcUrl,
  text,
  profile,
  skipWarm,
  probeOnly,
  runs,
  warmupRuns,
  streamOnly,
})

// Push minimal config so probe_tts respects expansion flag
await rpcCall(rpcUrl, 'config_updated', {
  config: {
    tts_expansion_enabled: true,
    tts_profile: profile,
    synth_provider: profile === 'bundled-cosyvoice2-zh' ? 'bundled' : 'local_http',
    local_synth_endpoint: 'http://127.0.0.1:50000',
  },
})

// A targeted speak/latency run must not be blocked by unrelated, optional
// adapters. The full adapter matrix remains available through --probe-only.
const profilesToProbe = probeOnly ? MULTI_ENGINE_PROFILES : [profile]
for (const pid of profilesToProbe) {
  const probe = await rpcCall(rpcUrl, 'voice.probe_tts', { profile: pid })
  const result = probe.body?.result ?? probe.body
  console.log('probe_tts', pid, probe.ms, 'ms', JSON.stringify({
    ok: result?.ok,
    engine: result?.engine,
    supports_warm: result?.supports_warm,
    supports_stream: result?.supports_stream,
    reason: result?.reason,
  }))
}

if (probeOnly) {
  console.log('probe-only done')
  process.exit(0)
}

const warmProfile = profile
let directive = { emo_text: '用自然平静的语气', speed: 1 }
if (rolePath) {
  const built = await rpcCall(rpcUrl, 'voice.build_directive', {
    role_path: rolePath,
    bot_emotion: emotion,
  })
  const result = built.body?.result ?? built.body
  if (!result?.ok || !result.directive)
    throw new Error(`voice.build_directive failed: ${JSON.stringify(result)}`)
  directive = result.directive
  console.log('directive', JSON.stringify({
    emotion_tag: directive.emotion_tag,
    speed: directive.speed,
    synth_profile: directive.synth_profile,
    has_ref_audio: Boolean(directive.ref_audio),
    has_ref_text: Boolean(directive.ref_text),
  }))
}

if (!skipWarm) {
  const warm = await rpcCall(rpcUrl, 'voice.warm', {
    profile: warmProfile,
    directive,
  })
  console.log('warm', warm.ms, 'ms', JSON.stringify({
    ok: warm.body?.result?.ok,
    skipped: warm.body?.result?.skipped,
    sidecar_endpoint: warm.body?.result?.sidecar_endpoint,
    prompt_prepared: warm.body?.result?.prompt_prepared,
    prompt_cache_hit: warm.body?.result?.prompt_cache_hit,
    precision_requested: warm.body?.result?.precision_requested,
    precision_active: warm.body?.result?.precision_active,
    precision_fallback_reason: warm.body?.result?.precision_fallback_reason,
    load_strategy: warm.body?.result?.load_strategy,
    load_vram_probe: warm.body?.result?.load_vram_probe,
    load_free_vram_before_mib: warm.body?.result?.load_free_vram_before_mib,
    load_min_free_vram_mib: warm.body?.result?.load_min_free_vram_mib,
    load_peak_reserved_mib: warm.body?.result?.load_peak_reserved_mib,
    reason: warm.body?.result?.reason,
  }))
}

const mainProbe = await rpcCall(rpcUrl, 'voice.probe_tts', { profile: warmProfile })
const sidecar = mainProbe.body?.result?.sidecar_endpoint || 'http://127.0.0.1:50000'

if (!skipWarm && warmProfile === 'bundled-cosyvoice2-zh') {
  for (let i = 0; i < warmupRuns; i += 1)
    await streamSpeak(sidecar, text, directive)
  const samples = []
  for (let i = 0; i < runs; i += 1) {
    const stream = await streamSpeak(sidecar, text, directive)
    samples.push(stream)
    console.log(`stream ${i + 1}/${runs}`, stream)
  }
  const ttfcValues = samples
    .map(sample => sample.ttfc_ms)
    .filter(value => Number.isFinite(value))
  const stageNames = [...new Set(samples.flatMap(sample => Object.keys(sample.timings_ms || {})))]
  const summary = {
    runs: samples.length,
    ok: samples.filter(sample => sample.ok).length,
    ttfc_ms: {
      min: ttfcValues.length ? Math.min(...ttfcValues) : null,
      p50: percentile(ttfcValues, 0.5),
      p95: percentile(ttfcValues, 0.95),
      max: ttfcValues.length ? Math.max(...ttfcValues) : null,
    },
    sidecar_ttfc_ms: distribution(samples.map(sample => sample.sidecar_ttfc_ms)),
    client_delivery_overhead_ms: distribution(
      samples.map(sample => sample.client_delivery_overhead_ms),
    ),
    stage_timings_ms: Object.fromEntries(
      stageNames.map(stage => [
        stage,
        distribution(samples.map(sample => sample.timings_ms?.[stage])),
      ]),
    ),
    prompt_cache: {
      hits: samples.filter(sample => sample.prompt_cache_hit === true).length,
      misses: samples.filter(sample => sample.prompt_cache_hit === false).length,
      unknown: samples.filter(sample => sample.prompt_cache_hit == null).length,
    },
    modes: [...new Set(samples.map(sample => sample.stream_mode).filter(Boolean))],
  }
  console.log('stream summary', JSON.stringify(summary))
  if (Number.isFinite(maxTtfcMs) && maxTtfcMs > 0 && summary.ttfc_ms.p95 > maxTtfcMs) {
    process.exitCode = 1
    console.error(`FAIL: stream TTFC p95 ${summary.ttfc_ms.p95}ms > ${maxTtfcMs}ms`)
  }
}

if (streamOnly)
  process.exit(process.exitCode || 0)

const speak = await rpcCall(rpcUrl, 'voice.speak', {
  text,
  profile: warmProfile,
  bot_emotion: emotion,
  directive,
})
const audioLen = speak.body?.result?.audio_base64?.length ?? 0
console.log('rpc speak', speak.ms, 'ms', JSON.stringify({
  ok: speak.body?.result?.ok,
  engine: speak.body?.result?.engine,
  reason: speak.body?.result?.reason,
  audio_base64_len: audioLen,
}))

if (!speak.body?.result?.ok && warmProfile === 'bundled-cosyvoice2-zh') {
  process.exitCode = 1
  console.error('FAIL: bundled cosyvoice speak failed (other engines may be offline — expected)')
}
