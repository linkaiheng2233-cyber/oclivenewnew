import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { randomUUID } from 'node:crypto'
import { mkdirSync, readFileSync, rmSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
// This script is executed directly by Node's built-in test runner.
// eslint-disable-next-line test/no-import-node-test
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  ONEBOT_GROUP_HISTORY_OUTCOMES,
  ONEBOT_GROUP_HISTORY_PROFILE,
  PINNED_ONEBOT_V11_COMMIT,
} from './lib/runtime-event-onebot-recovery-contract.mjs'
import {
  decideUnpersistedLocatorRecovery,
  parseRecoveryPolicyProbeConfig,
  RECOVERY_POLICY_CONFIRMATION,
  recoveryPolicyEvidence,
} from './runtime-event-onebot-recovery-policy-probe.mjs'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const scriptPath = join(
  repoRoot,
  'scripts/runtime-event-onebot-recovery-policy-probe.mjs',
)
const tokenSentinel = 'must-not-export-recovery-policy-token'
const targetSentinel = '1000040000'

function outputDir(label) {
  const path = join(
    repoRoot,
    'target/oclive-event/onebot-recovery-policy-probe-self-test',
    `${label}-${randomUUID()}`,
  )
  mkdirSync(path, { recursive: true })
  return path
}

function confirmedArgs(directory) {
  return [
    '--confirm-policy',
    RECOVERY_POLICY_CONFIRMATION,
    '--output-dir',
    directory,
  ]
}

function syntheticEnv() {
  return {
    ...process.env,
    OCLIVE_ONEBOT_ACCESS_TOKEN: tokenSentinel,
    OCLIVE_ONEBOT_BASE_URL: 'https://should-not-be-contacted.invalid',
    OCLIVE_ONEBOT_PROBE_SYNTHETIC: '1',
    OCLIVE_ONEBOT_TEST_PRIVATE_ID: targetSentinel,
  }
}

function assertCommonBoundary(result) {
  assert.equal(result.automatic_failover_allowed, false)
  assert.equal(result.automatic_provider_effects_allowed, false)
  assert.equal(result.automatic_send_retries, 0)
  assert.equal(result.checkpoint_advances, false)
  assert.equal(result.emits_delivered, false)
  assert.equal(result.requires_explicit_operator_action, true)
}

test('recovery policy probe is confirmation-gated, synthetic-only, and networkless', () => {
  const directory = outputDir('parse')
  try {
    assert.throws(
      () => parseRecoveryPolicyProbeConfig([], syntheticEnv()),
      /ONEBOT_RECOVERY_POLICY_CONFIRMATION_REQUIRED/,
    )
    assert.throws(
      () => parseRecoveryPolicyProbeConfig(
        confirmedArgs(directory),
        { ...syntheticEnv(), OCLIVE_ONEBOT_PROBE_SYNTHETIC: '0' },
      ),
      /ONEBOT_RECOVERY_POLICY_SYNTHETIC_ONLY/,
    )
    assert.throws(
      () => parseRecoveryPolicyProbeConfig(
        [...confirmedArgs(directory), '--allow-remote'],
        syntheticEnv(),
      ),
      /ONEBOT_RECOVERY_POLICY_NETWORK_FORBIDDEN/,
    )
    assert.deepEqual(
      parseRecoveryPolicyProbeConfig(confirmedArgs(directory), syntheticEnv()),
      { outputDir: directory, synthetic: true },
    )
  }
  finally {
    rmSync(directory, { force: true, recursive: true })
  }
})

test('recovery policy rejects impossible or unknown scenario shapes', () => {
  assert.throws(
    () => decideUnpersistedLocatorRecovery({
      lookupOutcome: 'not_queried',
      lookupProfile: 'none',
      targetKind: 'channel',
    }),
    /ONEBOT_RECOVERY_POLICY_TARGET_KIND_INVALID/,
  )
  assert.throws(
    () => decideUnpersistedLocatorRecovery({
      lookupOutcome: 'not_queried',
      lookupProfile: '',
      targetKind: 'group',
    }),
    /ONEBOT_RECOVERY_POLICY_LOOKUP_PROFILE_INVALID/,
  )
  assert.throws(
    () => decideUnpersistedLocatorRecovery({
      lookupOutcome: 'unknown',
      lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
      targetKind: 'group',
    }),
    /ONEBOT_RECOVERY_POLICY_LOOKUP_OUTCOME_INVALID/,
  )
  assert.throws(
    () => decideUnpersistedLocatorRecovery({
      lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
      lookupProfile: 'none',
      targetKind: 'group',
    }),
    /ONEBOT_RECOVERY_POLICY_LOOKUP_WAS_NOT_AVAILABLE/,
  )
})

test('private and non-recoverable history scenarios fail closed', async (t) => {
  const blocked = [
    {
      id: 'private without history',
      input: {
        lookupOutcome: 'not_queried',
        lookupProfile: 'none',
        targetKind: 'private',
      },
      reason: 'no_trusted_lookup_profile',
    },
    {
      id: 'group without history',
      input: {
        lookupOutcome: 'not_queried',
        lookupProfile: 'none',
        targetKind: 'group',
      },
      reason: 'no_trusted_lookup_profile',
    },
    {
      id: 'private with group-only profile',
      input: {
        lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
        lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
        targetKind: 'private',
      },
      reason: 'lookup_scope_mismatch',
    },
    {
      id: 'private with untrusted extension',
      input: {
        lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
        lookupProfile: 'vendor.private_history',
        targetKind: 'private',
      },
      reason: 'lookup_profile_not_allowlisted',
    },
    {
      id: 'group history unavailable',
      input: {
        lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.UNAVAILABLE,
        lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
        targetKind: 'group',
      },
      reason: 'history_unavailable',
    },
    {
      id: 'group history not yet queried',
      input: {
        lookupOutcome: 'not_queried',
        lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
        targetKind: 'group',
      },
      reason: 'trusted_lookup_not_queried',
    },
    {
      id: 'group without exact match',
      input: {
        lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.NONE,
        lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
        targetKind: 'group',
      },
      reason: 'no_exact_own_match',
    },
    {
      id: 'group with ambiguous matches',
      input: {
        lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.AMBIGUOUS,
        lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
        targetKind: 'group',
      },
      reason: 'ambiguous_exact_own_match',
    },
  ]

  for (const item of blocked) {
    await t.test(item.id, () => {
      const result = decideUnpersistedLocatorRecovery(item.input)
      assertCommonBoundary(result)
      assert.equal(result.decision, 'manual_reconciliation_required')
      assert.equal(result.locator_persistence_allowed, false)
      assert.equal(result.reason, item.reason)
    })
  }
})

test('only a trusted group unique match becomes an explicit reconciliation candidate', () => {
  const result = decideUnpersistedLocatorRecovery({
    lookupOutcome: ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
    lookupProfile: ONEBOT_GROUP_HISTORY_PROFILE,
    targetKind: 'group',
  })
  assertCommonBoundary(result)
  assert.equal(
    result.decision,
    'eligible_for_explicit_locator_reconciliation',
  )
  assert.equal(result.locator_persistence_allowed, true)
  assert.equal(result.reason, 'trusted_scope_unique_exact_own_match')
})

test('CLI writes deterministic sanitized evidence without provider access', () => {
  const directory = outputDir('cli')
  try {
    const result = spawnSync(
      process.execPath,
      [scriptPath, ...confirmedArgs(directory)],
      {
        cwd: repoRoot,
        encoding: 'utf8',
        env: syntheticEnv(),
        timeout: 2000,
        windowsHide: true,
      },
    )
    assert.equal(result.status, 0, result.stderr)
    const output = `${result.stdout}\n${result.stderr}`
    assert.doesNotMatch(output, new RegExp(tokenSentinel, 'u'))
    assert.doesNotMatch(output, new RegExp(targetSentinel, 'u'))

    const path = join(
      directory,
      'onebot-recovery-policy-probe.evidence.json',
    )
    const serialized = readFileSync(path, 'utf8')
    const evidence = JSON.parse(serialized)
    assert.deepEqual(evidence, recoveryPolicyEvidence())
    assert.equal(evidence.probe_success, true)
    assert.equal(evidence.live_adapter_tested, false)
    assert.equal(evidence.network_requests_performed, 0)
    assert.equal(
      evidence.protocol_contract.pinned_onebot_v11_commit,
      PINNED_ONEBOT_V11_COMMIT,
    )
    assert.equal(evidence.evidence_result.scenario_count, 9)
    assert.equal(evidence.evidence_result.eligible_scenario_count, 1)
    assert.equal(
      evidence.evidence_result.private_target_fail_closed_policy_frozen,
      true,
    )
    assert.equal(
      evidence.evidence_result.no_history_fail_closed_policy_frozen,
      true,
    )
    assert.doesNotMatch(serialized, new RegExp(tokenSentinel, 'u'))
    assert.doesNotMatch(serialized, new RegExp(targetSentinel, 'u'))
    assert.doesNotMatch(serialized, /https?:\/\//u)
  }
  finally {
    rmSync(directory, { force: true, recursive: true })
  }
})
