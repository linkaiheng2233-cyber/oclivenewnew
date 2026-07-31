/**
 * Auto-generated SSOT for kernel static error codes.
 * Source: `oclive_kernel_types::kernel_error_codes::all_documented_kernel_codes`
 * Regenerate: `node scripts/generate-kernel-error-codes.mjs`
 * Gate: `node scripts/check-error-codes-drift.mjs`
 */
export const KERNEL_STATIC_ERROR_CODES = [
  'DB_ERROR',
  'DB_MIGRATION_FAILED',
  'EMPTY_MESSAGE',
  'HIGH_RISK_CAPABILITY_NOT_GRANTED',
  'INVALID_PARAMETER',
  'INVALID_ROLE_PATH',
  'IO_ERROR',
  'KERNEL_OFFLINE',
  'LLM_ERROR',
  'LOAD_ROLE_TASK_PANIC',
  'PLUGIN_MANIFEST_INVALID',
  'REMOTE_SERVICE_UNAVAILABLE',
  'ROLE_NOT_FOUND',
  'ROLE_PACK_EXISTS',
  'ROLE_RUNTIME_NOT_READY',
  'SERDE_ERROR',
  'STARTUP_HEALTH_FAILED',
  'THEATER_SCENE_GEN_FAILED',
  'UNKNOWN_ERROR',
] as const

export type KernelStaticErrorCode = (typeof KERNEL_STATIC_ERROR_CODES)[number]

/** Structured `KernelErrorBody.context.kind` values consumed by host i18n. */
export const KERNEL_ERROR_CONTEXT_KINDS = {
  PLUGIN_BACKENDS_DIRECTORY_SLOT: 'plugin_backends_directory_slot',
  HOST_JSON: 'host_json',
  VOICE_RPC_TIMEOUT: 'voice_rpc_timeout',
} as const

export type KernelErrorContextKind
  = (typeof KERNEL_ERROR_CONTEXT_KINDS)[keyof typeof KERNEL_ERROR_CONTEXT_KINDS]

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
