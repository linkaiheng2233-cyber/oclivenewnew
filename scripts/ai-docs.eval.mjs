#!/usr/bin/env node
/**
 * Repository entry point for the vendored evaluation harness.
 *
 * Implementation: scripts/vendor/stackmap/eval.mjs (upstream: stackmap-check).
 * Cases stay in this repository: handoff/catalog/eval.cases.json.
 */
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
if (!process.argv.includes('--root')) process.argv.push('--root', root);
await import('./vendor/stackmap/eval.mjs');
