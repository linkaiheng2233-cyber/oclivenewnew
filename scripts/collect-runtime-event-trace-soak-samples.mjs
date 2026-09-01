#!/usr/bin/env node

import {
  assertPrivateEvidenceEnvelope,
  readContract,
  readPrivateEvidence,
  resolveTargetOutputDir,
  runSampler,
  sourceState,
  uniqueScenarioIds,
} from './lib/runtime-event-trace-private-evidence.mjs'

const contractRelativePath
  = 'kernel/crates/oclive_kernel_host/tests/fixtures/runtime_event_trace_shadow_soak_scenarios.v1.json'
const { contractPath, contractText, contract } = readContract(contractRelativePath)
if (contract.schema_version !== 1
  || contract.evidence_kind !== 'runtime_event_trace_shadow_synthetic_sustained_soak'
  || contract.authoritative_runtime_stream !== false
  || contract.behavior_driver !== false
  || contract.synthetic_only !== true
  || contract.exports_raw_database !== false
  || contract.bounded_execution !== true
  || contract.routine_ci !== false
  || contract.production_duration_claim !== false
  || !Array.isArray(contract.scenarios)
  || contract.scenarios.length !== 1) {
  throw new Error('invalid runtime event trace sustained-soak contract header')
}
uniqueScenarioIds(contract, 'runtime event trace sustained soak')

const expected = contract.scenarios[0]
const expectedRounds = expected.duration_ms / expected.interval_ms
const expectedWorkerRuns = expectedRounds * expected.workers
const expectedDispatches
  = expectedWorkerRuns * expected.dispatches_per_worker_per_interval
const expectedHealthChecks
  = expectedRounds / expected.health_check_interval_rounds
if (expected.action !== 'fixed_sustained_soak'
  || expected.duration_ms !== 600_000
  || !Number.isInteger(expectedRounds)
  || !Number.isInteger(expectedHealthChecks)) {
  throw new Error('runtime event trace sustained-soak schedule drifted')
}

const outputDir = resolveTargetOutputDir('target/oclive-event/trace-shadow-soak-samples')
const { sourceCommit, sourceWorktreeDirty } = sourceState()
const cargoOutput = runSampler({
  mode: 'soak',
  contractPath,
  outputDir,
  sourceCommit,
  sourceWorktreeDirty,
})
const {
  evidencePath,
  summaryPath,
  evidenceText,
  evidence,
} = readPrivateEvidence(
  outputDir,
  'runtime-event-trace-shadow.soak-evidence.json',
  'runtime-event-trace-shadow.soak-summary.md',
)
assertPrivateEvidenceEnvelope({
  evidence,
  evidenceText,
  contract,
  contractText,
  contractRelativePath,
  sourceCommit,
  sourceWorktreeDirty,
})
if (evidence.bounded_execution !== true
  || evidence.routine_ci !== false
  || evidence.production_duration_claim !== false) {
  throw new Error('runtime event trace sustained-soak evidence boundary drifted')
}

const observed = evidence.scenarios[0]
const steady = observed?.steady_diagnostics
const restartInitial = observed?.restart_initial_diagnostics
const restart = observed?.restart_diagnostics
const ring = observed?.ring
const persistence = observed?.persistence
const expectedRestartPosition = expectedDispatches + 1
const steadyHealthy = steady?.schema_version === 2
  && steady.configured === true
  && steady.accepting_dispatches === true
  && steady.worker_active === true
  && steady.enqueued_dispatches === expectedDispatches
  && steady.enqueued_events === expectedDispatches
  && steady.persisted_dispatches === expectedDispatches
  && steady.persisted_events === expectedDispatches
  && steady.duplicate_events === 0
  && steady.failed_dispatches === 0
  && steady.failed_events === 0
  && steady.dropped_dispatches === 0
  && steady.dropped_events === 0
  && steady.failure_count === 0
  && steady.last_persisted_position === expectedDispatches
  && steady.last_error_kind === null
  && steady.captures_payloads === false
  && steady.captures_metadata === false
  && steady.captures_stream_key === false
const restartContinuous = restartInitial?.schema_version === 2
  && restartInitial.configured === true
  && restartInitial.accepting_dispatches === true
  && restartInitial.worker_active === true
  && restartInitial.enqueued_dispatches === 0
  && restartInitial.persisted_dispatches === 0
  && restartInitial.last_persisted_position === expectedDispatches
  && restart?.schema_version === 2
  && restart.configured === true
  && restart.accepting_dispatches === true
  && restart.worker_active === true
  && restart.enqueued_dispatches === 1
  && restart.enqueued_events === 1
  && restart.persisted_dispatches === 1
  && restart.persisted_events === 1
  && restart.duplicate_events === 0
  && restart.failed_dispatches === 0
  && restart.failed_events === 0
  && restart.dropped_dispatches === 0
  && restart.dropped_events === 0
  && restart.failure_count === 0
  && restart.last_persisted_position === expectedRestartPosition
  && restart.last_error_kind === null
const boundedRing = ring?.history_len
    === Math.min(expectedDispatches, ring?.history_capacity)
  && ring?.last_allocated_sequence === expectedDispatches
const rssBounded = Number.isInteger(observed?.rss_start_mib)
  && Number.isInteger(observed?.rss_peak_mib)
  && Number.isInteger(observed?.rss_end_mib)
  && observed.rss_peak_mib >= observed.rss_start_mib
  && observed.rss_peak_mib >= observed.rss_end_mib
  && observed.rss_growth_mib
    === observed.rss_peak_mib - observed.rss_start_mib
  && observed.rss_growth_mib <= expected.maximum_rss_growth_mib
if (!observed
  || observed.id !== expected.id
  || observed.action !== expected.action
  || observed.outcome !== 'pass'
  || observed.duration_ms !== expected.duration_ms
  || observed.maximum_elapsed_ms !== expected.maximum_elapsed_ms
  || observed.interval_ms !== expected.interval_ms
  || observed.workers !== expected.workers
  || observed.dispatches_per_worker_per_interval
    !== expected.dispatches_per_worker_per_interval
  || observed.rounds !== expectedRounds
  || observed.expected_worker_runs !== expectedWorkerRuns
  || observed.health_check_interval_rounds !== expected.health_check_interval_rounds
  || observed.expected_health_checks !== expectedHealthChecks
  || observed.completed_health_checks !== expectedHealthChecks
  || observed.maximum_rss_growth_mib !== expected.maximum_rss_growth_mib
  || observed.attempted_dispatches !== expectedDispatches
  || observed.completed_rounds !== expectedRounds
  || observed.completed_worker_runs !== expectedWorkerRuns
  || observed.successful_ring_dispatches !== expectedDispatches
  || observed.post_restart_dispatches !== 1
  || observed.elapsed_ms < expected.duration_ms
  || observed.elapsed_ms > expected.maximum_elapsed_ms
  || !steadyHealthy
  || !restartContinuous
  || !boundedRing
  || !rssBounded
  || persistence?.rows_after_shutdown !== expectedDispatches
  || persistence?.last_position_after_shutdown !== expectedDispatches
  || persistence?.rows_after_restart !== expectedRestartPosition
  || persistence?.last_position_after_restart !== expectedRestartPosition) {
  throw new Error(`runtime event trace sustained-soak scenario ${expected.id} drifted`)
}
for (const invariant of expected.expected_invariants) {
  if (observed.invariants?.[invariant] !== true)
    throw new Error(`${expected.id} failed invariant ${invariant}`)
}
if (evidence.summary.steady_attempted_dispatches !== expectedDispatches
  || evidence.summary.steady_successful_ring_dispatches !== expectedDispatches
  || evidence.summary.post_restart_dispatches !== 1
  || evidence.summary.rows_after_shutdown !== expectedDispatches
  || evidence.summary.rows_after_restart !== expectedRestartPosition
  || evidence.summary.elapsed_ms !== observed.elapsed_ms
  || evidence.summary.rss_start_mib !== observed.rss_start_mib
  || evidence.summary.rss_peak_mib !== observed.rss_peak_mib
  || evidence.summary.rss_end_mib !== observed.rss_end_mib) {
  throw new Error('runtime event trace sustained-soak summary drifted')
}

console.log(cargoOutput)
console.log('runtime-event-trace-soak-samples: PASS (1 fixed 10-minute scenario)')
console.log(`evidence: ${evidencePath}`)
console.log(`summary: ${summaryPath}`)
