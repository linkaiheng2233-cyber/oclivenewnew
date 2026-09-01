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
  = 'kernel/crates/oclive_kernel_host/tests/fixtures/runtime_event_trace_shadow_load_scenarios.v1.json'
const { contractPath, contractText, contract } = readContract(contractRelativePath)
if (contract.schema_version !== 1
  || contract.evidence_kind !== 'runtime_event_trace_shadow_synthetic_load_samples'
  || contract.authoritative_runtime_stream !== false
  || contract.behavior_driver !== false
  || contract.synthetic_only !== true
  || contract.exports_raw_database !== false
  || contract.bounded_execution !== true
  || !Array.isArray(contract.scenarios)
  || contract.scenarios.length !== 2) {
  throw new Error('invalid runtime event trace load sample contract header')
}
uniqueScenarioIds(contract, 'runtime event trace load')

const outputDir = resolveTargetOutputDir('target/oclive-event/trace-shadow-load-samples')
const { sourceCommit, sourceWorktreeDirty } = sourceState()
const cargoOutput = runSampler({
  mode: 'load',
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
  'runtime-event-trace-shadow.load-evidence.json',
  'runtime-event-trace-shadow.load-summary.md',
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
if (evidence.bounded_execution !== true)
  throw new Error('runtime event trace load evidence must remain bounded')

let expectedTotal = 0
for (const expected of contract.scenarios) {
  const attemptedDispatches
    = expected.workers * expected.rounds * expected.dispatches_per_worker
  const expectedWorkerRuns = expected.workers * expected.rounds
  expectedTotal += attemptedDispatches
  const observed = evidence.scenarios.find(scenario => scenario.id === expected.id)
  const diagnostics = observed?.diagnostics
  const ring = observed?.ring
  const allowedError = diagnostics?.last_error_kind === null
    || diagnostics?.last_error_kind === 'queue_full'
  const dispatchAccountingBalanced
    = diagnostics?.enqueued_dispatches + diagnostics?.dropped_dispatches
      === attemptedDispatches
  const traceDrained
    = diagnostics?.persisted_dispatches === diagnostics?.enqueued_dispatches
      && diagnostics?.persisted_events + diagnostics?.duplicate_events
      === diagnostics?.enqueued_events
  const boundedRingHistory
    = ring?.history_len === Math.min(attemptedDispatches, ring?.history_capacity)
      && ring?.last_allocated_sequence === attemptedDispatches
  if (!observed
    || observed.action !== expected.action
    || observed.outcome !== 'pass'
    || observed.workers !== expected.workers
    || observed.rounds !== expected.rounds
    || observed.dispatches_per_worker !== expected.dispatches_per_worker
    || observed.pause_between_rounds_ms !== expected.pause_between_rounds_ms
    || observed.minimum_elapsed_ms !== expected.minimum_elapsed_ms
    || observed.maximum_elapsed_ms !== expected.maximum_elapsed_ms
    || observed.attempted_dispatches !== attemptedDispatches
    || observed.completed_rounds !== expected.rounds
    || observed.completed_worker_runs !== expectedWorkerRuns
    || observed.successful_ring_dispatches !== attemptedDispatches
    || observed.elapsed_ms < expected.minimum_elapsed_ms
    || observed.elapsed_ms > expected.maximum_elapsed_ms
    || diagnostics?.configured !== true
    || diagnostics?.worker_active !== true
    || !dispatchAccountingBalanced
    || !traceDrained
    || diagnostics?.duplicate_events !== 0
    || !allowedError
    || !boundedRingHistory) {
    throw new Error(`runtime event trace load scenario ${expected.id} drifted`)
  }
  for (const invariant of expected.expected_invariants) {
    if (observed.invariants?.[invariant] !== true)
      throw new Error(`${expected.id} failed invariant ${invariant}`)
  }
}
if (evidence.summary.attempted_dispatches !== expectedTotal
  || evidence.summary.successful_ring_dispatches !== expectedTotal) {
  throw new Error('runtime event trace load summary dispatch totals drifted')
}

console.log(cargoOutput)
console.log(`runtime-event-trace-load-samples: PASS (${contract.scenarios.length} scenarios)`)
console.log(`evidence: ${evidencePath}`)
console.log(`summary: ${summaryPath}`)
