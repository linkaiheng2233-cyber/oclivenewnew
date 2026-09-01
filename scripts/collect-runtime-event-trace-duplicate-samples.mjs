#!/usr/bin/env node

import { createHash } from 'node:crypto'
import { writeFileSync } from 'node:fs'
import { join } from 'node:path'

import {
  assertNoPrivateEvidence,
  assertPrivateEvidenceEnvelope,
  readContract,
  readPrivateEvidence,
  resolveTargetOutputDir,
  run,
  sourceState,
  uniqueScenarioIds,
} from './lib/runtime-event-trace-private-evidence.mjs'

const contractRelativePath
  = 'kernel/crates/oclive_kernel_host/tests/fixtures/runtime_event_trace_shadow_duplicate_scenarios.v1.json'
const { contractText, contract } = readContract(contractRelativePath)
if (contract.schema_version !== 1
  || contract.evidence_kind !== 'runtime_event_trace_shadow_synthetic_duplicate_samples'
  || contract.authoritative_runtime_stream !== false
  || contract.behavior_driver !== false
  || contract.synthetic_only !== true
  || contract.exports_raw_database !== false
  || contract.production_replay_entry !== false
  || contract.test_compilation_only !== true
  || !Array.isArray(contract.scenarios)
  || contract.scenarios.length !== 1) {
  throw new Error('invalid runtime event trace duplicate sample contract header')
}
uniqueScenarioIds(contract, 'runtime event trace duplicate')

const expected = contract.scenarios[0]
const outputDir = resolveTargetOutputDir(
  'target/oclive-event/trace-shadow-duplicate-samples',
)
const { sourceCommit, sourceWorktreeDirty } = sourceState()
const cargoOutput = run(process.env.CARGO ?? 'cargo', [
  'test',
  '--locked',
  '-p',
  'oclive_kernel_host',
  '--lib',
  'infrastructure::runtime_event_trace::tests::duplicate_headers_are_idempotent_and_counted_without_replay_entry',
  '--',
  '--exact',
  '--nocapture',
  '--test-threads=1',
])

const markerPrefix = 'runtime-event-trace-duplicate-evidence:'
const markerLine = cargoOutput
  .split(/\r?\n/u)
  .find(line => line.includes(markerPrefix))
if (!markerLine)
  throw new Error('duplicate trace test did not emit its normalized evidence marker')
const markerText = markerLine.slice(
  markerLine.indexOf(markerPrefix) + markerPrefix.length,
).trim()
const marker = JSON.parse(markerText)
assertNoPrivateEvidence(marker, markerText)

const invariants = {
  both_dispatches_enqueued:
    marker.configured === true
    && marker.worker_active_during_sample === true
    && marker.enqueued_dispatches === expected.attempted_dispatches
    && marker.enqueued_events === expected.attempted_dispatches,
  both_dispatches_processed:
    marker.persisted_dispatches === expected.attempted_dispatches
    && marker.worker_stopped_after_shutdown === true,
  single_row_persisted:
    marker.persisted_events === expected.expected_inserted_events
    && marker.persisted_rows === expected.expected_persisted_rows
    && marker.last_persisted_position === expected.expected_persisted_rows,
  duplicate_counted:
    marker.duplicate_events === expected.expected_duplicate_events,
  no_dispatch_dropped:
    marker.dropped_dispatches === 0 && marker.dropped_events === 0,
  no_trace_failure: marker.failure_count === 0,
  privacy_flags_false:
    marker.captures_payloads === false
    && marker.captures_metadata === false
    && marker.captures_stream_key === false,
  test_only_no_production_replay:
    marker.test_compilation_only === true
    && marker.production_replay_entry === false,
}
for (const invariant of expected.expected_invariants) {
  if (invariants[invariant] !== true)
    throw new Error(`${expected.id} failed invariant ${invariant}`)
}
if (marker.schema_version !== 1
  || marker.attempted_dispatches !== expected.attempted_dispatches) {
  throw new Error('runtime event trace duplicate marker drifted from its contract')
}

const observed = {
  id: expected.id,
  action: expected.action,
  outcome: 'pass',
  attempted_dispatches: marker.attempted_dispatches,
  inserted_events: marker.persisted_events,
  duplicate_events: marker.duplicate_events,
  persisted_rows: marker.persisted_rows,
  diagnostics: {
    configured: marker.configured,
    worker_active_during_sample: marker.worker_active_during_sample,
    worker_stopped_after_shutdown: marker.worker_stopped_after_shutdown,
    enqueued_dispatches: marker.enqueued_dispatches,
    enqueued_events: marker.enqueued_events,
    persisted_dispatches: marker.persisted_dispatches,
    persisted_events: marker.persisted_events,
    duplicate_events: marker.duplicate_events,
    dropped_dispatches: marker.dropped_dispatches,
    dropped_events: marker.dropped_events,
    failure_count: marker.failure_count,
    last_persisted_position: marker.last_persisted_position,
    captures_payloads: marker.captures_payloads,
    captures_metadata: marker.captures_metadata,
    captures_stream_key: marker.captures_stream_key,
  },
  invariants,
}
const evidence = {
  schema_version: 1,
  evidence_kind: contract.evidence_kind,
  authoritative_runtime_stream: false,
  behavior_driver: false,
  synthetic_only: true,
  exports_raw_database: false,
  production_replay_entry: false,
  test_compilation_only: true,
  source_commit: sourceCommit,
  source_worktree_dirty: sourceWorktreeDirty,
  generated_at: new Date().toISOString(),
  scenario_contract: contractRelativePath,
  scenario_contract_sha256: createHash('sha256').update(contractText).digest('hex'),
  summary: {
    scenarios: 1,
    passed: 1,
    attempted_dispatches: marker.attempted_dispatches,
    inserted_events: marker.persisted_events,
    duplicate_events: marker.duplicate_events,
    persisted_rows: marker.persisted_rows,
  },
  scenarios: [observed],
  limitations: [
    'This evidence covers the test-compiled B0 trace recorder only.',
    'It does not add or validate a production replay entry point.',
    'It does not prove consumer delivery, consumer idempotency, or a Runtime Event Stream.',
  ],
}
assertNoPrivateEvidence(evidence)

const evidenceFileName = 'runtime-event-trace-shadow.duplicate-evidence.json'
const summaryFileName = 'runtime-event-trace-shadow.duplicate-summary.md'
const evidencePath = join(outputDir, evidenceFileName)
const summaryPath = join(outputDir, summaryFileName)
writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8')
writeFileSync(summaryPath, `# Runtime Event Trace duplicate evidence

- Result: PASS (1/1 synthetic scenario)
- Attempted dispatches: ${marker.attempted_dispatches}
- Inserted records: ${marker.persisted_events}
- Counted duplicates: ${marker.duplicate_events}
- Persisted rows: ${marker.persisted_rows}
- Boundary: test compilation only; no production replay; no behavior authority
- Source commit: ${sourceCommit}
- Source worktree dirty: ${sourceWorktreeDirty}
`, 'utf8')

const written = readPrivateEvidence(
  outputDir,
  evidenceFileName,
  summaryFileName,
)
assertPrivateEvidenceEnvelope({
  evidence: written.evidence,
  evidenceText: written.evidenceText,
  contract,
  contractText,
  contractRelativePath,
  sourceCommit,
  sourceWorktreeDirty,
})

console.log(cargoOutput)
console.log('runtime-event-trace-duplicate-samples: PASS (1 scenario)')
console.log(`evidence: ${written.evidencePath}`)
console.log(`summary: ${written.summaryPath}`)
