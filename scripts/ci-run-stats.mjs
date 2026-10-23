#!/usr/bin/env node
// Weekly wall-clock statistics for the main CI workflow.
//
// Usage:
//   node scripts/ci-run-stats.mjs [--days 30] [--branch main] [--limit 300] [--repo owner/name]
//
// Requires the GitHub CLI (`gh`) to be authenticated. The script is read-only and prints a
// Markdown table; it also lists the five slowest runs so infrastructure stalls are visible.

import { execFileSync } from 'node:child_process';

function parseArgs(argv) {
  const options = { days: 30, branch: 'main', limit: 300, repo: null };
  for (let index = 0; index < argv.length; index += 1) {
    const name = argv[index];
    const value = argv[index + 1];
    if (name === '--days') options.days = Number(value);
    if (name === '--branch') options.branch = value;
    if (name === '--limit') options.limit = Number(value);
    if (name === '--repo') options.repo = value;
  }
  if (!Number.isFinite(options.days) || options.days <= 0) throw new Error('--days must be positive');
  return options;
}

function fetchRuns(options) {
  const args = [
    'run',
    'list',
    '--branch',
    options.branch,
    '--limit',
    String(options.limit),
    '--json',
    'databaseId,conclusion,createdAt,updatedAt,status,event,displayTitle',
  ];
  if (options.repo) args.push('--repo', options.repo);
  const output = execFileSync('gh', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  return JSON.parse(output);
}

function weekStart(date) {
  const copy = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()));
  const weekday = (copy.getUTCDay() + 6) % 7;
  copy.setUTCDate(copy.getUTCDate() - weekday);
  return copy.toISOString().slice(0, 10);
}

function minutes(ms) {
  return Math.round((ms / 60000) * 10) / 10;
}

function percentile(values, ratio) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.floor((sorted.length - 1) * ratio));
  return sorted[index];
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const cutoff = Date.now() - options.days * 24 * 3600 * 1000;
  const runs = fetchRuns(options)
    .filter(run => run.createdAt && Date.parse(run.createdAt) >= cutoff)
    .filter(run => run.status === 'completed');

  const byWeek = new Map();
  for (const run of runs) {
    const week = weekStart(new Date(run.createdAt));
    const bucket = byWeek.get(week) ?? { runs: [], success: 0, failure: 0 };
    bucket.runs.push(run);
    if (run.conclusion === 'success') bucket.success += 1;
    else if (run.conclusion === 'failure') bucket.failure += 1;
    byWeek.set(week, bucket);
  }

  console.log(`| week (UTC) | runs | success | failure | avg min | p50 min | max min |`);
  console.log('|---|---|---|---|---|---|---|');
  for (const week of [...byWeek.keys()].sort()) {
    const bucket = byWeek.get(week);
    const durations = bucket.runs.map(run => Date.parse(run.updatedAt) - Date.parse(run.createdAt)).filter(value => value > 0);
    const avg = durations.length > 0 ? durations.reduce((sum, value) => sum + value, 0) / durations.length : 0;
    console.log(
      `| ${week} | ${bucket.runs.length} | ${bucket.success} | ${bucket.failure} | ${minutes(avg)} | ${minutes(percentile(durations, 0.5))} | ${minutes(Math.max(0, ...durations))} |`,
    );
  }

  const slowest = [...runs]
    .map(run => ({
      title: run.displayTitle,
      duration: Date.parse(run.updatedAt) - Date.parse(run.createdAt),
      conclusion: run.conclusion,
      id: run.databaseId,
    }))
    .filter(entry => entry.duration > 0)
    .sort((left, right) => right.duration - left.duration)
    .slice(0, 5);

  console.log('\nSlowest runs:');
  for (const run of slowest) {
    console.log(`- ${minutes(run.duration)} min (${run.conclusion}) ${run.title} — run ${run.id}`);
  }
}

main();
