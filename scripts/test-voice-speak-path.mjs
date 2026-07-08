/**
 * Smoke-test voice speak path: probe → warm (optional) → speak.
 * Usage:
 *   node scripts/test-voice-speak-path.mjs [--rpc-url URL] [--skip-warm] [--text TEXT]
 *   node scripts/test-voice-speak-path.mjs --probe-only
 *   node scripts/test-voice-speak-path.mjs --profile local-gpt-sovits-http
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

async function streamSpeak(endpoint, text) {
  const t0 = performance.now()
  let ttfc = null
  let chunks = 0
  const res = await fetch(`${endpoint.replace(/\/+$/, '')}/synthesize/stream`, {
    method: 'POST',
    headers: { 'content-type': 'application/json; charset=utf-8' },
    body: JSON.stringify({ text, emo_text: '用自然平静的语气', speed: 1 }),
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
      if (ev.event === 'chunk' && ttfc === null)
        ttfc = Math.round(performance.now() - t0)
      if (ev.event === 'chunk')
        chunks += 1
    }
  }
  return {
    ok: chunks > 0,
    ms: Math.round(performance.now() - t0),
    ttfc_ms: ttfc,
    chunks,
  }
}

const cliRpcUrl = arg('--rpc-url', null)
const text = arg('--text', '你好呀，')
const profile = arg('--profile', 'bundled-cosyvoice2-zh')
const skipWarm = process.argv.includes('--skip-warm')
const probeOnly = process.argv.includes('--probe-only')
const rpcUrl = await resolveRpcUrl(cliRpcUrl)

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

console.log('voice speak path smoke', { rpcUrl, text, profile, skipWarm, probeOnly })

// Push minimal config so probe_tts respects expansion flag
await rpcCall(rpcUrl, 'config_updated', {
  config: {
    tts_expansion_enabled: true,
    tts_profile: profile,
    synth_provider: profile === 'bundled-cosyvoice2-zh' ? 'bundled' : 'local_http',
    local_synth_endpoint: 'http://127.0.0.1:50000',
  },
})

for (const pid of MULTI_ENGINE_PROFILES) {
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
const warm = await rpcCall(rpcUrl, 'voice.warm', { profile: warmProfile })
console.log('warm', warm.ms, 'ms', JSON.stringify({
  ok: warm.body?.result?.ok,
  skipped: warm.body?.result?.skipped,
  sidecar_endpoint: warm.body?.result?.sidecar_endpoint,
  reason: warm.body?.result?.reason,
}))

const mainProbe = await rpcCall(rpcUrl, 'voice.probe_tts', { profile: warmProfile })
const sidecar = mainProbe.body?.result?.sidecar_endpoint || 'http://127.0.0.1:50000'

if (!skipWarm && warmProfile === 'bundled-cosyvoice2-zh') {
  const stream = await streamSpeak(sidecar, text)
  console.log('stream', stream)
}

const speak = await rpcCall(rpcUrl, 'voice.speak', {
  text,
  profile: warmProfile,
  bot_emotion: 'neutral',
  directive: { emo_text: '用自然平静的语气', speed: 1 },
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
