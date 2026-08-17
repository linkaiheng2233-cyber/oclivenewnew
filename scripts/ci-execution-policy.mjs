#!/usr/bin/env node

import assert from 'node:assert/strict';
import fs from 'node:fs';

const POLICY_ID = 'domain-aware-pr-v2';
const PLAN_JOB = 'ci-impact-plan';
const PROMOTED_READY_DIRECT_MODULES = new Set([
  'oclive.docs',
]);
const SELECTIVE_REASONS = new Set(['draft_pr_selective', 'promoted_domain_canary']);

function uniqueSorted(values) {
  return [...new Set(values)].sort();
}

function parseArgs(argv) {
  const options = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith('--')) {
      throw new Error(`unexpected argument: ${token}`);
    }
    const value = argv[index + 1];
    if (!value || value.startsWith('--')) {
      options.set(token, true);
      continue;
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

function moduleIds(values, field) {
  if (!Array.isArray(values)) {
    throw new Error(`plan.${field} must be an array`);
  }
  return uniqueSorted(
    values.map((value) => {
      if (!value || typeof value.id !== 'string' || value.id.length === 0) {
        throw new Error(`plan.${field} contains an invalid module`);
      }
      return value.id;
    }),
  );
}

function selectedWorkflowJobs(plan) {
  if (!Array.isArray(plan.selected_validators)) {
    throw new Error('plan.selected_validators must be an array');
  }
  return uniqueSorted(
    plan.selected_validators.flatMap((validator) => {
      if (!validator || !Array.isArray(validator.workflow_jobs)) {
        throw new Error('selected validator is missing workflow_jobs');
      }
      return validator.workflow_jobs.map((job) => {
        if (typeof job !== 'string' || job.length === 0) {
          throw new Error('selected validator contains an invalid workflow job');
        }
        return job;
      });
    }),
  );
}

function isPromotedReadyScope(values) {
  return values.length > 0 && values.every((value) => PROMOTED_READY_DIRECT_MODULES.has(value));
}

function resolveExecution(plan, eventName, forceFullReason = null, prDraft = false) {
  if (!plan || typeof plan !== 'object') {
    throw new Error('plan must be an object');
  }
  if (!plan.fallback || typeof plan.fallback.full !== 'boolean') {
    throw new Error('plan.fallback.full must be a boolean');
  }
  if (!Array.isArray(plan.warnings)) {
    throw new Error('plan.warnings must be an array');
  }

  const directModules = moduleIds(plan.direct_modules, 'direct_modules');
  const affectedModules = moduleIds(plan.affected_modules, 'affected_modules');
  const selectedJobs = selectedWorkflowJobs(plan);

  let reason = forceFullReason ?? 'promoted_domain_canary';
  if (forceFullReason !== null) {
    if (!/^[a-z0-9_]+$/.test(forceFullReason)) {
      throw new Error('force-full reason must use lowercase snake_case');
    }
  } else if (eventName !== 'pull_request') {
    reason = 'event_not_pull_request';
  } else if (plan.policy !== 'pull_request') {
    reason = 'planner_policy_not_pull_request';
  } else if (plan.shadow !== false) {
    reason = 'planner_shadow_mode';
  } else if (plan.fallback.full) {
    reason = 'planner_full_fallback';
  } else if (plan.warnings.length > 0) {
    reason = 'planner_warning';
  } else if (selectedJobs.length === 0) {
    reason = 'no_selected_workflow_jobs';
  } else if (directModules.length === 0 || affectedModules.length === 0) {
    reason = 'planner_empty_module_scope';
  } else if (prDraft) {
    reason = 'draft_pr_selective';
  } else if (!isPromotedReadyScope(directModules)) {
    reason = 'ready_pr_domain_not_promoted';
  }

  const runFull = !SELECTIVE_REASONS.has(reason);
  return {
    schema_version: 1,
    policy: POLICY_ID,
    event: eventName,
    pr_draft: prDraft,
    mode: runFull ? 'full' : 'selective',
    run_full: runFull,
    reason,
    selected_jobs: selectedJobs,
  };
}

function writeExecutionOutputs(execution, githubOutputPath) {
  const lines = [
    `run_full=${execution.run_full}`,
    `selected_jobs=${JSON.stringify(execution.selected_jobs)}`,
    `mode=${execution.mode}`,
    `reason=${execution.reason}`,
  ];
  fs.appendFileSync(githubOutputPath, `${lines.join('\n')}\n`, 'utf8');
}

function appendExecutionSummary(execution, githubSummaryPath) {
  const jobs = execution.selected_jobs.length > 0 ? execution.selected_jobs.join(', ') : '(none)';
  const summary = [
    '',
    '### Execution policy',
    '',
    `- Policy: \`${execution.policy}\``,
    `- Mode: **${execution.mode}**`,
    `- Reason: \`${execution.reason}\``,
    `- Planner-selected jobs: ${jobs}`,
    '',
  ].join('\n');
  fs.appendFileSync(githubSummaryPath, summary, 'utf8');
}

function verifyGate(needs) {
  if (!needs || typeof needs !== 'object' || Array.isArray(needs)) {
    throw new Error('needs must be an object');
  }
  const planner = needs[PLAN_JOB];
  if (!planner || planner.result !== 'success') {
    throw new Error(`impact planner did not succeed (result=${planner?.result ?? 'missing'})`);
  }

  const outputs = planner.outputs ?? {};
  if (outputs.run_full !== 'true' && outputs.run_full !== 'false') {
    throw new Error(`invalid run_full output: ${outputs.run_full ?? 'missing'}`);
  }
  const runFull = outputs.run_full === 'true';
  let selectedJobs;
  try {
    selectedJobs = uniqueSorted(JSON.parse(outputs.selected_jobs ?? ''));
  } catch (error) {
    throw new Error(`invalid selected_jobs output: ${error.message}`);
  }
  if (!Array.isArray(selectedJobs) || selectedJobs.some((job) => typeof job !== 'string')) {
    throw new Error('selected_jobs output must be a JSON string array');
  }

  const validationJobs = Object.keys(needs).filter((job) => job !== PLAN_JOB).sort();
  for (const selectedJob of selectedJobs) {
    if (!validationJobs.includes(selectedJob)) {
      throw new Error(`selected workflow job is not wired into ci-gate: ${selectedJob}`);
    }
  }

  for (const job of validationJobs) {
    const expectedToRun = runFull || selectedJobs.includes(job);
    const expectedResult = expectedToRun ? 'success' : 'skipped';
    const actualResult = needs[job]?.result ?? 'missing';
    if (actualResult !== expectedResult) {
      throw new Error(
        `${job} result mismatch: expected ${expectedResult}, received ${actualResult}`,
      );
    }
  }

  return {
    mode: runFull ? 'full' : 'selective',
    selected_jobs: selectedJobs,
    validated_jobs: validationJobs,
  };
}

function basePlan(overrides = {}) {
  return {
    policy: 'pull_request',
    shadow: false,
    direct_modules: [{ id: 'oclive.docs' }],
    affected_modules: [{ id: 'oclive.docs' }],
    selected_validators: [
      { workflow_jobs: ['dimension5-acceptance'] },
      { workflow_jobs: ['stale-paths'] },
    ],
    fallback: { full: false, reasons: [] },
    warnings: [],
    ...overrides,
  };
}

function runSelfTest() {
  const selective = resolveExecution(basePlan(), 'pull_request');
  assert.equal(selective.mode, 'selective');
  assert.equal(selective.reason, 'promoted_domain_canary');
  assert.deepEqual(selective.selected_jobs, ['dimension5-acceptance', 'stale-paths']);

  assert.equal(resolveExecution(basePlan(), 'push').reason, 'event_not_pull_request');
  assert.equal(
    resolveExecution(basePlan(), 'pull_request', 'trusted_policy_bootstrap').reason,
    'trusted_policy_bootstrap',
  );
  assert.equal(resolveExecution(basePlan({ shadow: true }), 'pull_request').reason, 'planner_shadow_mode');
  assert.equal(
    resolveExecution(basePlan({ fallback: { full: true, reasons: ['risk'] } }), 'pull_request')
      .reason,
    'planner_full_fallback',
  );
  assert.equal(
    resolveExecution(basePlan({ warnings: ['unknown optional extension'] }), 'pull_request', null, true)
      .reason,
    'planner_warning',
  );
  assert.equal(
    resolveExecution(
      basePlan({ fallback: { full: true, reasons: ['risk'] } }),
      'pull_request',
      null,
      true,
    ).mode,
    'full',
  );
  const readyCodePlan = basePlan({
    direct_modules: [{ id: 'oclive.cli' }],
    affected_modules: [{ id: 'oclive.cli' }],
  });
  assert.equal(
    resolveExecution(readyCodePlan, 'pull_request').reason,
    'ready_pr_domain_not_promoted',
  );
  assert.equal(resolveExecution(readyCodePlan, 'pull_request', null, true).reason, 'draft_pr_selective');
  assert.equal(resolveExecution(readyCodePlan, 'pull_request', null, true).mode, 'selective');

  const unpromotedLowRiskPlan = basePlan({
    direct_modules: [{ id: 'oclive.docs' }, { id: 'oclive.scaffold' }],
    affected_modules: [
      { id: 'oclive.cli' },
      { id: 'oclive.docs' },
      { id: 'oclive.scaffold' },
    ],
  });
  assert.equal(
    resolveExecution(unpromotedLowRiskPlan, 'pull_request').reason,
    'ready_pr_domain_not_promoted',
  );
  assert.equal(
    resolveExecution(basePlan({ selected_validators: [] }), 'pull_request').reason,
    'no_selected_workflow_jobs',
  );

  const selectiveNeeds = {
    [PLAN_JOB]: {
      result: 'success',
      outputs: {
        run_full: 'false',
        selected_jobs: '["dimension5-acceptance","stale-paths"]',
      },
    },
    rust: { result: 'skipped' },
    'dimension5-acceptance': { result: 'success' },
    'stale-paths': { result: 'success' },
  };
  assert.equal(verifyGate(selectiveNeeds).mode, 'selective');

  const fullNeeds = structuredClone(selectiveNeeds);
  fullNeeds[PLAN_JOB].outputs.run_full = 'true';
  fullNeeds.rust.result = 'success';
  assert.equal(verifyGate(fullNeeds).mode, 'full');

  const plannerFailure = structuredClone(fullNeeds);
  plannerFailure[PLAN_JOB].result = 'failure';
  assert.throws(() => verifyGate(plannerFailure), /impact planner did not succeed/);

  const selectedFailure = structuredClone(selectiveNeeds);
  selectedFailure['stale-paths'].result = 'failure';
  assert.throws(() => verifyGate(selectedFailure), /stale-paths result mismatch/);

  const unexpectedExecution = structuredClone(selectiveNeeds);
  unexpectedExecution.rust.result = 'success';
  assert.throws(() => verifyGate(unexpectedExecution), /rust result mismatch/);

  console.log('ci-execution-policy: self-test PASS');
}

function runResolve(options) {
  const planPath = requireString(options, '--plan');
  const eventName = requireString(options, '--event');
  const outputPath = requireString(options, '--output');
  const githubOutputPath = requireString(options, '--github-output');
  const plan = JSON.parse(fs.readFileSync(planPath, 'utf8'));
  const forceFullReason = options.get('--force-full-reason') ?? null;
  if (forceFullReason !== null && typeof forceFullReason !== 'string') {
    throw new Error('--force-full-reason requires a value');
  }
  const prDraftValue = options.get('--pr-draft') ?? 'false';
  if (prDraftValue !== 'true' && prDraftValue !== 'false') {
    throw new Error('--pr-draft must be true or false');
  }
  const execution = resolveExecution(plan, eventName, forceFullReason, prDraftValue === 'true');
  fs.writeFileSync(outputPath, `${JSON.stringify(execution, null, 2)}\n`, 'utf8');
  writeExecutionOutputs(execution, githubOutputPath);
  const githubSummaryPath = options.get('--github-summary');
  if (typeof githubSummaryPath === 'string' && githubSummaryPath.length > 0) {
    appendExecutionSummary(execution, githubSummaryPath);
  }
  console.log(JSON.stringify(execution));
}

function runVerify(options) {
  const needsEnv = options.get('--needs-env') ?? 'NEEDS_JSON';
  if (typeof needsEnv !== 'string' || needsEnv.length === 0) {
    throw new Error('--needs-env must name an environment variable');
  }
  const rawNeeds = process.env[needsEnv];
  if (!rawNeeds) {
    throw new Error(`missing ${needsEnv} environment variable`);
  }
  const result = verifyGate(JSON.parse(rawNeeds));
  console.log(
    `ci-gate PASS (${result.mode}; selected=${result.selected_jobs.join(',') || 'none'}; wired=${result.validated_jobs.length})`,
  );
}

function main() {
  const [command, ...rest] = process.argv.slice(2);
  if (command === '--self-test') {
    runSelfTest();
    return;
  }
  const options = parseArgs(rest);
  if (command === 'resolve') {
    runResolve(options);
    return;
  }
  if (command === 'verify') {
    runVerify(options);
    return;
  }
  throw new Error('usage: ci-execution-policy.mjs <resolve|verify|--self-test>');
}

try {
  main();
} catch (error) {
  console.error(`ci-execution-policy: ${error.message}`);
  process.exitCode = 1;
}
