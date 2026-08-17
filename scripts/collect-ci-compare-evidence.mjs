#!/usr/bin/env node

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const PLAN_JOB = 'ci-impact-plan';
const GATE_JOB_NAMES = new Set(['ci-gate', 'ci-draft-gate']);
const CONCLUSIVE_RESULTS = new Set(['success', 'failure']);

function parseArgs(argv) {
  const options = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith('--')) {
      throw new Error(`unexpected argument: ${token}`);
    }
    const value = argv[index + 1];
    if (!value || value.startsWith('--')) {
      throw new Error(`${token} requires a value`);
    }
    options.set(token, value);
    index += 1;
  }
  return options;
}

function requireString(options, key) {
  const value = options.get(key);
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`missing required option ${key}`);
  }
  return value;
}

function readJson(filePath, label) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    throw new Error(`read ${label} from ${filePath}: ${error.message}`);
  }
}

function readOptionalJson(filePath, label) {
  if (!filePath || !fs.existsSync(filePath)) {
    return null;
  }
  return readJson(filePath, label);
}

function uniqueSorted(values) {
  return [...new Set(values)].sort();
}

function ids(values) {
  if (!Array.isArray(values)) {
    return [];
  }
  return values
    .map((value) => value?.id)
    .filter((value) => typeof value === 'string' && value.length > 0);
}

function workflowJobs(validators) {
  if (!Array.isArray(validators)) {
    return [];
  }
  return uniqueSorted(
    validators.flatMap((validator) =>
      Array.isArray(validator?.workflow_jobs) ? validator.workflow_jobs : [],
    ),
  );
}

function stable(value) {
  if (Array.isArray(value)) {
    return value.map(stable);
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, stable(value[key])]),
    );
  }
  return value;
}

function sha256(value) {
  return createHash('sha256').update(JSON.stringify(stable(value))).digest('hex');
}

function logicalJobName(name) {
  if (typeof name !== 'string') {
    return '';
  }
  const matrix = name.match(/^(.+?) \(.+\)$/);
  return matrix ? matrix[1] : name;
}

function normalizeRemoteJobs(snapshot, runAttempt) {
  const jobs = Array.isArray(snapshot?.jobs) ? snapshot.jobs : [];
  return jobs
    .filter((job) => !job.run_attempt || String(job.run_attempt) === String(runAttempt))
    .filter((job) => !GATE_JOB_NAMES.has(job.name))
    .map((job) => ({
      id: job.id ?? null,
      name: job.name ?? '',
      logical_job: logicalJobName(job.name),
      status: job.status ?? 'unknown',
      conclusion: job.conclusion ?? null,
      started_at: job.started_at ?? null,
      completed_at: job.completed_at ?? null,
      runner_name: job.runner_name ?? null,
      labels: Array.isArray(job.labels) ? job.labels : [],
      run_attempt: job.run_attempt ?? Number(runAttempt),
    }))
    .sort((left, right) =>
      `${left.logical_job}\0${left.name}\0${left.id}`.localeCompare(
        `${right.logical_job}\0${right.name}\0${right.id}`,
      ),
    );
}

function collectEvidence({ plan, execution, needs, jobsSnapshot, metadata }) {
  if (!plan || typeof plan !== 'object') {
    throw new Error('plan must be an object');
  }
  if (!execution || typeof execution !== 'object') {
    throw new Error('execution must be an object');
  }
  if (!needs || typeof needs !== 'object' || Array.isArray(needs)) {
    throw new Error('needs must be an object');
  }

  const selectedJobs = workflowJobs(plan.selected_validators);
  const validationJobs = Object.keys(needs)
    .filter((job) => job !== PLAN_JOB)
    .sort();
  const logicalResults = validationJobs.map((job) => ({
    job,
    selected_by_plan: selectedJobs.includes(job),
    result: needs[job]?.result ?? 'missing',
  }));
  const unselectedResults = logicalResults.filter((job) => !job.selected_by_plan);
  const falseNegativeCandidates = execution.run_full === true
    ? unselectedResults.filter((job) => job.result === 'failure').map((job) => job.job)
    : [];
  const inconclusiveWouldSkip = execution.run_full === true
    ? unselectedResults
        .filter((job) => !CONCLUSIVE_RESULTS.has(job.result))
        .map((job) => job.job)
    : [];

  const remoteJobs = normalizeRemoteJobs(jobsSnapshot, metadata.run_attempt);
  const snapshotLogicalJobs = new Set(remoteJobs.map((job) => job.logical_job));
  const snapshotComplete = remoteJobs.length > 0
    && [PLAN_JOB, ...validationJobs].every((job) => snapshotLogicalJobs.has(job))
    && remoteJobs.every((job) => job.status === 'completed');
  const conclusiveFullResults = execution.run_full === true
    && logicalResults.every((job) => CONCLUSIVE_RESULTS.has(job.result));
  const headMatches = plan.head_sha === metadata.workflow_sha;
  const authoritative = execution.run_full === true
    && needs[PLAN_JOB]?.result === 'success'
    && snapshotComplete
    && conclusiveFullResults
    && headMatches;

  const routeShape = {
    policy: plan.policy,
    changed_files: uniqueSorted(plan.changed_files ?? []),
    direct_modules: ids(plan.direct_modules),
    affected_modules: ids(plan.affected_modules),
    selected_validators: ids(plan.selected_validators),
    selected_workflow_jobs: selectedJobs,
    fallback_full: plan.fallback?.full ?? null,
    fallback_reasons: plan.fallback?.reasons ?? [],
    warnings: plan.warnings ?? [],
    impact_map_sha256: plan.impact_map_sha256 ?? null,
    validation_catalog_sha256: plan.validation_catalog_sha256 ?? null,
  };
  const sampleShape = {
    repository: metadata.repository,
    run_id: metadata.run_id,
    run_attempt: metadata.run_attempt,
    workflow_sha: metadata.workflow_sha,
    plan_head_sha: plan.head_sha,
    route: routeShape,
  };

  return {
    schema_version: 1,
    evidence_kind: execution.run_full === true
      ? 'remote_ci_compare'
      : 'remote_ci_selective_execution',
    authoritative_ci_comparison: authoritative,
    generated_at: new Date().toISOString(),
    source: {
      repository: metadata.repository,
      event: metadata.event,
      run_id: metadata.run_id,
      run_attempt: metadata.run_attempt,
      run_url: metadata.run_url,
      ref: metadata.ref,
      workflow_sha: metadata.workflow_sha,
      source_head_sha: metadata.source_head_sha,
      pull_request: metadata.pull_request,
      plan_base_sha: plan.base_sha ?? null,
      plan_head_sha: plan.head_sha ?? null,
    },
    fingerprints: {
      sample_id: sha256(sampleShape),
      route_fingerprint: sha256(routeShape),
    },
    plan: routeShape,
    execution: {
      policy: execution.policy ?? null,
      event: execution.event ?? null,
      pr_draft: execution.pr_draft ?? false,
      mode: execution.mode ?? null,
      run_full: execution.run_full ?? null,
      reason: execution.reason ?? null,
      selected_jobs: execution.selected_jobs ?? [],
    },
    comparison: {
      eligible_full_run: execution.run_full === true,
      head_matches_workflow: headMatches,
      remote_job_snapshot_complete: snapshotComplete,
      conclusive_full_results: conclusiveFullResults,
      selected_job_results: logicalResults.filter((job) => job.selected_by_plan),
      would_skip_job_results: unselectedResults,
      false_negative_candidates: falseNegativeCandidates,
      inconclusive_would_skip_jobs: inconclusiveWouldSkip,
    },
    logical_job_results: logicalResults,
    remote_jobs: remoteJobs,
    limitations: [
      'A false-negative candidate requires maintainer review before changing the impact graph.',
      'Successful would-skip jobs are evidence for a route, not proof that every future diff in that domain is safe.',
      'Selective executions do not validate skipped jobs and are never authoritative Compare samples.',
    ],
  };
}

function renderSummary(evidence) {
  const wouldSkip = evidence.comparison.would_skip_job_results;
  const candidates = evidence.comparison.false_negative_candidates;
  const status = evidence.authoritative_ci_comparison ? 'authoritative Compare' : 'non-authoritative';
  return [
    '# OCLive CI Compare Evidence',
    '',
    `- Run: [${evidence.source.run_id}](${evidence.source.run_url}) attempt ${evidence.source.run_attempt}`,
    `- Workflow SHA: \`${evidence.source.workflow_sha}\``,
    `- Plan base/head: \`${evidence.source.plan_base_sha}\` → \`${evidence.source.plan_head_sha}\``,
    `- Execution: **${evidence.execution.mode}** (\`${evidence.execution.reason}\`)`,
    `- Evidence status: **${status}**`,
    `- Route fingerprint: \`${evidence.fingerprints.route_fingerprint}\``,
    '',
    '## Would-skip outcomes',
    '',
    wouldSkip.length === 0
      ? '- None.'
      : wouldSkip.map((job) => `- \`${job.job}\`: **${job.result}**`).join('\n'),
    '',
    '## False-negative candidates',
    '',
    candidates.length === 0
      ? '- None observed.'
      : candidates.map((job) => `- \`${job}\``).join('\n'),
    '',
    'This artifact binds one planner result to the terminal jobs visible for the same workflow run. It does not generalize beyond this diff.',
    '',
  ].join('\n');
}

function writeFile(filePath, contents) {
  fs.mkdirSync(path.dirname(path.resolve(filePath)), { recursive: true });
  fs.writeFileSync(filePath, contents, 'utf8');
}

function runSelfTest() {
  const plan = {
    base_sha: 'base',
    head_sha: 'head',
    policy: 'pull_request',
    changed_files: ['kernel/example.rs'],
    direct_modules: [{ id: 'oclive.kernel-runtime' }],
    affected_modules: [{ id: 'oclive.kernel-runtime' }],
    selected_validators: [{ id: 'rust-workspace', workflow_jobs: ['rust'] }],
    fallback: { full: false, reasons: [] },
    warnings: [],
    impact_map_sha256: 'a'.repeat(64),
    validation_catalog_sha256: 'b'.repeat(64),
  };
  const execution = {
    policy: 'domain-aware-pr-v2',
    event: 'pull_request',
    mode: 'full',
    run_full: true,
    reason: 'ready_pr_domain_not_promoted',
    selected_jobs: ['rust'],
  };
  const needs = {
    [PLAN_JOB]: { result: 'success' },
    rust: { result: 'success' },
    frontend: { result: 'failure' },
  };
  const jobsSnapshot = {
    jobs: [
      { id: 1, name: PLAN_JOB, status: 'completed', conclusion: 'success', run_attempt: 1 },
      { id: 2, name: 'rust (ubuntu-22.04)', status: 'completed', conclusion: 'success', run_attempt: 1 },
      { id: 3, name: 'frontend (ubuntu-latest)', status: 'completed', conclusion: 'failure', run_attempt: 1 },
    ],
  };
  const metadata = {
    repository: 'owner/repo',
    event: 'pull_request',
    run_id: '42',
    run_attempt: '1',
    run_url: 'https://example.invalid/runs/42',
    ref: 'refs/pull/1/merge',
    workflow_sha: 'head',
    source_head_sha: 'source',
    pull_request: '1',
  };
  const evidence = collectEvidence({ plan, execution, needs, jobsSnapshot, metadata });
  assert.equal(evidence.authoritative_ci_comparison, true);
  assert.deepEqual(evidence.comparison.false_negative_candidates, ['frontend']);
  assert.equal(evidence.comparison.would_skip_job_results[0].result, 'failure');

  const selective = collectEvidence({
    plan,
    execution: { ...execution, mode: 'selective', run_full: false, reason: 'draft_pr_selective' },
    needs: {
      [PLAN_JOB]: { result: 'success' },
      rust: { result: 'success' },
      frontend: { result: 'skipped' },
    },
    jobsSnapshot: {
      jobs: [
        jobsSnapshot.jobs[0],
        jobsSnapshot.jobs[1],
        { ...jobsSnapshot.jobs[2], conclusion: 'skipped' },
      ],
    },
    metadata,
  });
  assert.equal(selective.authoritative_ci_comparison, false);
  assert.deepEqual(selective.comparison.false_negative_candidates, []);
  console.log('ci-compare-evidence: self-test PASS');
}

function runCollect(options) {
  const needsEnv = options.get('--needs-env') ?? 'NEEDS_JSON';
  const needsRaw = process.env[needsEnv];
  if (!needsRaw) {
    throw new Error(`missing ${needsEnv} environment variable`);
  }
  const metadata = {
    repository: requireString(options, '--repository'),
    event: requireString(options, '--event'),
    run_id: requireString(options, '--run-id'),
    run_attempt: requireString(options, '--run-attempt'),
    run_url: requireString(options, '--run-url'),
    ref: requireString(options, '--ref'),
    workflow_sha: requireString(options, '--workflow-sha'),
    source_head_sha: requireString(options, '--source-head-sha'),
    pull_request: options.get('--pull-request') ?? '',
  };
  const evidence = collectEvidence({
    plan: readJson(requireString(options, '--plan'), 'plan'),
    execution: readJson(requireString(options, '--execution'), 'execution'),
    needs: JSON.parse(needsRaw),
    jobsSnapshot: readOptionalJson(options.get('--jobs'), 'remote jobs'),
    metadata,
  });
  const output = requireString(options, '--output');
  const summaryOutput = requireString(options, '--summary-output');
  writeFile(output, `${JSON.stringify(evidence, null, 2)}\n`);
  const summary = renderSummary(evidence);
  writeFile(summaryOutput, summary);
  const githubSummary = options.get('--github-summary');
  if (githubSummary) {
    fs.appendFileSync(githubSummary, `\n${summary}`, 'utf8');
  }
  console.log(
    `ci-compare-evidence: PASS (${evidence.authoritative_ci_comparison ? 'authoritative' : 'observational'}; false-negative-candidates=${evidence.comparison.false_negative_candidates.length})`,
  );
}

function main() {
  const [command, ...rest] = process.argv.slice(2);
  if (command === '--self-test') {
    runSelfTest();
    return;
  }
  if (command === 'collect') {
    runCollect(parseArgs(rest));
    return;
  }
  throw new Error('usage: collect-ci-compare-evidence.mjs <collect|--self-test>');
}

try {
  main();
} catch (error) {
  console.error(`ci-compare-evidence: ${error.message}`);
  process.exitCode = 1;
}
