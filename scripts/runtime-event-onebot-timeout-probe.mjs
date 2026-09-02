#!/usr/bin/env node

import { Buffer } from 'node:buffer'
import { randomUUID } from 'node:crypto'
import {
  closeSync,
  mkdirSync,
  openSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

import {
  LIVE_CONFIRMATION,
  parseProbeConfig,
  runLiveProbe,
} from './runtime-event-onebot-live-probe.mjs'

export const TIMEOUT_CONFIRMATION = 'A2.2.2_TIMEOUT_AFTER_SUBMISSION'
export const RECONCILE_CONFIRMATION = 'A2.2.2_RECONCILE_UNCERTAIN'

const defaultOutputDir = 'target/oclive-event/onebot-timeout-probe'
const maxResponseBytes = 64 * 1024

function fail(code) {
  throw new Error(code)
}

function optionValue(argv, name, fallback) {
  const index = argv.indexOf(name)
  if (index === -1)
    return fallback
  const value = argv[index + 1]
  if (!value || value.startsWith('--'))
    fail(`${name.slice(2).toUpperCase().replaceAll('-', '_')}_REQUIRED`)
  return value
}

function positiveSafeInteger(value, code) {
  if (!/^\d+$/.test(value ?? ''))
    fail(code)
  const parsed = Number(value)
  if (!Number.isSafeInteger(parsed) || parsed <= 0)
    fail(code)
  return parsed
}

export function parseTimeoutProbeConfig(
  argv = process.argv.slice(2),
  env = process.env,
) {
  if (optionValue(argv, '--confirm-live', '') !== LIVE_CONFIRMATION)
    fail('ONEBOT_LIVE_CONFIRMATION_REQUIRED')
  if (optionValue(argv, '--confirm-timeout', '') !== TIMEOUT_CONFIRMATION)
    fail('ONEBOT_TIMEOUT_CONFIRMATION_REQUIRED')
  if (optionValue(argv, '--confirm-reconcile', '') !== RECONCILE_CONFIRMATION)
    fail('ONEBOT_RECONCILIATION_CONFIRMATION_REQUIRED')
  if (argv.includes('--allow-remote'))
    fail('ONEBOT_TIMEOUT_PROBE_LOOPBACK_ONLY')

  const outputDir = optionValue(argv, '--output-dir', defaultOutputDir)
  const base = parseProbeConfig(
    [
      '--confirm-live',
      LIVE_CONFIRMATION,
      '--output-dir',
      outputDir,
    ],
    env,
  )
  if (base.endpointScope !== 'loopback')
    fail('ONEBOT_TIMEOUT_PROBE_LOOPBACK_ONLY')

  const clientTimeoutMs = positiveSafeInteger(
    env.OCLIVE_ONEBOT_FAULT_TIMEOUT_MS?.trim() || '1200',
    'OCLIVE_ONEBOT_FAULT_TIMEOUT_MS_INVALID',
  )
  if (clientTimeoutMs < 1000 || clientTimeoutMs > 5000)
    fail('OCLIVE_ONEBOT_FAULT_TIMEOUT_MS_OUT_OF_RANGE')
  if (base.timeoutMs < clientTimeoutMs + 1000)
    fail('OCLIVE_ONEBOT_TIMEOUT_MS_MUST_EXCEED_FAULT_TIMEOUT')

  return {
    ...base,
    clientTimeoutMs,
    reconciliationAuthorized: true,
    timeoutFaultAuthorized: true,
  }
}

function actionUrl(endpoint, action) {
  const url = new URL(endpoint)
  url.pathname = `${url.pathname.replace(/\/+$/, '')}/${action}`
  return url
}

export async function postUpstream(config, action, body, fetchImpl) {
  let response
  try {
    response = await fetchImpl(actionUrl(config.endpoint, action), {
      method: 'POST',
      redirect: 'error',
      headers: {
        'Accept': 'application/json',
        'Authorization': `Bearer ${config.accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(config.timeoutMs),
    })
  }
  catch {
    return { httpStatus: null, json: null, transportUncertain: true }
  }

  let text
  try {
    text = await response.text()
  }
  catch {
    return { httpStatus: response.status, json: null, transportUncertain: true }
  }
  if (Buffer.byteLength(text, 'utf8') > maxResponseBytes)
    return { httpStatus: response.status, json: null, transportUncertain: true }

  let json = null
  try {
    json = JSON.parse(text)
  }
  catch {
    // A malformed response after submission cannot prove delivery or rejection.
  }
  return { httpStatus: response.status, json, transportUncertain: false }
}

function isExplicitRejection(result) {
  if ([400, 401, 403, 404, 406].includes(result.httpStatus))
    return true
  return result.httpStatus === 200
    && result.json?.status === 'failed'
    && Number.isInteger(result.json?.retcode)
    && ![0, 1].includes(result.json.retcode)
}

export function classifySend(result) {
  if (isExplicitRejection(result))
    return { outcome: 'rejected', messageId: null }
  const messageId = result.json?.data?.message_id
  if (result.httpStatus === 200
    && result.json?.status === 'ok'
    && result.json?.retcode === 0
    && Number.isSafeInteger(messageId)) {
    return { outcome: 'delivered', messageId }
  }
  return { outcome: 'delivery_uncertain', messageId: null }
}

export function classifyAck(result) {
  if (isExplicitRejection(result))
    return 'rejected'
  if (result.httpStatus === 200
    && result.json?.status === 'ok'
    && result.json?.retcode === 0) {
    return 'acknowledged'
  }
  return 'delivery_uncertain'
}

function requestBody(options) {
  try {
    return JSON.parse(String(options?.body ?? ''))
  }
  catch {
    fail('ONEBOT_TIMEOUT_FAULT_REQUEST_INVALID')
  }
}

function validateInterceptedRequest(config, action, input, options) {
  const expectedUrl = actionUrl(config.endpoint, action).href
  const actualUrl = new URL(input).href
  const headers = new Headers(options?.headers)
  if (actualUrl !== expectedUrl
    || options?.method !== 'POST'
    || headers.get('authorization') !== `Bearer ${config.accessToken}`
    || headers.get('content-type') !== 'application/json') {
    fail('ONEBOT_TIMEOUT_FAULT_REQUEST_INVALID')
  }
}

function waitForAbort(signal, onAbort, watchdogMs) {
  return new Promise((_, reject) => {
    const watchdog = setTimeout(() => {
      reject(new Error('ONEBOT_TIMEOUT_FAULT_SIGNAL_DID_NOT_ABORT'))
    }, watchdogMs)
    const rejectForAbort = () => {
      clearTimeout(watchdog)
      onAbort()
      reject(signal?.reason ?? new Error('ONEBOT_TIMEOUT_FAULT_ABORTED'))
    }
    if (!signal) {
      clearTimeout(watchdog)
      return reject(new Error('ONEBOT_TIMEOUT_FAULT_SIGNAL_REQUIRED'))
    }
    if (signal.aborted)
      return rejectForAbort()
    signal.addEventListener('abort', rejectForAbort, { once: true })
  })
}

function createResponseWithholdingFetch(config, nativeFetch) {
  const state = {
    clientAbortObserved: false,
    preflightRequests: 0,
    probeRecallAttempts: 0,
    sendRequests: 0,
    upstreamSendPromise: null,
  }

  const fetchImpl = async (input, options) => {
    const url = new URL(input)
    const action = url.pathname.split('/').filter(Boolean).at(-1)
    if (action === 'get_version_info') {
      validateInterceptedRequest(config, action, input, options)
      state.preflightRequests += 1
      return nativeFetch(input, options)
    }
    if (action === config.target.sendAction) {
      validateInterceptedRequest(config, action, input, options)
      if (state.sendRequests !== 0)
        fail('ONEBOT_TIMEOUT_FAULT_DUPLICATE_SEND')
      const body = requestBody(options)
      if (body[config.target.field] !== config.target.id
        || body.auto_escape !== true
        || typeof body.message !== 'string'
        || !/^OCLive A\.2\.2\.2 delivery probe [a-z0-9][\w.-]+$/i.test(body.message)) {
        fail('ONEBOT_TIMEOUT_FAULT_REQUEST_INVALID')
      }
      state.sendRequests += 1
      state.upstreamSendPromise = postUpstream(
        config,
        action,
        body,
        nativeFetch,
      )
      return waitForAbort(
        options.signal,
        () => {
          state.clientAbortObserved = true
        },
        config.clientTimeoutMs + 1000,
      )
    }
    if (action === 'delete_msg') {
      state.probeRecallAttempts += 1
      fail('ONEBOT_TIMEOUT_PROBE_MUST_NOT_AUTO_RECALL')
    }
    fail('ONEBOT_TIMEOUT_FAULT_ACTION_UNEXPECTED')
  }

  return {
    fetchImpl,
    state,
    async upstreamSendResult() {
      if (!state.upstreamSendPromise)
        fail('ONEBOT_TIMEOUT_FAULT_SEND_NOT_OBSERVED')
      return state.upstreamSendPromise
    },
  }
}

export async function runTimeoutProbe(config, {
  fetchImpl = fetch,
  now = () => new Date(),
  runId = randomUUID(),
  source,
  synthetic = false,
} = {}) {
  const fault = createResponseWithholdingFetch(config, fetchImpl)
  const baseResult = await runLiveProbe(
    { ...config, timeoutMs: config.clientTimeoutMs },
    { fetchImpl: fault.fetchImpl, now, runId, source, synthetic },
  )
  const upstreamResult = await fault.upstreamSendResult()
  const upstreamSend = classifySend(upstreamResult)
  const timeoutObserved = baseResult.evidence.send.outcome === 'delivery_uncertain'
    && baseResult.evidence.send.http_status === null
    && fault.state.clientAbortObserved
    && fault.state.sendRequests === 1
    && fault.state.probeRecallAttempts === 0

  let reconciliationOutcome = 'not_attempted'
  let reconciliationHttpStatus = null
  if (upstreamSend.messageId !== null) {
    const recallResult = await postUpstream(
      config,
      'delete_msg',
      { message_id: upstreamSend.messageId },
      fetchImpl,
    )
    reconciliationOutcome = classifyAck(recallResult)
    reconciliationHttpStatus = recallResult.httpStatus
  }

  const success = timeoutObserved
    && upstreamSend.outcome === 'delivered'
    && reconciliationOutcome === 'acknowledged'
  const liveTimeoutVerified = !synthetic
    && timeoutObserved
    && upstreamSend.outcome === 'delivered'
  const liveReconciliationVerified = liveTimeoutVerified
    && reconciliationOutcome === 'acknowledged'

  return {
    success,
    evidence: {
      schema_version: 1,
      evidence_kind: 'runtime_event_stream_onebot_timeout_probe',
      stage: 'a2_2_2_post_submission_timeout_reconciliation_probe_only',
      run_id: baseResult.evidence.run_id,
      started_at: baseResult.evidence.started_at,
      finished_at: now().toISOString(),
      source_commit: baseResult.evidence.source_commit,
      source_worktree_dirty: baseResult.evidence.source_worktree_dirty,
      scenario_contract: baseResult.evidence.scenario_contract,
      scenario_contract_sha256: baseResult.evidence.scenario_contract_sha256,
      protocol_source_commit: baseResult.evidence.protocol_source_commit,
      synthetic,
      live_adapter_tested: !synthetic,
      production_runtime_enabled: false,
      production_ready: false,
      endpoint_scope: config.endpointScope,
      implementation_label: config.implementationLabel,
      transport: 'http_json_post_with_in_process_response_withholding',
      target_kind: config.target.kind,
      authorization: {
        timeout_fault_confirmed_before_network: config.timeoutFaultAuthorized,
        reconciliation_confirmed_before_network:
          config.reconciliationAuthorized,
      },
      preflight: baseResult.evidence.preflight,
      injected_fault: {
        kind: 'withhold_send_response_until_client_timeout',
        client_timeout_ms: config.clientTimeoutMs,
        upstream_send_requests: fault.state.sendRequests,
        client_abort_observed: fault.state.clientAbortObserved,
      },
      probe_observation: {
        send_outcome: baseResult.evidence.send.outcome,
        send_http_status: baseResult.evidence.send.http_status,
        automatic_retries: baseResult.evidence.send.automatic_retries,
        probe_recall_attempts: fault.state.probeRecallAttempts,
        checkpoint_must_remain_blocked: true,
      },
      upstream_observation: {
        send_outcome: upstreamSend.outcome,
        send_http_status: upstreamResult.httpStatus,
        provider_message_id_present: upstreamSend.messageId !== null,
      },
      reconciliation: {
        started_only_after_probe_uncertain: timeoutObserved,
        action: 'delete_msg',
        attempts: upstreamSend.messageId === null ? 0 : 1,
        outcome: reconciliationOutcome,
        http_status: reconciliationHttpStatus,
      },
      privacy: {
        provider_locator_storage: 'process_memory_only',
        persisted_provider_locator: false,
        exported_access_token: false,
        exported_endpoint: false,
        exported_target_id: false,
        exported_message_body: false,
        exported_provider_message_id: false,
      },
      remaining_gaps: {
        adapter_private_store_tested: false,
        timeout_after_possible_submission_tested: liveTimeoutVerified,
        explicit_reconciliation_tested: liveReconciliationVerified,
        multi_host_owner_lease_tested: false,
        production_stream_connected: false,
      },
      probe_success: success,
    },
  }
}

async function withProbeLock(outputDir, callback) {
  mkdirSync(outputDir, { recursive: true })
  const lockPath = join(outputDir, '.probe.lock')
  let lock
  try {
    lock = openSync(lockPath, 'wx')
  }
  catch {
    fail('ONEBOT_TIMEOUT_PROBE_ALREADY_RUNNING_OR_STALE_LOCK')
  }
  try {
    return await callback()
  }
  finally {
    closeSync(lock)
    try {
      unlinkSync(lockPath)
    }
    catch {
      // A stale lock fails closed on the next run.
    }
  }
}

export async function executeTimeoutProbe(config, dependencies) {
  return withProbeLock(config.outputDir, async () => {
    const result = await runTimeoutProbe(config, dependencies)
    const runDir = join(config.outputDir, result.evidence.run_id)
    mkdirSync(runDir, { recursive: false })
    const evidencePath = join(runDir, 'onebot-timeout-probe.evidence.json')
    writeFileSync(evidencePath, `${JSON.stringify(result.evidence, null, 2)}\n`, 'utf8')
    return { ...result, evidencePath }
  })
}

async function main() {
  const config = parseTimeoutProbeConfig()
  const result = await executeTimeoutProbe(config)
  console.log(JSON.stringify({
    probe_success: result.success,
    probe_send_outcome: result.evidence.probe_observation.send_outcome,
    upstream_send_outcome: result.evidence.upstream_observation.send_outcome,
    reconciliation_outcome: result.evidence.reconciliation.outcome,
  }))
  console.log(`evidence: ${result.evidencePath}`)
  if (!result.success)
    process.exitCode = 1
}

const isMain = process.argv[1]
  && import.meta.url === pathToFileURL(resolve(process.argv[1])).href
if (isMain) {
  main().catch((error) => {
    const safeMessage = typeof error?.message === 'string'
      && /^[A-Z0-9_-]+$/.test(error.message)
      ? error.message
      : 'ONEBOT_TIMEOUT_PROBE_FAILED'
    console.error(safeMessage)
    process.exitCode = 1
  })
}
