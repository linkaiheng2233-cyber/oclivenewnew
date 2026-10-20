#!/usr/bin/env node
/** Verify scripts/vendor/stackmap matches its manifest; exit 1 on any mismatch. */
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const vendorDir = path.join(fileURLToPath(new URL('../', import.meta.url)), 'scripts/vendor/stackmap');
let manifest;
try {
  manifest = JSON.parse(fs.readFileSync(path.join(vendorDir, 'VENDOR.json'), 'utf8'));
} catch (error) {
  console.error(`check-vendor: unreadable manifest (${error.message})`);
  process.exitCode = 2;
  manifest = null;
}

if (manifest) {
  let ok = true;
  for (const [name, expected] of Object.entries(manifest.files ?? {})) {
    const file = path.join(vendorDir, name);
    if (!fs.existsSync(file)) {
      console.error(`check-vendor: missing ${name}`);
      ok = false;
      continue;
    }
    const actual = createHash('sha256').update(fs.readFileSync(file)).digest('hex');
    if (actual !== expected) {
      console.error(`check-vendor: mismatch ${name}`);
      ok = false;
    }
  }
  if (ok) console.log(`check-vendor: ok (${Object.keys(manifest.files ?? {}).length} files @ ${String(manifest.commit).slice(0, 8)})`);
  process.exitCode = ok ? 0 : 1;
}
