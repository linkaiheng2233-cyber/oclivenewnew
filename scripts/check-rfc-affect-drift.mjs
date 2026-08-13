#!/usr/bin/env node
/**
 * RFC affect drift ratchet: once display_metrics lands, legacy scalar fields should migrate off hot paths.
 * Emits RFC-pending while deprecated favor/personality/relation scalars remain primary in frontend DTO consumers.
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

const checks = [
  {
    label: 'DTO display_metrics struct',
    file: path.join(repoRoot, 'kernel/crates/oclive_kernel_types/src/models/dto/chat.rs'),
    pattern: 'pub struct DisplayMetricsDto',
    expect: true,
  },
  {
    label: 'role_snapshot fills display_metrics',
    file: path.join(repoRoot, 'kernel/crates/oclive_kernel_host/src/domain/role_snapshot.rs'),
    pattern: 'build_display_metrics',
    expect: true,
  },
  {
    label: 'frontend roleStore prefers display_metrics',
    file: path.join(repoRoot, 'distros/shared/src/stores/roleStore.ts'),
    pattern: 'display_metrics',
    expect: true,
  },
];

let pending = 0;
for (const c of checks) {
  const text = fs.readFileSync(c.file, 'utf8');
  const hit = text.includes(c.pattern);
  if (hit !== c.expect) {
    console.error(`RFC-pending: ${c.label} (${path.relative(repoRoot, c.file)})`);
    pending += 1;
  }
}

if (pending > 0) {
  console.error(`RFC affect drift: ${pending} pending item(s)`);
  process.exit(1);
}

console.log('RFC affect drift ratchet ok');
