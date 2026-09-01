#!/usr/bin/env node

import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, readFileSync } from 'node:fs'
import { dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const contractRelativePath
  = 'kernel/crates/oclive_kernel_host/tests/fixtures/runtime_event_trace_shadow_fault_scenarios.v1.json'
const contractPath = join(repoRoot, contractRelativePath)

function optionValue(name, fallback) {
  const index = process.argv.indexOf(name)
  if (index === -1)
    return fallback
  const value = process.argv[index + 1]
  if (!value || value.startsWith('--'))
    throw new Error(`${name} requires a value`)
  return value
}

function run(program, args) {
  const result = spawnSync(program, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    windowsHide: true,
  })
  if (result.status !== 0) {
    process.stderr.write(result.stdout ?? '')
    process.stderr.write(result.stderr ?? '')
    throw new Error(`${program} exited with ${result.status ?? 'no status'}`)
  }
  return (result.stdout ?? '').trim()
}

const forbiddenEvidenceKeys = new Set([
  'payload',
  'metadata',
  'stream_key',
  'event_id',
  'correlation_id',
  'occurred_at',
  'ingested_at',
  'observation',
  'user_message',
  'role_id',
  'session_id',
  'scene_id',
  'trace_path',
  'database_path',
  'raw_trace_database',
])

function hasForbiddenEvidenceKey(value) {
  if (Array.isArray(value))
    return value.some(hasForbiddenEvidenceKey)
  if (!value || typeof value !== 'object')
    return false
  return Object.entries(value).some(([key, nested]) => (
    forbiddenEvidenceKeys.has(key) || hasForbiddenEvidenceKey(nested)
  ))
}

const contractText = readFileSync(contractPath, 'utf8')
const contract = JSON.parse(contractText)
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

const scenarioIds = new Set(contract.scenarios.map(scenario => scenario.id))
if (scenarioIds.size !== contract.scenarios.length)
  throw new Error('runtime event trace fault scenario ids must be unique')

const targetRoot = resolve(repoRoot, 'target')
const outputDir = resolve(
  repoRoot,
  optionValue('--output-dir', 'target/oclive-event/trace-shadow-fault-samples'),
)
if (outputDir !== targetRoot && !outputDir.startsWith(`${targetRoot}${sep}`))
  throw new Error('output directory must stay under repository target/')
mkdirSync(outputDir, { recursive: true })

const sourceCommit = run('git', ['rev-parse', 'HEAD'])
const sourceWorktreeDirty = run('git', ['status', '--short']).length > 0
const cargoOutput = run(process.env.CARGO ?? 'cargo', [
  'run',
  '--locked',
  '--quiet',
  '-p',
  'oclive_kernel_host',
  '--example',
  'runtime_event_trace_sampler',
  '--',
  '--mode',
  'fault',
  '--contract',
  contractPath,
  '--output-dir',
  outputDir,
  '--source-commit',
  sourceCommit,
  '--source-worktree-dirty',
  String(sourceWorktreeDirty),
])

const evidencePath = join(outputDir, 'runtime-event-trace-shadow.fault-evidence.json')
const summaryPath = join(outputDir, 'runtime-event-trace-shadow.fault-summary.md')
for (const path of [evidencePath, summaryPath]) {
  if (!existsSync(path))
    throw new Error(`fault sample collector did not create ${path}`)
}

const evidenceText = readFileSync(evidencePath, 'utf8')
const evidence = JSON.parse(evidenceText)
const expectedContractHash = createHash('sha256').update(contractText).digest('hex')
if (evidence.schema_version !== 1
  || evidence.evidence_kind !== contract.evidence_kind
  || evidence.authoritative_runtime_stream !== false
  || evidence.behavior_driver !== false
  || evidence.synthetic_only !== true
  || evidence.exports_raw_database !== false
  || evidence.source_commit !== sourceCommit
  || evidence.source_worktree_dirty !== sourceWorktreeDirty
  || evidence.scenario_contract !== contractRelativePath
  || evidence.scenario_contract_sha256 !== expectedContractHash
  || evidence.summary?.scenarios !== contract.scenarios.length
  || evidence.summary?.passed !== contract.scenarios.length
  || evidence.scenarios?.length !== contract.scenarios.length
  || hasForbiddenEvidenceKey(evidence)
  || evidenceText.includes('must-not-be-exported')) {
  throw new Error('runtime event trace fault evidence failed boundary validation')
}

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
