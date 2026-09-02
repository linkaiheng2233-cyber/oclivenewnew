#!/usr/bin/env node

import { createHash, randomUUID } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import {
  closeSync,
  mkdirSync,
  openSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs'
import { dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

export const LIVE_CONFIRMATION = 'A2.2.2_TEST_ACCOUNT'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const contractRelativePath
  = 'kernel/crates/oclive_kernel_types/tests/fixtures/runtime_event_stream_stage_a2_onebot_review.v1.json'
const contractPath = join(repoRoot, contractRelativePath)
const defaultOutputDir = 'target/oclive-event/onebot-live-probe'
const maxResponseBytes = 64 * 1024
const probeMessagePrefix = 'OCLive A.2.2.2 delivery probe'

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

function endpointScope(endpoint, allowRemote) {
  if (endpoint.username || endpoint.password || endpoint.search || endpoint.hash)
    fail('ONEBOT_ENDPOINT_MUST_NOT_CONTAIN_CREDENTIALS_QUERY_OR_FRAGMENT')
  if (!['http:', 'https:'].includes(endpoint.protocol))
    fail('ONEBOT_ENDPOINT_PROTOCOL_UNSUPPORTED')

  const loopbackHosts = new Set(['127.0.0.1', '[::1]'])
  if (loopbackHosts.has(endpoint.hostname))
    return 'loopback'
  if (!allowRemote)
    fail('ONEBOT_REMOTE_ENDPOINT_REQUIRES_ALLOW_REMOTE')
  if (endpoint.protocol !== 'https:')
    fail('ONEBOT_REMOTE_ENDPOINT_REQUIRES_TLS')
  return 'remote_https'
}

function resolveOutputDir(argv) {
  const targetRoot = resolve(repoRoot, 'target')
  const outputDir = resolve(
    repoRoot,
    optionValue(argv, '--output-dir', defaultOutputDir),
  )
  if (outputDir !== targetRoot && !outputDir.startsWith(`${targetRoot}${sep}`))
    fail('ONEBOT_PROBE_OUTPUT_MUST_STAY_UNDER_TARGET')
  return outputDir
}

export function parseProbeConfig(argv = process.argv.slice(2), env = process.env) {
  if (optionValue(argv, '--confirm-live', '') !== LIVE_CONFIRMATION)
    fail('ONEBOT_LIVE_CONFIRMATION_REQUIRED')

  const endpointText = env.OCLIVE_ONEBOT_BASE_URL?.trim()
  const accessToken = env.OCLIVE_ONEBOT_ACCESS_TOKEN?.trim()
  const implementationLabel = env.OCLIVE_ONEBOT_IMPLEMENTATION_LABEL?.trim()
  if (!endpointText)
    fail('OCLIVE_ONEBOT_BASE_URL_REQUIRED')
  if (!accessToken || accessToken.length > 4096)
    fail('OCLIVE_ONEBOT_ACCESS_TOKEN_REQUIRED')
  if (!implementationLabel
    || !/^[a-z0-9][a-z0-9._-]{0,63}$/i.test(implementationLabel)) {
    fail('OCLIVE_ONEBOT_IMPLEMENTATION_LABEL_INVALID')
  }

  let endpoint
  try {
    endpoint = new URL(endpointText)
  }
  catch {
    fail('OCLIVE_ONEBOT_BASE_URL_INVALID')
  }
  const scope = endpointScope(endpoint, argv.includes('--allow-remote'))

  const privateId = env.OCLIVE_ONEBOT_TEST_USER_ID?.trim()
  const groupId = env.OCLIVE_ONEBOT_TEST_GROUP_ID?.trim()
  if (Boolean(privateId) === Boolean(groupId))
    fail('EXACTLY_ONE_ONEBOT_TEST_TARGET_REQUIRED')

  const timeoutText = env.OCLIVE_ONEBOT_TIMEOUT_MS?.trim() || '10000'
  const timeoutMs = positiveSafeInteger(timeoutText, 'OCLIVE_ONEBOT_TIMEOUT_MS_INVALID')
  if (timeoutMs < 1000 || timeoutMs > 30000)
    fail('OCLIVE_ONEBOT_TIMEOUT_MS_OUT_OF_RANGE')

  const target = privateId
    ? {
        kind: 'private',
        field: 'user_id',
        id: positiveSafeInteger(privateId, 'OCLIVE_ONEBOT_TEST_USER_ID_INVALID'),
        sendAction: 'send_private_msg',
      }
    : {
        kind: 'group',
        field: 'group_id',
        id: positiveSafeInteger(groupId, 'OCLIVE_ONEBOT_TEST_GROUP_ID_INVALID'),
        sendAction: 'send_group_msg',
      }

  return {
    accessToken,
    endpoint,
    endpointScope: scope,
    implementationLabel,
    outputDir: resolveOutputDir(argv),
    target,
    timeoutMs,
  }
}

function actionUrl(endpoint, action) {
  const url = new URL(endpoint)
  url.pathname = `${url.pathname.replace(/\/+$/, '')}/${action}`
  return url
}

async function postAction(config, action, body, fetchImpl) {
  let response
  try {
    response = await fetchImpl(actionUrl(config.endpoint, action), {
      method: 'POST',
      redirect: 'error',
      headers: {
        Accept: 'application/json',
        Authorization: `Bearer ${config.accessToken}`,
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
    // A malformed response after submission has an unknown delivery result.
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

function classifySend(result) {
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

function classifyAck(result) {
  if (isExplicitRejection(result))
    return 'rejected'
  if (result.httpStatus === 200
    && result.json?.status === 'ok'
    && result.json?.retcode === 0) {
    return 'acknowledged'
  }
  return 'delivery_uncertain'
}

export function sourceState() {
  return {
    commit: execFileSync('git', ['rev-parse', 'HEAD'], {
      cwd: repoRoot,
      encoding: 'utf8',
    }).trim(),
    worktreeDirty: execFileSync('git', ['status', '--short'], {
      cwd: repoRoot,
      encoding: 'utf8',
    }).trim().length > 0,
  }
}

export function readProtocolContract() {
  const text = readFileSync(contractPath, 'utf8')
  const contract = JSON.parse(text)
  if (contract.stage !== 'a2_2_1_onebot_v11_protocol_review_only'
    || contract.production_runtime_enabled !== false
    || contract.review_result?.production_ready !== false
    || contract.security_contract?.authorization_transport
    !== 'bearer_access_token_header') {
    fail('ONEBOT_PROTOCOL_CONTRACT_BOUNDARY_INVALID')
  }
  return {
    commit: contract.protocol_source.commit,
    sha256: createHash('sha256').update(text).digest('hex'),
  }
}

export async function runLiveProbe(config, {
  fetchImpl = fetch,
  now = () => new Date(),
  runId = randomUUID(),
  source = sourceState(),
  synthetic = false,
} = {}) {
  if (!/^[a-z0-9][a-z0-9._-]{0,63}$/i.test(runId))
    fail('ONEBOT_PROBE_RUN_ID_INVALID')
  const startedAt = now().toISOString()
  const protocol = readProtocolContract()
  const versionResponse = await postAction(config, 'get_version_info', {}, fetchImpl)
  const versionOutcome = classifyAck(versionResponse)
  const versionAccepted = versionOutcome === 'acknowledged'
    && versionResponse.json?.data?.protocol_version === 'v11'
  const preflightOutcome = versionAccepted
    ? 'accepted'
    : versionOutcome === 'rejected' ? 'rejected' : 'unverified'

  let sendOutcome = 'not_attempted'
  let sendHttpStatus = null
  let messageId = null
  let recallOutcome = 'not_attempted'
  let recallHttpStatus = null

  if (versionAccepted) {
    const message = `${probeMessagePrefix} ${runId}`
    const sendResponse = await postAction(config, config.target.sendAction, {
      [config.target.field]: config.target.id,
      message,
      auto_escape: true,
    }, fetchImpl)
    const send = classifySend(sendResponse)
    sendOutcome = send.outcome
    sendHttpStatus = sendResponse.httpStatus
    messageId = send.messageId

    if (send.outcome === 'delivered') {
      const recallResponse = await postAction(
        config,
        'delete_msg',
        { message_id: messageId },
        fetchImpl,
      )
      recallOutcome = classifyAck(recallResponse)
      recallHttpStatus = recallResponse.httpStatus
    }
  }

  const success = versionAccepted
    && sendOutcome === 'delivered'
    && recallOutcome === 'acknowledged'
  const evidence = {
    schema_version: 1,
    evidence_kind: 'runtime_event_stream_onebot_live_probe',
    stage: 'a2_2_2_send_and_recall_probe_only',
    run_id: runId,
    started_at: startedAt,
    finished_at: now().toISOString(),
    source_commit: source.commit,
    source_worktree_dirty: source.worktreeDirty,
    scenario_contract: contractRelativePath,
    scenario_contract_sha256: protocol.sha256,
    protocol_source_commit: protocol.commit,
    synthetic,
    live_adapter_tested: !synthetic,
    production_runtime_enabled: false,
    production_ready: false,
    endpoint_scope: config.endpointScope,
    implementation_label: config.implementationLabel,
    transport: 'http_json_post',
    target_kind: config.target.kind,
    preflight: {
      outcome: preflightOutcome,
      protocol_v11_confirmed: versionAccepted,
      implementation_metadata_present:
        typeof versionResponse.json?.data?.app_name === 'string'
        && typeof versionResponse.json?.data?.app_version === 'string',
    },
    send: {
      action: config.target.sendAction,
      outcome: sendOutcome,
      http_status: sendHttpStatus,
      message_id_present: messageId !== null,
      automatic_retries: 0,
    },
    recall: {
      action: 'delete_msg',
      outcome: recallOutcome,
      http_status: recallHttpStatus,
    },
    privacy: {
      exported_access_token: false,
      exported_endpoint: false,
      exported_target_id: false,
      exported_message_body: false,
      exported_provider_message_id: false,
    },
    remaining_gaps: {
      adapter_private_store_tested: false,
      timeout_after_possible_submission_tested: false,
      multi_host_owner_lease_tested: false,
      production_stream_connected: false,
    },
    probe_success: success,
  }

  return { evidence, success }
}

async function withProbeLock(outputDir, callback) {
  mkdirSync(outputDir, { recursive: true })
  const lockPath = join(outputDir, '.probe.lock')
  let lock
  try {
    lock = openSync(lockPath, 'wx')
  }
  catch {
    fail('ONEBOT_PROBE_ALREADY_RUNNING_OR_STALE_LOCK')
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
      // A stale lock fails closed on the next run instead of enabling concurrency.
    }
  }
}

export async function executeLiveProbe(config, dependencies) {
  return withProbeLock(config.outputDir, async () => {
    const result = await runLiveProbe(config, dependencies)
    const runDir = join(config.outputDir, result.evidence.run_id)
    mkdirSync(runDir, { recursive: false })
    const evidencePath = join(runDir, 'onebot-live-probe.evidence.json')
    writeFileSync(evidencePath, `${JSON.stringify(result.evidence, null, 2)}\n`, 'utf8')
    return { ...result, evidencePath }
  })
}

async function main() {
  const config = parseProbeConfig()
  const result = await executeLiveProbe(config)
  console.log(JSON.stringify({
    probe_success: result.success,
    send_outcome: result.evidence.send.outcome,
    recall_outcome: result.evidence.recall.outcome,
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
      : 'ONEBOT_LIVE_PROBE_FAILED'
    console.error(safeMessage)
    process.exitCode = 1
  })
}
