#!/usr/bin/env node

import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

import {
  ONEBOT_GROUP_HISTORY_OUTCOMES,
  ONEBOT_GROUP_HISTORY_PROFILE,
  PINNED_ONEBOT_V11_COMMIT,
} from './lib/runtime-event-onebot-recovery-contract.mjs'

export const RECOVERY_POLICY_CONFIRMATION
  = 'A2.2.2_FAIL_CLOSED_RECOVERY_POLICY'

const defaultOutputDir
  = 'target/oclive-event/onebot-recovery-policy-probe/r6-synthetic'
const evidenceName = 'onebot-recovery-policy-probe.evidence.json'
const noLookupProfile = 'none'
const notQueriedOutcome = 'not_queried'

const targetKinds = new Set(['group', 'private'])
const lookupOutcomes = new Set([
  notQueriedOutcome,
  ...Object.values(ONEBOT_GROUP_HISTORY_OUTCOMES),
])

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

export function parseRecoveryPolicyProbeConfig(
  argv = process.argv.slice(2),
  env = process.env,
) {
  if (optionValue(argv, '--confirm-policy', '')
    !== RECOVERY_POLICY_CONFIRMATION) {
    fail('ONEBOT_RECOVERY_POLICY_CONFIRMATION_REQUIRED')
  }
  if (env.OCLIVE_ONEBOT_PROBE_SYNTHETIC !== '1')
    fail('ONEBOT_RECOVERY_POLICY_SYNTHETIC_ONLY')
  if (argv.includes('--allow-remote'))
    fail('ONEBOT_RECOVERY_POLICY_NETWORK_FORBIDDEN')

  return {
    outputDir: optionValue(argv, '--output-dir', defaultOutputDir),
    synthetic: true,
  }
}

function validateScenario({ lookupOutcome, lookupProfile, targetKind }) {
  if (!targetKinds.has(targetKind))
    fail('ONEBOT_RECOVERY_POLICY_TARGET_KIND_INVALID')
  if (typeof lookupProfile !== 'string'
    || !/^[a-z0-9][a-z0-9_.-]{0,63}$/u.test(lookupProfile)) {
    fail('ONEBOT_RECOVERY_POLICY_LOOKUP_PROFILE_INVALID')
  }
  if (!lookupOutcomes.has(lookupOutcome))
    fail('ONEBOT_RECOVERY_POLICY_LOOKUP_OUTCOME_INVALID')
  if (lookupProfile === noLookupProfile && lookupOutcome !== notQueriedOutcome)
    fail('ONEBOT_RECOVERY_POLICY_LOOKUP_WAS_NOT_AVAILABLE')
}

export function decideUnpersistedLocatorRecovery(scenario) {
  validateScenario(scenario)
  const { lookupOutcome, lookupProfile, targetKind } = scenario

  let decision = 'manual_reconciliation_required'
  let reason = 'lookup_profile_not_allowlisted'
  let locatorPersistenceAllowed = false
  if (lookupProfile === noLookupProfile) {
    reason = 'no_trusted_lookup_profile'
  }
  else if (lookupProfile === ONEBOT_GROUP_HISTORY_PROFILE
    && targetKind !== 'group') {
    reason = 'lookup_scope_mismatch'
  }
  else if (lookupProfile === ONEBOT_GROUP_HISTORY_PROFILE
    && lookupOutcome === notQueriedOutcome) {
    reason = 'trusted_lookup_not_queried'
  }
  else if (lookupProfile === ONEBOT_GROUP_HISTORY_PROFILE
    && lookupOutcome === ONEBOT_GROUP_HISTORY_OUTCOMES.UNAVAILABLE) {
    reason = 'history_unavailable'
  }
  else if (lookupProfile === ONEBOT_GROUP_HISTORY_PROFILE
    && lookupOutcome === ONEBOT_GROUP_HISTORY_OUTCOMES.NONE) {
    reason = 'no_exact_own_match'
  }
  else if (lookupProfile === ONEBOT_GROUP_HISTORY_PROFILE
    && lookupOutcome === ONEBOT_GROUP_HISTORY_OUTCOMES.AMBIGUOUS) {
    reason = 'ambiguous_exact_own_match'
  }
  else if (lookupProfile === ONEBOT_GROUP_HISTORY_PROFILE
    && lookupOutcome === ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE) {
    decision = 'eligible_for_explicit_locator_reconciliation'
    reason = 'trusted_scope_unique_exact_own_match'
    locatorPersistenceAllowed = true
  }

  return {
    automatic_failover_allowed: false,
    automatic_provider_effects_allowed: false,
    automatic_send_retries: 0,
    checkpoint_advances: false,
    decision,
    emits_delivered: false,
    locator_persistence_allowed: locatorPersistenceAllowed,
    reason,
    requires_explicit_operator_action: true,
  }
}

function scenario(id, targetKind, lookupProfile, lookupOutcome) {
  const input = {
    lookupOutcome,
    lookupProfile,
    targetKind,
  }
  return {
    id,
    input: {
      lookup_outcome: lookupOutcome,
      lookup_profile: lookupProfile,
      target_kind: targetKind,
    },
    result: decideUnpersistedLocatorRecovery(input),
  }
}

export function recoveryPolicyEvidence() {
  const scenarios = [
    scenario(
      'private_without_standard_history',
      'private',
      noLookupProfile,
      notQueriedOutcome,
    ),
    scenario(
      'group_without_standard_history',
      'group',
      noLookupProfile,
      notQueriedOutcome,
    ),
    scenario(
      'private_with_group_only_profile',
      'private',
      ONEBOT_GROUP_HISTORY_PROFILE,
      ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
    ),
    scenario(
      'private_with_untrusted_extension',
      'private',
      'vendor.private_history',
      ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
    ),
    scenario(
      'group_history_unavailable',
      'group',
      ONEBOT_GROUP_HISTORY_PROFILE,
      ONEBOT_GROUP_HISTORY_OUTCOMES.UNAVAILABLE,
    ),
    scenario(
      'group_history_not_yet_queried',
      'group',
      ONEBOT_GROUP_HISTORY_PROFILE,
      notQueriedOutcome,
    ),
    scenario(
      'group_without_exact_match',
      'group',
      ONEBOT_GROUP_HISTORY_PROFILE,
      ONEBOT_GROUP_HISTORY_OUTCOMES.NONE,
    ),
    scenario(
      'group_with_ambiguous_matches',
      'group',
      ONEBOT_GROUP_HISTORY_PROFILE,
      ONEBOT_GROUP_HISTORY_OUTCOMES.AMBIGUOUS,
    ),
    scenario(
      'group_with_unique_exact_own_match',
      'group',
      ONEBOT_GROUP_HISTORY_PROFILE,
      ONEBOT_GROUP_HISTORY_OUTCOMES.UNIQUE,
    ),
  ]
  const eligible = scenarios.filter(item => (
    item.result.decision
    === 'eligible_for_explicit_locator_reconciliation'
  ))
  const commonBoundaryHeld = scenarios.every(item => (
    item.result.automatic_failover_allowed === false
    && item.result.automatic_provider_effects_allowed === false
    && item.result.automatic_send_retries === 0
    && item.result.checkpoint_advances === false
    && item.result.emits_delivered === false
    && item.result.requires_explicit_operator_action === true
  ))
  const success = eligible.length === 1
    && eligible[0].id === 'group_with_unique_exact_own_match'
    && eligible[0].result.locator_persistence_allowed === true
    && scenarios.filter(item => item.input.target_kind === 'private')
      .every(item => (
        item.result.decision === 'manual_reconciliation_required'
        && item.result.locator_persistence_allowed === false
      ))
      && commonBoundaryHeld

  return {
    schema_version: 1,
    evidence_kind:
      'runtime_event_stream_onebot_fail_closed_recovery_policy_probe',
    stage:
      'a2_2_2_private_and_no_history_fail_closed_policy_probe_only',
    synthetic: true,
    live_adapter_tested: false,
    network_requests_performed: 0,
    production_runtime_enabled: false,
    production_ready: false,
    trigger_state: 'attempting_without_provider_locator',
    protocol_contract: {
      pinned_onebot_v11_commit: PINNED_ONEBOT_V11_COMMIT,
      review_fixture:
        'kernel/crates/oclive_kernel_types/tests/fixtures/runtime_event_stream_stage_a2_onebot_review.v1.json',
      onebot_v11_standard_history_lookup_available: false,
      protocol_native_idempotency_key_available: false,
      provider_side_fencing_available: false,
    },
    policy_contract: {
      default_decision: 'manual_reconciliation_required',
      accepted_lookup_profiles: [ONEBOT_GROUP_HISTORY_PROFILE],
      lookup_profile_scope_must_match_target: true,
      unique_exact_own_match_required: true,
      read_only_lookup_requires_explicit_operator_action: true,
      recovered_locator_must_be_encrypted_before_recall: true,
      recall_requires_separate_explicit_reconciliation: true,
      eligible_decision_does_not_authorize_recall: true,
      original_send_must_never_be_retried_from_uncertain_state: true,
      owner_failover_must_remain_blocked: true,
      checkpoint_must_remain_blocked: true,
    },
    scenarios,
    privacy: {
      exported_access_token: false,
      exported_endpoint: false,
      exported_target_id: false,
      exported_message_body: false,
      exported_provider_message_id: false,
      exported_history_payload: false,
    },
    evidence_result: {
      scenario_count: scenarios.length,
      private_target_fail_closed_policy_frozen: true,
      no_history_fail_closed_policy_frozen: true,
      eligible_scenario_count: eligible.length,
      automatic_provider_request_count: 0,
    },
    remaining_gaps: {
      private_target_automatic_crash_recovery_available: false,
      generic_onebot_automatic_crash_recovery_available: false,
      host_level_key_recovery_tested: false,
      multi_host_owner_lease_tested: false,
      provider_side_fencing_available: false,
      production_adapter_store_connected: false,
      production_stream_connected: false,
    },
    probe_success: success,
  }
}

export function writeRecoveryPolicyEvidence(config) {
  const evidence = recoveryPolicyEvidence()
  const path = join(config.outputDir, evidenceName)
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, `${JSON.stringify(evidence, null, 2)}\n`, {
    encoding: 'utf8',
    mode: 0o600,
  })
  return { evidence, path }
}

function main() {
  const config = parseRecoveryPolicyProbeConfig()
  const { evidence } = writeRecoveryPolicyEvidence(config)
  console.log(JSON.stringify({
    network_requests_performed: evidence.network_requests_performed,
    probe_success: evidence.probe_success,
    scenario_count: evidence.evidence_result.scenario_count,
  }))
  if (!evidence.probe_success)
    process.exitCode = 2
}

const isMain = process.argv[1]
  && import.meta.url === pathToFileURL(resolve(process.argv[1])).href
if (isMain) {
  try {
    main()
  }
  catch (error) {
    const safeMessage = typeof error?.message === 'string'
      && /^[A-Z0-9_-]+$/.test(error.message)
      ? error.message
      : 'ONEBOT_RECOVERY_POLICY_PROBE_FAILED'
    console.error(safeMessage)
    process.exitCode = 1
  }
}
