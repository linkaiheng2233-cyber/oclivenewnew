#!/usr/bin/env node

import { Buffer } from 'node:buffer'
import {
  createCipheriv,
  createDecipheriv,
  randomBytes,
  randomUUID,
} from 'node:crypto'
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

import {
  LIVE_CONFIRMATION,
  parseProbeConfig,
  readProtocolContract,
  sourceState,
} from './runtime-event-onebot-live-probe.mjs'
import {
  classifyAck,
  classifySend,
  postUpstream,
} from './runtime-event-onebot-timeout-probe.mjs'

export const PRIVATE_STORE_CONFIRMATION = 'A2.2.2_PRIVATE_STORE'
export const PRIVATE_RECONCILE_CONFIRMATION = 'A2.2.2_RESTART_RECONCILE'

const storeSchemaVersion = 1
const defaultOutputDir = 'target/oclive-event/onebot-private-store-probe'
const messagePrefix = 'OCLive A.2.2.2 private store probe'

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

function validateSafeId(value, code) {
  if (!/^[a-z0-9][\w.-]{0,63}$/i.test(value ?? ''))
    fail(code)
  return value
}

function parseStoreKey(value) {
  if (!/^[a-f0-9]{64}$/i.test(value ?? ''))
    fail('OCLIVE_ONEBOT_PRIVATE_STORE_KEY_INVALID')
  return Buffer.from(value, 'hex')
}

export function parsePrivateStoreProbeConfig(
  argv = process.argv.slice(2),
  env = process.env,
) {
  const mode = argv[0]
  if (!['prepare', 'reconcile'].includes(mode))
    fail('ONEBOT_PRIVATE_STORE_MODE_REQUIRED')
  if (optionValue(argv, '--confirm-live', '') !== LIVE_CONFIRMATION)
    fail('ONEBOT_LIVE_CONFIRMATION_REQUIRED')
  if (argv.includes('--allow-remote'))
    fail('ONEBOT_PRIVATE_STORE_PROBE_LOOPBACK_ONLY')
  if (mode === 'prepare'
    && optionValue(argv, '--confirm-private-store', '')
    !== PRIVATE_STORE_CONFIRMATION) {
    fail('ONEBOT_PRIVATE_STORE_CONFIRMATION_REQUIRED')
  }
  if (mode === 'reconcile'
    && optionValue(argv, '--confirm-reconcile', '')
    !== PRIVATE_RECONCILE_CONFIRMATION) {
    fail('ONEBOT_PRIVATE_RECONCILIATION_CONFIRMATION_REQUIRED')
  }

  const recordId = validateSafeId(
    optionValue(argv, '--record-id', ''),
    'ONEBOT_PRIVATE_STORE_RECORD_ID_INVALID',
  )
  const keyId = validateSafeId(
    env.OCLIVE_ONEBOT_PRIVATE_STORE_KEY_ID?.trim(),
    'OCLIVE_ONEBOT_PRIVATE_STORE_KEY_ID_INVALID',
  )
  const storeKey = parseStoreKey(
    env.OCLIVE_ONEBOT_PRIVATE_STORE_KEY?.trim(),
  )
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
    fail('ONEBOT_PRIVATE_STORE_PROBE_LOOPBACK_ONLY')

  return {
    ...base,
    keyId,
    mode,
    recordId,
    storeKey,
    synthetic: env.OCLIVE_ONEBOT_PROBE_SYNTHETIC === '1',
  }
}

function storePath(config) {
  return join(config.outputDir, 'private-store', `${config.recordId}.json`)
}

function evidencePath(config) {
  return join(
    config.outputDir,
    config.recordId,
    'onebot-private-store-probe.evidence.json',
  )
}

function aad(recordId) {
  return Buffer.from(
    `oclive-onebot-private-store:v${storeSchemaVersion}:${recordId}`,
    'utf8',
  )
}

function encryptPayload(key, recordId, payload) {
  const nonce = randomBytes(12)
  const cipher = createCipheriv('aes-256-gcm', key, nonce)
  cipher.setAAD(aad(recordId))
  const ciphertext = Buffer.concat([
    cipher.update(JSON.stringify(payload), 'utf8'),
    cipher.final(),
  ])
  return {
    algorithm: 'aes-256-gcm',
    auth_tag_base64url: cipher.getAuthTag().toString('base64url'),
    ciphertext_base64url: ciphertext.toString('base64url'),
    nonce_base64url: nonce.toString('base64url'),
  }
}

function decryptPayload(key, envelope) {
  if (envelope.schema_version !== storeSchemaVersion
    || envelope.store_kind !== 'onebot_adapter_private_recovery_probe'
    || envelope.cipher?.algorithm !== 'aes-256-gcm') {
    fail('ONEBOT_PRIVATE_STORE_ENVELOPE_INVALID')
  }
  try {
    const decipher = createDecipheriv(
      'aes-256-gcm',
      key,
      Buffer.from(envelope.cipher.nonce_base64url, 'base64url'),
    )
    decipher.setAAD(aad(envelope.record_id))
    decipher.setAuthTag(
      Buffer.from(envelope.cipher.auth_tag_base64url, 'base64url'),
    )
    const plaintext = Buffer.concat([
      decipher.update(
        Buffer.from(envelope.cipher.ciphertext_base64url, 'base64url'),
      ),
      decipher.final(),
    ]).toString('utf8')
    return JSON.parse(plaintext)
  }
  catch {
    fail('ONEBOT_PRIVATE_STORE_DECRYPT_FAILED')
  }
}

function atomicWriteJson(path, value) {
  mkdirSync(dirname(path), { recursive: true })
  const temporary = `${path}.${randomUUID()}.tmp`
  writeFileSync(temporary, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: 'utf8',
    flag: 'wx',
    mode: 0o600,
  })
  try {
    renameSync(temporary, path)
  }
  catch (error) {
    try {
      unlinkSync(temporary)
    }
    catch {
      // Preserve the original rename failure.
    }
    throw error
  }
}

function encryptedEnvelope({
  audit,
  config,
  createdAt,
  payload,
  state,
  updatedAt,
}) {
  return {
    schema_version: storeSchemaVersion,
    store_kind: 'onebot_adapter_private_recovery_probe',
    record_id: config.recordId,
    state,
    key_id: config.keyId,
    created_at: createdAt,
    updated_at: updatedAt,
    cipher: encryptPayload(config.storeKey, config.recordId, payload),
    audit,
  }
}

function readEnvelope(config) {
  const path = storePath(config)
  let envelope
  try {
    envelope = JSON.parse(readFileSync(path, 'utf8'))
  }
  catch {
    fail('ONEBOT_PRIVATE_STORE_RECORD_UNAVAILABLE')
  }
  if (envelope.record_id !== config.recordId
    || envelope.key_id !== config.keyId) {
    fail('ONEBOT_PRIVATE_STORE_RECORD_MISMATCH')
  }
  return { envelope, path }
}

function plaintextAbsent(serialized, payload) {
  return !serialized.includes(String(payload.target_id))
    && !serialized.includes(payload.message_body)
    && (payload.provider_message_id === null
      || !serialized.includes(String(payload.provider_message_id)))
}

function preflightAccepted(result) {
  return classifyAck(result) === 'acknowledged'
    && result.json?.data?.protocol_version === 'v11'
}

function lockPath(config) {
  return join(config.outputDir, `.${config.recordId}.lock`)
}

async function withRecordLock(config, callback) {
  mkdirSync(config.outputDir, { recursive: true })
  const path = lockPath(config)
  let lock
  try {
    lock = openSync(path, 'wx')
  }
  catch {
    fail('ONEBOT_PRIVATE_STORE_PROBE_ALREADY_RUNNING_OR_STALE_LOCK')
  }
  try {
    return await callback()
  }
  finally {
    closeSync(lock)
    try {
      unlinkSync(path)
    }
    catch {
      // A stale lock fails closed on the next run.
    }
  }
}

export async function preparePrivateStoreProbe(config, {
  fetchImpl = fetch,
  instanceId = randomUUID(),
  now = () => new Date(),
  source = sourceState(),
} = {}) {
  return withRecordLock(config, async () => {
    const path = storePath(config)
    if (existsSync(path))
      fail('ONEBOT_PRIVATE_STORE_RECORD_ALREADY_EXISTS')

    const createdAt = now().toISOString()
    const protocol = readProtocolContract()
    const messageBody = `${messagePrefix} ${config.recordId}`
    const payload = {
      message_body: messageBody,
      provider_message_id: null,
      target_id: config.target.id,
      target_kind: config.target.kind,
    }
    const audit = {
      automatic_retries: 0,
      implementation_label: config.implementationLabel,
      prepare_instance_id: instanceId,
      preflight_outcome: 'not_attempted',
      protocol_source_commit: protocol.commit,
      scenario_contract_sha256: protocol.sha256,
      send_http_status: null,
      send_outcome: 'not_attempted',
      source_commit: source.commit,
      source_worktree_dirty: source.worktreeDirty,
      synthetic: config.synthetic,
      target_kind: config.target.kind,
    }
    atomicWriteJson(path, encryptedEnvelope({
      audit,
      config,
      createdAt,
      payload,
      state: 'attempting',
      updatedAt: createdAt,
    }))

    const preflight = await postUpstream(
      config,
      'get_version_info',
      {},
      fetchImpl,
    )
    audit.preflight_outcome = preflightAccepted(preflight)
      ? 'accepted'
      : classifyAck(preflight) === 'rejected' ? 'rejected' : 'unverified'
    if (audit.preflight_outcome !== 'accepted') {
      atomicWriteJson(path, encryptedEnvelope({
        audit,
        config,
        createdAt,
        payload,
        state: 'preflight_failed',
        updatedAt: now().toISOString(),
      }))
      return { prepared: false, state: 'preflight_failed', storePath: path }
    }

    const sendResult = await postUpstream(
      config,
      config.target.sendAction,
      {
        [config.target.field]: config.target.id,
        message: messageBody,
        auto_escape: true,
      },
      fetchImpl,
    )
    const send = classifySend(sendResult)
    audit.send_http_status = sendResult.httpStatus
    audit.send_outcome = send.outcome
    payload.provider_message_id = send.messageId
    const state = send.outcome === 'delivered'
      ? 'delivered_pending_reconciliation'
      : 'delivery_not_reconciliable_by_locator'
    const envelope = encryptedEnvelope({
      audit,
      config,
      createdAt,
      payload,
      state,
      updatedAt: now().toISOString(),
    })
    atomicWriteJson(path, envelope)
    const serialized = readFileSync(path, 'utf8')
    if (!plaintextAbsent(serialized, payload))
      fail('ONEBOT_PRIVATE_STORE_PLAINTEXT_LEAK')

    return {
      encryptedSensitiveFields: true,
      prepared: state === 'delivered_pending_reconciliation',
      state,
      storePath: path,
    }
  })
}

export async function reconcilePrivateStoreProbe(config, {
  fetchImpl = fetch,
  instanceId = randomUUID(),
  now = () => new Date(),
  source = sourceState(),
} = {}) {
  return withRecordLock(config, async () => {
    const { envelope, path } = readEnvelope(config)
    if (envelope.state !== 'delivered_pending_reconciliation')
      fail('ONEBOT_PRIVATE_STORE_RECORD_NOT_RECONCILABLE')
    const payload = decryptPayload(config.storeKey, envelope)
    if (payload.target_kind !== config.target.kind
      || payload.target_id !== config.target.id
      || !Number.isSafeInteger(payload.provider_message_id)
      || typeof payload.message_body !== 'string'
      || payload.message_body !== `${messagePrefix} ${config.recordId}`) {
      fail('ONEBOT_PRIVATE_STORE_PAYLOAD_INVALID')
    }

    const before = readFileSync(path, 'utf8')
    const plaintextAbsentBefore = plaintextAbsent(before, payload)
    if (!plaintextAbsentBefore)
      fail('ONEBOT_PRIVATE_STORE_PLAINTEXT_LEAK')

    const preflight = await postUpstream(
      config,
      'get_version_info',
      {},
      fetchImpl,
    )
    if (!preflightAccepted(preflight))
      fail('ONEBOT_PRIVATE_STORE_RECONCILIATION_PREFLIGHT_FAILED')

    const recallResult = await postUpstream(
      config,
      'delete_msg',
      { message_id: payload.provider_message_id },
      fetchImpl,
    )
    const recallOutcome = classifyAck(recallResult)
    const reconciledAt = now().toISOString()
    const distinctInstances = envelope.audit.prepare_instance_id !== instanceId
    const recallAcknowledged = recallOutcome === 'acknowledged'

    let stateAfter = 'reconciliation_failed_locator_retained'
    if (recallAcknowledged) {
      stateAfter = 'recalled_locator_removed'
      atomicWriteJson(path, {
        schema_version: storeSchemaVersion,
        store_kind: 'onebot_adapter_private_recovery_probe',
        record_id: config.recordId,
        state: stateAfter,
        key_id: config.keyId,
        created_at: envelope.created_at,
        updated_at: reconciledAt,
        cipher: null,
        audit: {
          ...envelope.audit,
          reconcile_instance_id: instanceId,
          reconcile_source_commit: source.commit,
          reconcile_source_worktree_dirty: source.worktreeDirty,
          reconciliation_attempts: 1,
          reconciliation_http_status: recallResult.httpStatus,
          reconciliation_outcome: recallOutcome,
        },
      })
    }
    else {
      atomicWriteJson(path, {
        ...envelope,
        state: stateAfter,
        updated_at: reconciledAt,
        audit: {
          ...envelope.audit,
          reconcile_instance_id: instanceId,
          reconcile_source_commit: source.commit,
          reconcile_source_worktree_dirty: source.worktreeDirty,
          reconciliation_attempts: 1,
          reconciliation_http_status: recallResult.httpStatus,
          reconciliation_outcome: recallOutcome,
        },
      })
    }

    const after = readFileSync(path, 'utf8')
    const protocol = readProtocolContract()
    const liveStoreVerified = !config.synthetic
      && recallAcknowledged
      && plaintextAbsentBefore
      && distinctInstances
      && !after.includes(payload.message_body)
      && !after.includes(String(payload.target_id))
      && !after.includes(String(payload.provider_message_id))
    const evidence = {
      schema_version: 1,
      evidence_kind: 'runtime_event_stream_onebot_private_store_probe',
      stage: 'a2_2_2_encrypted_private_store_restart_reconciliation_probe_only',
      record_id: config.recordId,
      prepared_at: envelope.created_at,
      reconciled_at: reconciledAt,
      prepare_source_commit: envelope.audit.source_commit,
      reconcile_source_commit: source.commit,
      source_worktree_dirty:
        envelope.audit.source_worktree_dirty || source.worktreeDirty,
      scenario_contract:
        'kernel/crates/oclive_kernel_types/tests/fixtures/runtime_event_stream_stage_a2_onebot_review.v1.json',
      scenario_contract_sha256: protocol.sha256,
      protocol_source_commit: protocol.commit,
      synthetic: config.synthetic,
      live_adapter_tested: !config.synthetic,
      production_runtime_enabled: false,
      production_ready: false,
      endpoint_scope: config.endpointScope,
      implementation_label: config.implementationLabel,
      target_kind: config.target.kind,
      send: {
        outcome: envelope.audit.send_outcome,
        http_status: envelope.audit.send_http_status,
        automatic_retries: envelope.audit.automatic_retries,
        provider_message_id_present_before_encryption: true,
      },
      private_store: {
        format: 'encrypted_json_envelope',
        algorithm: 'aes-256-gcm',
        key_source: 'process_environment_only',
        key_persisted_in_store: false,
        state_before_reconciliation: envelope.state,
        state_after_reconciliation: stateAfter,
        sensitive_plaintext_absent_before_reconciliation: plaintextAbsentBefore,
        encrypted_locator_present_before_reconciliation: true,
        locator_present_after_acknowledged_reconciliation:
          !recallAcknowledged,
      },
      process_boundary: {
        prepare_instance_id: envelope.audit.prepare_instance_id,
        reconcile_instance_id: instanceId,
        distinct_invocations: distinctInstances,
      },
      reconciliation: {
        action: 'delete_msg',
        attempts: 1,
        outcome: recallOutcome,
        http_status: recallResult.httpStatus,
      },
      privacy: {
        exported_store_key: false,
        exported_access_token: false,
        exported_endpoint: false,
        exported_target_id: false,
        exported_message_body: false,
        exported_provider_message_id: false,
      },
      remaining_gaps: {
        encrypted_probe_private_store_tested: liveStoreVerified,
        restart_reconciliation_tested: liveStoreVerified,
        provider_accept_to_locator_persist_crash_window_closed: false,
        multi_host_owner_lease_tested: false,
        production_stream_connected: false,
      },
      probe_success: liveStoreVerified || (
        config.synthetic
        && recallAcknowledged
        && plaintextAbsentBefore
        && distinctInstances
      ),
    }
    const outputPath = evidencePath(config)
    atomicWriteJson(outputPath, evidence)
    return {
      evidence,
      evidencePath: outputPath,
      success: evidence.probe_success,
    }
  })
}

async function main() {
  const config = parsePrivateStoreProbeConfig()
  if (config.mode === 'prepare') {
    const result = await preparePrivateStoreProbe(config)
    console.log(JSON.stringify({
      prepared: result.prepared,
      state: result.state,
    }))
    if (!result.prepared)
      process.exitCode = 1
    return
  }

  const result = await reconcilePrivateStoreProbe(config)
  console.log(JSON.stringify({
    probe_success: result.success,
    reconciliation_outcome: result.evidence.reconciliation.outcome,
    state_after: result.evidence.private_store.state_after_reconciliation,
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
      : 'ONEBOT_PRIVATE_STORE_PROBE_FAILED'
    console.error(safeMessage)
    process.exitCode = 1
  })
}
