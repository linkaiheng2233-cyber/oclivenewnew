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
  = 'kernel/crates/oclive_kernel_host/tests/fixtures/runtime_event_trace_shadow_fault_scenarios.v1.json'
const { contractPath, contractText, contract } = readContract(contractRelativePath)
if (contract.schema_version !== 1
  || contract.evidence_kind !== 'runtime_event_trace_shadow_synthetic_fault_samples'
  || contract.authoritative_runtime_stream !== false
  || contract.behavior_driver !== false
  || contract.synthetic_only !== true
  || contract.exports_raw_database !== false
  || !Array.isArray(contract.scenarios)
  || contract.scenarios.length !== 2) {
  throw new Error('invalid runtime event trace fault sample contract header')
}
uniqueScenarioIds(contract, 'runtime event trace fault')

const outputDir = resolveTargetOutputDir('target/oclive-event/trace-shadow-fault-samples')
const { sourceCommit, sourceWorktreeDirty } = sourceState()
const cargoOutput = runSampler({
  mode: 'fault',
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
  'runtime-event-trace-shadow.fault-evidence.json',
  'runtime-event-trace-shadow.fault-summary.md',
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

for (const expected of contract.scenarios) {
  const observed = evidence.scenarios.find(scenario => scenario.id === expected.id)
  if (!observed
    || observed.action !== expected.action
    || observed.outcome !== 'pass'
    || observed.attempted_dispatches !== expected.attempted_dispatches
    || observed.successful_ring_dispatches !== expected.attempted_dispatches
    || observed.diagnostics?.last_error_kind !== expected.expected_error_kind) {
    throw new Error(`runtime event trace fault scenario ${expected.id} drifted`)
  }
  for (const invariant of expected.expected_invariants) {
    if (observed.invariants?.[invariant] !== true)
      throw new Error(`${expected.id} failed invariant ${invariant}`)
  }
}

console.log(cargoOutput)
console.log(`runtime-event-trace-fault-samples: PASS (${contract.scenarios.length} scenarios)`)
console.log(`evidence: ${evidencePath}`)
console.log(`summary: ${summaryPath}`)
