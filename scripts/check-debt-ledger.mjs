#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateDebtLedger } from './lib/debt-ledger.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
try {
  const args = process.argv.slice(2);
  if (args.length && (args.length !== 2 || args[0] !== '--file' || !args[1])) {
    throw new Error('usage: check-debt-ledger.mjs [--file <inventory.md>]');
  }
  const file = args[1] ?? path.join(root, 'handoff/TECHNICAL_DEBT_INVENTORY.md');
  const result = validateDebtLedger(fs.readFileSync(file, 'utf8'));
  console.log(`debt-ledger structure: PASS ${JSON.stringify(result)} (not a debt-status assessment)`);
} catch (error) {
  console.error(`debt-ledger structure: FAIL\n${error.message}`);
  process.exitCode = 1;
}
