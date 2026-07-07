/**
 * Smoke-test voice speak path: probe → warm (optional) → speak.
 * Usage: node scripts/test-voice-speak-path.mjs [--rpc-url URL] [--skip-warm] [--text TEXT]
 */
import { performance } from 'node:perf_hooks'

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

const rpcUrl = arg('--rpc-url', 'http://127.0.0.1:62658')
const text = arg('--text', '你好呀，')
const skipWarm = process.argv.includes('--skip-warm')

console.log('voice speak path smoke', { rpcUrl, text, skipWarm })

const probe = await rpcCall(rpcUrl, 'voice.probe_tts', { profile: 'bundled-cosyvoice2-zh' })
console.log('probe_tts', probe.ms, 'ms', JSON.stringify(probe.body?.result ?? probe.body))

const sidecar = probe.body?.result?.sidecar_endpoint || 'http://127.0.0.1:50000'

if (!skipWarm) {
  const warm = await rpcCall(rpcUrl, 'voice.warm', { profile: 'bundled-cosyvoice2-zh' })
  console.log('warm', warm.ms, 'ms', JSON.stringify({
    ok: warm.body?.result?.ok,
    sidecar_endpoint: warm.body?.result?.sidecar_endpoint,
    reason: warm.body?.result?.reason,
  }))
}

const stream = await streamSpeak(sidecar, text)
console.log('stream', stream)

const speak = await rpcCall(rpcUrl, 'voice.speak', {
  text,
  profile: 'bundled-cosyvoice2-zh',
  bot_emotion: 'neutral',
  directive: { emo_text: '用自然平静的语气', speed: 1 },
})
const audioLen = speak.body?.result?.audio_base64?.length ?? 0
console.log('rpc speak', speak.ms, 'ms', JSON.stringify({
  ok: speak.body?.result?.ok,
  reason: speak.body?.result?.reason,
  audio_base64_len: audioLen,
}))

if (!stream.ok && !speak.body?.result?.ok) {
  process.exitCode = 1
  console.error('FAIL: both stream and RPC speak failed')
}
