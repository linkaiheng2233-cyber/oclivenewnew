#!/usr/bin/env node

import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, readFileSync } from 'node:fs'
import { dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const contractRelativePath
  = 'kernel/crates/oclive_kernel_host/tests/fixtures/runtime_event_trace_shadow_scenarios.v1.json'
const contractPath = join(
  repoRoot,
  contractRelativePath,
)

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

function hasForbiddenEvidenceKey(value) {
  if (Array.isArray(value))
    return value.some(hasForbiddenEvidenceKey)
  if (!value || typeof value !== 'object')
    return false
  const forbidden = new Set([
    'payload',
    'metadata',
    'stream_key',
    'event_id',
    'occurred_at',
    'ingested_at',
    'observation',
    'user_message',
  ])
  return Object.entries(value).some(([key, nested]) => (
    forbidden.has(key) || hasForbiddenEvidenceKey(nested)
  ))
}

const contractText = readFileSync(contractPath, 'utf8')
const contract = JSON.parse(contractText)
if (contract.schema_version !== 1
  || contract.evidence_kind !== 'runtime_event_trace_shadow_synthetic_samples'
  || contract.authoritative_runtime_stream !== false
  || contract.behavior_driver !== false
  || contract.synthetic_only !== true
  || !Array.isArray(contract.scenarios)
  || contract.scenarios.length === 0) {
  throw new Error('invalid runtime event trace sample contract header')
}

const targetRoot = resolve(repoRoot, 'target')
const outputDir = resolve(
  repoRoot,
  optionValue('--output-dir', 'target/oclive-event/trace-shadow-samples'),
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
  '--contract',
  contractPath,
  '--output-dir',
  outputDir,
  '--source-commit',
  sourceCommit,
  '--source-worktree-dirty',
  String(sourceWorktreeDirty),
])

const evidencePath = join(outputDir, 'runtime-event-trace-shadow.evidence.json')
const summaryPath = join(outputDir, 'runtime-event-trace-shadow.summary.md')
const databasePath = join(outputDir, 'runtime-event-trace-shadow.samples.sqlite3')
for (const path of [evidencePath, summaryPath, databasePath]) {
  if (!existsSync(path))
    throw new Error(`sample collector did not create ${path}`)
}

const evidence = JSON.parse(readFileSync(evidencePath, 'utf8'))
const expectedRecordCount = contract.scenarios.reduce(
  (sum, scenario) => sum + scenario.expected_events.length,
  0,
)
const expectedContractHash = createHash('sha256').update(contractText).digest('hex')
if (evidence.schema_version !== 1
  || evidence.evidence_kind !== contract.evidence_kind
  || evidence.authoritative_runtime_stream !== false
  || evidence.behavior_driver !== false
  || evidence.synthetic_only !== true
  || evidence.source_commit !== sourceCommit
  || evidence.source_worktree_dirty !== sourceWorktreeDirty
  || evidence.scenario_contract !== contractRelativePath
  || evidence.scenario_contract_sha256 !== expectedContractHash
  || evidence.summary?.scenarios !== contract.scenarios.length
  || evidence.summary?.records !== expectedRecordCount
  || evidence.scenarios?.length !== contract.scenarios.length
  || evidence.observations?.positions_contiguous !== true
  || evidence.observations?.causation_resolved !== true
  || evidence.observations?.position_continues_after_restart !== true
  || evidence.observations?.ring_sequence_reset_after_restart !== true
  || evidence.observations?.privacy_minimized_schema !== true
  || hasForbiddenEvidenceKey(evidence)) {
  throw new Error('runtime event trace sample evidence failed boundary validation')
}

console.log(cargoOutput)
console.log(`runtime-event-trace-samples: PASS (${contract.scenarios.length} scenarios; ${expectedRecordCount} records)`)
console.log(`database: ${databasePath}`)
console.log(`evidence: ${evidencePath}`)
console.log(`summary: ${summaryPath}`)
