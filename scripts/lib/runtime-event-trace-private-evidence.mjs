import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, readdirSync, readFileSync } from 'node:fs'
import { dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

export const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..')

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

export function optionValue(name, fallback) {
  const index = process.argv.indexOf(name)
  if (index === -1)
    return fallback
  const value = process.argv[index + 1]
  if (!value || value.startsWith('--'))
    throw new Error(`${name} requires a value`)
  return value
}

export function run(program, args) {
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

export function readContract(contractRelativePath) {
  const contractPath = join(repoRoot, contractRelativePath)
  const contractText = readFileSync(contractPath, 'utf8')
  return {
    contractPath,
    contractText,
    contract: JSON.parse(contractText),
  }
}

export function uniqueScenarioIds(contract, context) {
  const ids = new Set(contract.scenarios.map(scenario => scenario.id))
  if (ids.size !== contract.scenarios.length)
    throw new Error(`${context} scenario ids must be unique`)
}

export function resolveTargetOutputDir(fallback) {
  const targetRoot = resolve(repoRoot, 'target')
  const outputDir = resolve(repoRoot, optionValue('--output-dir', fallback))
  if (outputDir !== targetRoot && !outputDir.startsWith(`${targetRoot}${sep}`))
    throw new Error('output directory must stay under repository target/')
  mkdirSync(outputDir, { recursive: true })
  return outputDir
}

export function sourceState() {
  return {
    sourceCommit: run('git', ['rev-parse', 'HEAD']),
    sourceWorktreeDirty: run('git', ['status', '--short']).length > 0,
  }
}

export function runSampler({
  mode,
  contractPath,
  outputDir,
  sourceCommit,
  sourceWorktreeDirty,
}) {
  return run(process.env.CARGO ?? 'cargo', [
    'run',
    '--locked',
    '--quiet',
    '-p',
    'oclive_kernel_host',
    '--example',
    'runtime_event_trace_sampler',
    '--',
    '--mode',
    mode,
    '--contract',
    contractPath,
    '--output-dir',
    outputDir,
    '--source-commit',
    sourceCommit,
    '--source-worktree-dirty',
    String(sourceWorktreeDirty),
  ])
}

export function readPrivateEvidence(outputDir, evidenceFileName, summaryFileName) {
  const expectedNames = [evidenceFileName, summaryFileName].sort()
  const observedEntries = readdirSync(outputDir, { withFileTypes: true })
  const observedNames = observedEntries.map(entry => entry.name).sort()
  if (observedEntries.some(entry => !entry.isFile())
    || JSON.stringify(observedNames) !== JSON.stringify(expectedNames)) {
    throw new Error(
      `private evidence directory must contain only ${expectedNames.join(', ')}`,
    )
  }
  const evidencePath = join(outputDir, evidenceFileName)
  const summaryPath = join(outputDir, summaryFileName)
  for (const path of [evidencePath, summaryPath]) {
    if (!existsSync(path))
      throw new Error(`sample collector did not create ${path}`)
  }
  const evidenceText = readFileSync(evidencePath, 'utf8')
  return {
    evidencePath,
    summaryPath,
    evidenceText,
    evidence: JSON.parse(evidenceText),
  }
}

export function assertPrivateEvidenceEnvelope({
  evidence,
  evidenceText,
  contract,
  contractText,
  contractRelativePath,
  sourceCommit,
  sourceWorktreeDirty,
}) {
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
    throw new Error('runtime event trace private evidence failed boundary validation')
  }
}

function hasForbiddenEvidenceKey(value) {
  if (Array.isArray(value))
    return value.some(hasForbiddenEvidenceKey)
  if (!value || typeof value !== 'object')
    return false
  return Object.entries(value).some(([key, nested]) => (
    forbiddenEvidenceKeys.has(key) || hasForbiddenEvidenceKey(nested)
  ))
}
