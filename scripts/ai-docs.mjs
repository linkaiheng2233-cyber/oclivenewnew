#!/usr/bin/env node
/**
 * Repository entry point for the vendored documentation navigator.
 *
 * Implementation: scripts/vendor/stackmap/cli.mjs (upstream: stackmap-check).
 * Do not edit the vendored copy here; change it upstream and run `npm run vendor:sync`.
 */
import { fileURLToPath } from 'node:url';
import { main } from './vendor/stackmap/cli.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const args = process.argv.slice(2);
if (!args.includes('--root')) args.push('--root', root);
process.exitCode = main(args);
