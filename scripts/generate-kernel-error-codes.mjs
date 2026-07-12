#!/usr/bin/env node
/**
 * Regenerate distros/shared/src/api/generated/kernelErrorCodes.ts from Rust SSOT.
 *
 * Usage: node scripts/generate-kernel-error-codes.mjs
 */
import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

const out = execFileSync(
  'cargo',
  ['test', '-p', 'oclive_kernel_types', 'export_kernel_error_codes_json', '--', '--nocapture'],
  { cwd: repoRoot, encoding: 'utf8' },
);

const marker = 'KERNEL_ERROR_CODES_JSON:';
const line = out.split('\n').find(l => l.includes(marker));
if (!line) {
  throw new Error('export_kernel_error_codes_json did not emit KERNEL_ERROR_CODES_JSON line');
}
const codes = JSON.parse(line.slice(line.indexOf(marker) + marker.length));

const target = path.join(repoRoot, 'distros/shared/src/api/generated/kernelErrorCodes.ts');
const kinds = [
  'PLUGIN_BACKENDS_DIRECTORY_SLOT: \'plugin_backends_directory_slot\'',
  'HOST_JSON: \'host_json\'',
  'VOICE_RPC_TIMEOUT: \'voice_rpc_timeout\'',
];

const body = `/**
 * Auto-generated SSOT for kernel static error codes.
 * Source: \`oclive_kernel_types::kernel_error_codes::all_documented_kernel_codes\`
 * Regenerate: \`node scripts/generate-kernel-error-codes.mjs\`
 * Gate: \`node scripts/check-error-codes-drift.mjs\`
 */
export const KERNEL_STATIC_ERROR_CODES = ${JSON.stringify(codes, null, 2)} as const

export type KernelStaticErrorCode = (typeof KERNEL_STATIC_ERROR_CODES)[number]

/** Structured \`KernelErrorBody.context.kind\` values consumed by host i18n. */
export const KERNEL_ERROR_CONTEXT_KINDS = {
  ${kinds.join(',\n  ')},
} as const

export type KernelErrorContextKind =
  (typeof KERNEL_ERROR_CONTEXT_KINDS)[keyof typeof KERNEL_ERROR_CONTEXT_KINDS]

export function isKernelStaticErrorCode(code: string): code is KernelStaticErrorCode {
  return (KERNEL_STATIC_ERROR_CODES as readonly string[]).includes(code)
}

export function kernelErrorContextKind(
  context: unknown,
): KernelErrorContextKind | undefined {
  if (!context || typeof context !== 'object')
    return undefined
  const kind = (context as { kind?: unknown }).kind
  return typeof kind === 'string' ? kind as KernelErrorContextKind : undefined
}
`;

fs.writeFileSync(target, body);
console.log(`Wrote ${path.relative(repoRoot, target)} (${codes.length} codes)`);
