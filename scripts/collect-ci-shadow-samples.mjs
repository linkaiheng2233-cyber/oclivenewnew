#!/usr/bin/env node

import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const corpusPath = join(repoRoot, 'data/ci/shadow-scenarios.v1.json')
const nightlyJobs = new Set([
  'loom',
  'fuzz',
  'cli-bench',
  'visual-presentation-smoke',
  'e2e-tauri',
])

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

function ids(entries) {
  return entries.map(entry => entry.id)
}

function workflowJobs(validators) {
  return [...new Set(validators.flatMap(validator => validator.workflow_jobs))].sort()
}

function assertEqual(scenarioId, field, actual, expected) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `${scenarioId}.${field} drifted\nexpected: ${JSON.stringify(expected)}\nactual:   ${JSON.stringify(actual)}`,
    )
  }
}

const corpusText = readFileSync(corpusPath, 'utf8')
const corpus = JSON.parse(corpusText)
if (corpus.schema_version !== 1
  || corpus.evidence_kind !== 'planner_contract_simulation'
  || corpus.authoritative_ci_comparison !== false
  || !Array.isArray(corpus.scenarios)
  || corpus.scenarios.length === 0) {
  throw new Error('invalid shadow scenario corpus header')
}

const outputDir = resolve(
  repoRoot,
  optionValue('--output-dir', 'target/oclive-ci/shadow-samples'),
)
if (!outputDir.startsWith(`${repoRoot}${sep}`))
  throw new Error('output directory must stay inside the repository')
mkdirSync(outputDir, { recursive: true })

const headSha = run('git', ['rev-parse', 'HEAD'])
const baseSha = run('git', ['rev-parse', 'HEAD^'])
const results = []
for (const scenario of corpus.scenarios) {
  const expectsUnmapped = scenario.expected.fallback_reason_prefixes
    .some(prefix => prefix.startsWith('unmapped_changed_path:'))
  if (!expectsUnmapped) {
    for (const changedFile of scenario.changed_files) {
      if (!existsSync(join(repoRoot, changedFile)))
        throw new Error(`${scenario.id} references missing sample path ${changedFile}`)
    }
  }
  const planPath = join(outputDir, `${scenario.id}.plan.json`)
  const args = [
    'run', '--locked', '-p', 'oclive-cli', '--quiet', '--',
    'ci', 'plan',
    '--path', '.',
    '--base', baseSha,
    '--head', headSha,
    '--policy', scenario.policy,
    '--shadow',
    '--output', planPath,
  ]
  for (const changedFile of scenario.changed_files)
    args.push('--changed-file', changedFile)
  run(process.env.CARGO ?? 'cargo', args)

  const plan = JSON.parse(readFileSync(planPath, 'utf8'))
  const observed = {
    direct_modules: ids(plan.direct_modules),
    affected_modules: ids(plan.affected_modules),
    selected_validators: ids(plan.selected_validators),
    workflow_jobs: workflowJobs(plan.selected_validators),
    fallback_full: plan.fallback.full,
    fallback_reasons: plan.fallback.reasons,
  }
  for (const field of [
    'direct_modules',
    'affected_modules',
    'selected_validators',
    'workflow_jobs',
    'fallback_full',
  ]) {
    assertEqual(scenario.id, field, observed[field], scenario.expected[field])
  }
  assertEqual(
    scenario.id,
    'fallback_reason_count',
    observed.fallback_reasons.length,
    scenario.expected.fallback_reason_prefixes.length,
  )
  for (const prefix of scenario.expected.fallback_reason_prefixes) {
    if (!observed.fallback_reasons.some(reason => reason.startsWith(prefix)))
      throw new Error(`${scenario.id} missing fallback reason prefix ${prefix}`)
  }

  results.push({
    id: scenario.id,
    policy: scenario.policy,
    changed_files: scenario.changed_files,
    outcome: 'pass',
    review_note: scenario.review_note,
    selected_validator_count: observed.selected_validators.length,
    skipped_validator_count: plan.skipped_validators.length,
    main_workflow_jobs: observed.workflow_jobs.filter(job => !nightlyJobs.has(job)),
    nightly_workflow_jobs: observed.workflow_jobs.filter(job => nightlyJobs.has(job)),
    impact_map_sha256: plan.impact_map_sha256,
    validation_catalog_sha256: plan.validation_catalog_sha256,
    ...observed,
  })
}

const targeted = results.filter(result => !result.fallback_full)
const failSafe = results.filter(result => result.fallback_full)
for (const field of ['impact_map_sha256', 'validation_catalog_sha256']) {
  if (new Set(results.map(result => result[field])).size !== 1)
    throw new Error(`${field} changed while collecting one evidence set`)
}
const evidence = {
  schema_version: 1,
  evidence_kind: corpus.evidence_kind,
  authoritative_ci_comparison: false,
  generated_at: new Date().toISOString(),
  source_commit: headSha,
  scenario_contract: 'data/ci/shadow-scenarios.v1.json',
  scenario_contract_sha256: createHash('sha256').update(corpusText).digest('hex'),
  impact_map_sha256: results[0]?.impact_map_sha256,
  validation_catalog_sha256: results[0]?.validation_catalog_sha256,
  summary: {
    scenarios: results.length,
    targeted_plans: targeted.length,
    fail_safe_plans: failSafe.length,
    pull_request_scenarios: results.filter(result => result.policy === 'pull_request').length,
    nightly_scenarios: results.filter(result => result.policy === 'nightly').length,
    average_targeted_validator_count: Number((
      targeted.reduce((sum, result) => sum + result.selected_validator_count, 0)
        / Math.max(1, targeted.length)
    ).toFixed(2)),
    max_targeted_validator_count: Math.max(...targeted.map(result => result.selected_validator_count)),
  },
  limitations: [
    'This corpus validates deterministic routing and fail-safe behavior; it does not execute validators.',
    'This corpus is not a comparison against remote CI outcomes and cannot prove a zero false-negative rate.',
    'Over-selection notes are review evidence only; central impact edges remain authoritative until changed separately.',
  ],
  scenarios: results,
}

const jsonPath = join(outputDir, 'shadow-samples.evidence.json')
writeFileSync(jsonPath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8')

const markdown = [
  '# OCLive Shadow Scenario Evidence',
  '',
  `- Source commit: \`${headSha}\``,
  `- Scenarios: **${results.length}** (${targeted.length} targeted / ${failSafe.length} fail-safe)`,
  '- Authority: **simulation only** — no validator or remote CI outcome was executed by this collector.',
  '',
  '| Scenario | Policy | Validators | Main jobs | Nightly jobs | Fallback |',
  '|---|---:|---:|---:|---:|---|',
  ...results.map(result => `| ${result.id} | ${result.policy} | ${result.selected_validator_count} | ${result.main_workflow_jobs.length} | ${result.nightly_workflow_jobs.length} | ${result.fallback_full ? 'full' : 'targeted'} |`),
  '',
  '## Review notes',
  '',
  ...results.map(result => `- **${result.id}**: ${result.review_note}`),
  '',
  'Real Stage 2 Compare evidence must still bind a plan to the final status of every actually executed remote job.',
  '',
].join('\n')
const markdownPath = join(outputDir, 'shadow-samples.summary.md')
writeFileSync(markdownPath, markdown, 'utf8')

console.log(`shadow-samples: PASS (${results.length} scenarios; ${targeted.length} targeted; ${failSafe.length} fail-safe)`)
console.log(`evidence: ${jsonPath}`)
console.log(`summary: ${markdownPath}`)
