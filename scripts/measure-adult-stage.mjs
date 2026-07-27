/**
 * Measure the real-model cost of Chat Pro's durable adult staged-beat queue.
 *
 * The frontend uses one global sequential generation pump, so this benchmark
 * intentionally stages beats in order and leaves them uncommitted until the
 * requested queue depth has been reached. Generated dialogue/narration is
 * never printed; only timing, lengths, state, and resource samples are kept.
 *
 * Usage:
 *   node scripts/measure-adult-stage.mjs --base http://127.0.0.1:8430
 *   node scripts/measure-adult-stage.mjs --caps 1,2,4,8 --role gentle-landlady
 */
import { execFile } from 'node:child_process'
import { performance } from 'node:perf_hooks'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

function arg(name, fallback) {
  const index = process.argv.indexOf(name)
  if (index === -1 || index + 1 >= process.argv.length)
    return fallback
  return process.argv[index + 1]
}

function positiveInteger(value, label) {
  const parsed = Number(value)
  if (!Number.isSafeInteger(parsed) || parsed <= 0)
    throw new Error(`${label} must be a positive integer`)
  return parsed
}

function parseCaps(raw) {
  const caps = [...new Set(
    raw
      .split(',')
      .map(value => positiveInteger(value.trim(), 'each --caps value')),
  )].sort((a, b) => a - b)
  if (!caps.length)
    throw new Error('--caps must contain at least one positive integer')
  return caps
}

function percentile(values, quantile) {
  if (!values.length)
    return null
  const sorted = [...values].sort((a, b) => a - b)
  return sorted[Math.min(
    sorted.length - 1,
    Math.max(0, Math.floor(quantile * (sorted.length - 1))),
  )]
}

function stats(values) {
  if (!values.length)
    return { min: null, p50: null, p95: null, max: null, mean: null }
  const sum = values.reduce((total, value) => total + value, 0)
  return {
    min: Math.min(...values),
    p50: percentile(values, 0.5),
    p95: percentile(values, 0.95),
    max: Math.max(...values),
    mean: Math.round(sum / values.length),
  }
}

async function fetchJson(base, path, body, timeoutMs) {
  const response = await fetch(`${base}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(timeoutMs),
  })
  const text = await response.text()
  if (!response.ok)
    throw new Error(`${path} HTTP ${response.status}: ${text.slice(0, 500)}`)
  return text ? JSON.parse(text) : null
}

async function sampleGpu() {
  try {
    const { stdout } = await execFileAsync('nvidia-smi', [
      '--query-gpu=memory.used,memory.total,utilization.gpu',
      '--format=csv,noheader,nounits',
    ], {
      timeout: 5_000,
      windowsHide: true,
    })
    const [used, total, utilization] = stdout
      .trim()
      .split(/\r?\n/, 1)[0]
      .split(',')
      .map(value => Number(value.trim()))
    if ([used, total, utilization].every(Number.isFinite))
      return { used_mib: used, total_mib: total, utilization_percent: utilization }
  }
  catch {
    // GPU telemetry is optional so the queue protocol can also be measured on CPU-only hosts.
  }
  return null
}

function startGpuSampler() {
  const samples = []
  let stopped = false
  let sampling = false
  const takeSample = async () => {
    if (stopped || sampling)
      return
    sampling = true
    const sample = await sampleGpu()
    if (sample)
      samples.push(sample)
    sampling = false
  }
  void takeSample()
  const timer = setInterval(() => void takeSample(), 250)
  return async () => {
    stopped = true
    clearInterval(timer)
    while (sampling)
      await new Promise(resolve => setTimeout(resolve, 10))
    return samples
  }
}

const base = arg('--base', 'http://127.0.0.1:8430').replace(/\/+$/, '')
const roleId = arg('--role', 'gentle-landlady')
const sceneId = arg('--scene', 'default')
const caps = parseCaps(arg('--caps', '1,2,4,8'))
const timeoutMs = positiveInteger(arg('--timeout-ms', '180000'), '--timeout-ms')
const runId = new Date().toISOString().replaceAll(/[:.]/g, '-')
const adult = {
  confirmed_adult: true,
  global_enabled: true,
  role_enabled: true,
  interaction_active: true,
  action: 'continue',
}

await fetchJson(base, '/role/load', { role_id: roleId }, timeoutMs)
await fetchJson(base, '/scene/switch', {
  role_id: roleId,
  scene_id: sceneId,
  together: true,
}, timeoutMs)
await fetchJson(base, '/scene/user_presence', {
  role_id: roleId,
  scene_id: sceneId,
}, timeoutMs)

const stopGpuSampler = startGpuSampler()
const startedAt = new Date().toISOString()
const runs = []

try {
  for (const cap of caps) {
    const sessionId = `adult-stage-bench-${runId}-cap-${cap}`
    const begin = await fetchJson(base, '/chat/adult-stage/begin', {
      role_id: roleId,
      scene_id: sceneId,
      session_id: sessionId,
      adult,
    }, timeoutMs)
    const generationId = begin.generation_id
    const beats = []
    let ended = false

    try {
      for (let sequence = 0; sequence < cap; sequence += 1) {
        const beforeGpu = await sampleGpu()
        const before = performance.now()
        const staged = await fetchJson(base, '/chat/adult-stage/beat', {
          role_id: roleId,
          scene_id: sceneId,
          session_id: sessionId,
          generation_id: generationId,
          sequence,
          adult,
        }, timeoutMs)
        const elapsedMs = Math.round(performance.now() - before)
        const afterGpu = await sampleGpu()
        const adultBeat = staged.response?.adult_beat
        if (
          !adultBeat
          || typeof adultBeat.dialogue !== 'string'
          || typeof adultBeat.narration !== 'string'
          || !['inactive', 'active', 'ended'].includes(adultBeat.interaction_state)
          || !Number.isSafeInteger(adultBeat.next_beat_interval_ms)
          || adultBeat.next_beat_interval_ms <= 0
        ) {
          throw new Error(`cap ${cap}, sequence ${sequence}: incomplete structured adult beat`)
        }
        beats.push({
          sequence,
          elapsed_ms: elapsedMs,
          dialogue_chars: adultBeat?.dialogue?.length ?? 0,
          narration_chars: adultBeat?.narration?.length ?? 0,
          interaction_state: adultBeat?.interaction_state ?? null,
          reply_is_fallback: staged.response?.reply_is_fallback === true,
          gpu_before_mib: beforeGpu?.used_mib ?? null,
          gpu_after_mib: afterGpu?.used_mib ?? null,
        })
        if (adultBeat?.interaction_state === 'ended') {
          ended = true
          break
        }
      }

      const listed = await fetchJson(base, '/chat/adult-stage/list', {
        role_id: roleId,
        scene_id: sceneId,
        session_id: sessionId,
        generation_id: generationId,
      }, timeoutMs)
      const expectedCount = beats.length
      if (listed.beats.length !== expectedCount) {
        throw new Error(
          `cap ${cap}: staged count ${listed.beats.length} did not match generated ${expectedCount}`,
        )
      }
      if (listed.next_sequence !== expectedCount) {
        throw new Error(
          `cap ${cap}: next_sequence ${listed.next_sequence} did not match ${expectedCount}`,
        )
      }

      runs.push({
        requested_cap: cap,
        generated_beats: beats.length,
        model_ended: ended,
        active_before_cancel: listed.active,
        latency_ms: stats(beats.map(beat => beat.elapsed_ms)),
        total_ms: beats.reduce((total, beat) => total + beat.elapsed_ms, 0),
        fallback_count: beats.filter(beat => beat.reply_is_fallback).length,
        structured_count: beats.length,
        beats,
      })
    }
    finally {
      await fetchJson(base, '/chat/adult-stage/cancel', {
        role_id: roleId,
        scene_id: sceneId,
        session_id: sessionId,
        generation_id: generationId,
      }, timeoutMs)
    }
  }
}
finally {
  const gpuSamples = await stopGpuSampler()
  const used = gpuSamples.map(sample => sample.used_mib)
  const utilization = gpuSamples.map(sample => sample.utilization_percent)
  const totalGenerated = runs.reduce((total, run) => total + run.generated_beats, 0)
  const allLatencies = runs.flatMap(run => run.beats.map(beat => beat.elapsed_ms))
  const report = {
    benchmark: 'adult-staged-beat-queue',
    started_at: startedAt,
    finished_at: new Date().toISOString(),
    endpoint: base,
    role_id: roleId,
    scene_id: sceneId,
    caps,
    scheduling: 'single-global-sequential-pump',
    generated_text_logged: false,
    total_generated_beats: totalGenerated,
    overall_latency_ms: stats(allLatencies),
    gpu: {
      samples: gpuSamples.length,
      memory_used_mib: stats(used),
      total_mib: gpuSamples[0]?.total_mib ?? null,
      utilization_percent: stats(utilization),
    },
    runs,
  }
  console.log(JSON.stringify(report, null, 2))
}
