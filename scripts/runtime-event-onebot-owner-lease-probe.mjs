#!/usr/bin/env node

import { mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { DatabaseSync } from 'node:sqlite'
import { pathToFileURL } from 'node:url'

import {
  LIVE_CONFIRMATION,
  parseProbeConfig,
} from './runtime-event-onebot-live-probe.mjs'
import {
  classifyAck,
  classifySend,
  postUpstream,
} from './runtime-event-onebot-timeout-probe.mjs'

export const OWNER_LEASE_CONFIRMATION = 'A2.2.2_LOCAL_OWNER_LEASE'
export const OWNER_LEASE_ATTEMPT_EXIT_CODE = 88

const defaultOutputDir = 'target/oclive-event/onebot-owner-lease-probe'
const messagePrefix = 'OCLive A.2.2.2 owner lease probe'

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

function safeId(value, code) {
  if (!/^[a-z0-9][\w.-]{0,63}$/i.test(value ?? ''))
    fail(code)
  return value
}

function boundedInteger(value, min, max, code) {
  if (!/^\d+$/.test(value ?? ''))
    fail(code)
  const parsed = Number(value)
  if (!Number.isSafeInteger(parsed) || parsed < min || parsed > max)
    fail(code)
  return parsed
}

export function parseOwnerLeaseProbeConfig(
  argv = process.argv.slice(2),
  env = process.env,
) {
  const mode = argv[0]
  if (!['claim-only', 'claim-send', 'attempt-crash'].includes(mode))
    fail('ONEBOT_OWNER_LEASE_MODE_REQUIRED')
  if (optionValue(argv, '--confirm-owner-lease', '')
    !== OWNER_LEASE_CONFIRMATION) {
    fail('ONEBOT_OWNER_LEASE_CONFIRMATION_REQUIRED')
  }
  if (env.OCLIVE_ONEBOT_PROBE_SYNTHETIC !== '1')
    fail('ONEBOT_OWNER_LEASE_SYNTHETIC_ONLY')
  if (argv.includes('--allow-remote'))
    fail('ONEBOT_OWNER_LEASE_PROBE_LOOPBACK_ONLY')

  const outputDir = optionValue(argv, '--output-dir', defaultOutputDir)
  const base = parseProbeConfig([
    '--confirm-live',
    LIVE_CONFIRMATION,
    '--output-dir',
    outputDir,
  ], env)
  if (base.endpointScope !== 'loopback')
    fail('ONEBOT_OWNER_LEASE_PROBE_LOOPBACK_ONLY')

  return {
    ...base,
    leaseTtlMs: boundedInteger(
      env.OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS?.trim() || '2000',
      100,
      30000,
      'OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS_INVALID',
    ),
    mode,
    ownerId: safeId(
      optionValue(argv, '--owner-id', ''),
      'ONEBOT_OWNER_LEASE_OWNER_ID_INVALID',
    ),
    pauseAfterClaimMs: boundedInteger(
      env.OCLIVE_ONEBOT_OWNER_LEASE_PAUSE_MS?.trim() || '0',
      0,
      5000,
      'OCLIVE_ONEBOT_OWNER_LEASE_PAUSE_MS_INVALID',
    ),
    recordId: safeId(
      optionValue(argv, '--record-id', ''),
      'ONEBOT_OWNER_LEASE_RECORD_ID_INVALID',
    ),
  }
}

export function ownerLeaseDatabasePath(outputDir) {
  return join(outputDir, 'owner-lease', 'onebot-owner-lease-probe.sqlite3')
}

export function openOwnerLeaseProbeStore(outputDir) {
  const path = ownerLeaseDatabasePath(outputDir)
  mkdirSync(dirname(path), { recursive: true })
  const database = new DatabaseSync(path, { timeout: 3000 })
  database.exec(`
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = FULL;
    CREATE TABLE IF NOT EXISTS output_owner_lease_probe (
      record_id TEXT PRIMARY KEY,
      revision INTEGER NOT NULL CHECK (revision >= 1),
      lease_epoch INTEGER NOT NULL CHECK (lease_epoch >= 1),
      owner_id TEXT NOT NULL,
      lease_expires_at_ms INTEGER NOT NULL,
      delivery_state TEXT NOT NULL CHECK (
        delivery_state IN (
          'leased', 'attempting', 'delivered', 'rejected', 'delivery_uncertain'
        )
      ),
      attempt_epoch INTEGER,
      outcome TEXT,
      updated_at_ms INTEGER NOT NULL
    ) STRICT;
  `)
  return database
}

function withImmediateTransaction(database, callback) {
  database.exec('BEGIN IMMEDIATE')
  try {
    const result = callback()
    database.exec('COMMIT')
    return result
  }
  catch (error) {
    try {
      database.exec('ROLLBACK')
    }
    catch {
      // Preserve the original transaction error.
    }
    throw error
  }
}

function leaseToken(row) {
  return {
    leaseEpoch: Number(row.lease_epoch),
    ownerId: row.owner_id,
    recordId: row.record_id,
    revision: Number(row.revision),
  }
}

export function readOwnerLease(database, recordId) {
  return database.prepare(`
    SELECT record_id, revision, lease_epoch, owner_id,
           lease_expires_at_ms, delivery_state, attempt_epoch,
           outcome, updated_at_ms
      FROM output_owner_lease_probe
     WHERE record_id = ?
  `).get(recordId)
}

export function claimOwnerLease(database, {
  leaseTtlMs,
  nowMs,
  ownerId,
  recordId,
}) {
  return withImmediateTransaction(database, () => {
    const current = readOwnerLease(database, recordId)
    if (!current) {
      database.prepare(`
        INSERT INTO output_owner_lease_probe (
          record_id, revision, lease_epoch, owner_id,
          lease_expires_at_ms, delivery_state, attempt_epoch,
          outcome, updated_at_ms
        ) VALUES (?, 1, 1, ?, ?, 'leased', NULL, NULL, ?)
      `).run(recordId, ownerId, nowMs + leaseTtlMs, nowMs)
      const inserted = readOwnerLease(database, recordId)
      return { acquired: true, outcome: 'acquired_new', token: leaseToken(inserted) }
    }

    if (current.delivery_state === 'leased'
      && Number(current.lease_expires_at_ms) <= nowMs) {
      const changed = database.prepare(`
        UPDATE output_owner_lease_probe
           SET revision = revision + 1,
               lease_epoch = lease_epoch + 1,
               owner_id = ?,
               lease_expires_at_ms = ?,
               attempt_epoch = NULL,
               outcome = NULL,
               updated_at_ms = ?
         WHERE record_id = ?
           AND revision = ?
           AND lease_epoch = ?
           AND delivery_state = 'leased'
           AND lease_expires_at_ms <= ?
      `).run(
        ownerId,
        nowMs + leaseTtlMs,
        nowMs,
        recordId,
        current.revision,
        current.lease_epoch,
        nowMs,
      )
      if (Number(changed.changes) !== 1)
        fail('ONEBOT_OWNER_LEASE_CAS_CONFLICT')
      const takenOver = readOwnerLease(database, recordId)
      return {
        acquired: true,
        outcome: 'acquired_expired_pre_attempt',
        token: leaseToken(takenOver),
      }
    }

    const outcome = current.delivery_state === 'leased'
      ? 'lease_active'
      : ['delivered', 'rejected'].includes(current.delivery_state)
          ? `already_${current.delivery_state}`
          : 'manual_reconciliation_required'
    return { acquired: false, outcome, token: null }
  })
}

export function beginOwnerAttempt(database, token, nowMs) {
  return withImmediateTransaction(database, () => {
    const changed = database.prepare(`
      UPDATE output_owner_lease_probe
         SET revision = revision + 1,
             delivery_state = 'attempting',
             attempt_epoch = lease_epoch,
             outcome = NULL,
             updated_at_ms = ?
       WHERE record_id = ?
         AND owner_id = ?
         AND revision = ?
         AND lease_epoch = ?
         AND delivery_state = 'leased'
         AND lease_expires_at_ms > ?
    `).run(
      nowMs,
      token.recordId,
      token.ownerId,
      token.revision,
      token.leaseEpoch,
      nowMs,
    )
    if (Number(changed.changes) !== 1)
      return { accepted: false, outcome: 'fenced_before_effect', token: null }
    const attempting = readOwnerLease(database, token.recordId)
    return {
      accepted: true,
      outcome: 'attempting_persisted',
      token: leaseToken(attempting),
    }
  })
}

export function finishOwnerAttempt(database, token, outcome, nowMs) {
  if (!['delivered', 'rejected', 'delivery_uncertain'].includes(outcome))
    fail('ONEBOT_OWNER_LEASE_FINISH_OUTCOME_INVALID')
  return withImmediateTransaction(database, () => {
    const changed = database.prepare(`
      UPDATE output_owner_lease_probe
         SET revision = revision + 1,
             delivery_state = ?,
             outcome = ?,
             updated_at_ms = ?
       WHERE record_id = ?
         AND owner_id = ?
         AND revision = ?
         AND lease_epoch = ?
         AND attempt_epoch = ?
         AND delivery_state = 'attempting'
    `).run(
      outcome,
      outcome,
      nowMs,
      token.recordId,
      token.ownerId,
      token.revision,
      token.leaseEpoch,
      token.leaseEpoch,
    )
    if (Number(changed.changes) !== 1)
      return { accepted: false, outcome: 'stale_completion_fenced' }
    return { accepted: true, outcome }
  })
}

function preflightAccepted(result) {
  return classifyAck(result) === 'acknowledged'
    && result.json?.data?.protocol_version === 'v11'
}

function wait(milliseconds) {
  return new Promise(resolvePromise => setTimeout(resolvePromise, milliseconds))
}

export async function runOwnerLeaseProbe(config, {
  fetchImpl = fetch,
  now = () => Date.now(),
  onClaim = () => {},
} = {}) {
  const database = openOwnerLeaseProbeStore(config.outputDir)
  try {
    if (config.mode === 'claim-send') {
      const preflight = await postUpstream(
        config,
        'get_version_info',
        {},
        fetchImpl,
      )
      if (!preflightAccepted(preflight))
        fail('ONEBOT_OWNER_LEASE_PREFLIGHT_FAILED')
    }

    const claim = claimOwnerLease(database, {
      leaseTtlMs: config.leaseTtlMs,
      nowMs: now(),
      ownerId: config.ownerId,
      recordId: config.recordId,
    })
    if (!claim.acquired) {
      return {
        automaticRetries: 0,
        claimOutcome: claim.outcome,
        effectOutcome: 'not_attempted',
        leaseEpoch: null,
        success: false,
      }
    }
    onClaim(claim)

    if (config.mode === 'claim-only') {
      return {
        automaticRetries: 0,
        claimOutcome: claim.outcome,
        effectOutcome: 'not_attempted',
        leaseEpoch: claim.token.leaseEpoch,
        success: true,
      }
    }

    if (config.pauseAfterClaimMs > 0)
      await wait(config.pauseAfterClaimMs)
    const attempt = beginOwnerAttempt(database, claim.token, now())
    if (!attempt.accepted) {
      return {
        automaticRetries: 0,
        claimOutcome: claim.outcome,
        effectOutcome: attempt.outcome,
        leaseEpoch: claim.token.leaseEpoch,
        success: false,
      }
    }
    if (config.mode === 'attempt-crash')
      fail('ONEBOT_OWNER_LEASE_FAULT_EXIT_AFTER_ATTEMPT_PERSIST')

    const messageBody = `${messagePrefix} ${config.recordId}`
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
    const effectOutcome = ['delivered', 'rejected'].includes(send.outcome)
      ? send.outcome
      : 'delivery_uncertain'
    const finish = finishOwnerAttempt(
      database,
      attempt.token,
      effectOutcome,
      now(),
    )
    if (!finish.accepted)
      fail('ONEBOT_OWNER_LEASE_COMPLETION_FENCED')
    return {
      automaticRetries: 0,
      claimOutcome: claim.outcome,
      effectOutcome,
      leaseEpoch: claim.token.leaseEpoch,
      success: effectOutcome === 'delivered',
    }
  }
  finally {
    database.close()
  }
}

export function ownerLeaseEvidence() {
  return {
    schema_version: 1,
    evidence_kind: 'runtime_event_stream_onebot_local_owner_lease_probe',
    stage: 'a2_2_2_single_host_cross_process_owner_lease_probe_only',
    synthetic: true,
    live_adapter_tested: false,
    production_runtime_enabled: false,
    production_ready: false,
    endpoint_scope: 'loopback',
    store_backend: 'node_sqlite_probe_only',
    lease_contract: {
      single_host_cross_process_claim_tested: true,
      sqlite_begin_immediate_serializes_claims: true,
      revision_cas_required: true,
      lease_epoch_fencing_required: true,
      expired_pre_attempt_takeover_increments_epoch: true,
      stale_owner_blocked_before_effect: true,
    },
    delivery_contract: {
      attempting_persisted_before_provider_effect: true,
      expired_attempting_blocks_automatic_failover: true,
      stale_completion_rejected: true,
      automatic_send_retries: 0,
      concurrent_provider_send_count: 1,
      pre_attempt_takeover_provider_send_count: 1,
      post_attempt_crash_provider_send_count: 0,
    },
    privacy: {
      exported_access_token: false,
      exported_endpoint: false,
      exported_target_id: false,
      exported_message_body: false,
      exported_provider_message_id: false,
      sqlite_contains_delivery_payload_or_locator: false,
    },
    remaining_gaps: {
      multi_host_owner_lease_tested: false,
      provider_side_fencing_available: false,
      sigkill_or_power_loss_tested: false,
      host_level_key_recovery_tested: false,
      production_adapter_store_connected: false,
      production_stream_connected: false,
    },
    probe_success: true,
  }
}

async function main() {
  const config = parseOwnerLeaseProbeConfig()
  const result = await runOwnerLeaseProbe(config, {
    onClaim: claim => console.log(JSON.stringify({
      event: 'lease_claimed',
      lease_epoch: claim.token.leaseEpoch,
    })),
  })
  console.log(JSON.stringify({
    automatic_retries: result.automaticRetries,
    claim_outcome: result.claimOutcome,
    effect_outcome: result.effectOutcome,
    lease_epoch: result.leaseEpoch,
    probe_success: result.success,
  }))
  if (!result.success)
    process.exitCode = 2
}

const isMain = process.argv[1]
  && import.meta.url === pathToFileURL(resolve(process.argv[1])).href
if (isMain) {
  main().catch((error) => {
    const safeMessage = typeof error?.message === 'string'
      && /^[A-Z0-9_-]+$/.test(error.message)
      ? error.message
      : 'ONEBOT_OWNER_LEASE_PROBE_FAILED'
    console.error(safeMessage)
    process.exitCode
      = safeMessage === 'ONEBOT_OWNER_LEASE_FAULT_EXIT_AFTER_ATTEMPT_PERSIST'
        ? OWNER_LEASE_ATTEMPT_EXIT_CODE
        : 1
  })
}
