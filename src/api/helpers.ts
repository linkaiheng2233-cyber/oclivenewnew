import { invoke } from '@tauri-apps/api/tauri'

import { i18n } from '../i18n/index'

function translateApiError(code: string): string | undefined {
  const key = `apiErrors.${code}`
  if (i18n.global.te(key))
    return String(i18n.global.t(key))
  return undefined
}

/** �?`oclive_kernel_runtime::KernelErrorBody` / HTTP `error` 对象同形（内核权�?JSON）�?*/
export interface KernelErrorPayload {
  code: string
  message: string
  hint?: string | null
}

function parseBackendError(err: unknown): {
  code?: string
  raw: string
  kernel?: KernelErrorPayload
} {
  const raw = String(err ?? '')
  const trimmed = raw.trim()
  if (trimmed.startsWith('{')) {
    try {
      const j = JSON.parse(trimmed) as Partial<KernelErrorPayload>
      if (j && typeof j.code === 'string' && typeof j.message === 'string') {
        return { code: j.code, raw, kernel: j as KernelErrorPayload }
      }
    }
    catch {
      /* fallthrough: legacy `[CODE]` or plain text */
    }
  }
  const match = raw.match(/\[([A-Z0-9_]+)\]/)
  return { code: match?.[1], raw }
}

export interface FriendlyError {
  code?: string
  message: string
  raw: string
  /** �?`invoke` 失败载荷为内�?JSON 时填充，便于 UI/遥测�?HTTP 对齐�?*/
  kernel?: KernelErrorPayload
}

type ErrorReporter = (err: FriendlyError) => void

let errorReporter: ErrorReporter | null = null

export function setErrorReporter(reporter: ErrorReporter | null): void {
  errorReporter = reporter
}

/** �?`invoke` 抛出的字符串中解析机器码：优先内�?JSON `code`，否�?legacy `[CODE]`�?*/
export function parseApiErrorCode(err: unknown): string | undefined {
  return parseBackendError(err).code
}

/** 是否为目录插件未找到类错误（便于 UI 分支）�?*/
export function isPluginNotFoundError(err: unknown): boolean {
  return parseApiErrorCode(err) === 'API_PLUGIN_NOT_FOUND'
}

export function isPermissionDeniedError(err: unknown): boolean {
  return parseApiErrorCode(err) === 'API_PERMISSION_DENIED'
}

export function isInvalidParameterError(err: unknown): boolean {
  return parseApiErrorCode(err) === 'INVALID_PARAMETER'
}

export function toFriendlyErrorMessage(err: unknown): string {
  const { code, raw, kernel } = parseBackendError(err)
  if (!code)
    return raw
  const text = kernel?.message ?? raw
  if (code === 'STARTUP_HEALTH_FAILED') {
    let detail = (kernel?.message ?? '').replace(/^Startup health failed:\s*/i, '').trim()
    if (!detail) {
      const bracket = raw.indexOf(']')
      detail
        = bracket !== -1
          ? raw.slice(bracket + 1).trim()
          : raw.trim()
      detail = detail.replace(/^Startup health failed:\s*/i, '').trim()
    }
    if (i18n.global.te('apiErrors.STARTUP_HEALTH_FAILED')) {
      return String(i18n.global.t('apiErrors.STARTUP_HEALTH_FAILED', { detail }))
    }
  }
  if (code === 'INVALID_PARAMETER') {
    if (text.includes('plugin_backends:')) {
      const mapped = translateApiError('PLUGIN_BACKENDS_DIRECTORY_SLOT')
      if (mapped)
        return mapped
    }
    let detail = ''
    if (kernel?.message) {
      const m = kernel.message.match(/^Invalid parameter:\s*(.*)/i)
      if (m)
        detail = m[1]?.trim() ?? ''
    }
    if (!detail) {
      const bracket = raw.indexOf(']')
      if (bracket !== -1) {
        detail = raw.slice(bracket + 1).trim()
        detail = detail.replace(/^Invalid parameter:\s*/i, '').trim()
      }
    }
    if (detail && i18n.global.te('apiErrors.INVALID_PARAMETER_DETAIL')) {
      return String(i18n.global.t('apiErrors.INVALID_PARAMETER_DETAIL', { detail }))
    }
  }
  if (code === 'ROLE_NOT_FOUND') {
    if (kernel?.message) {
      const km = kernel.message.trim()
      if (km.startsWith('Role not found:')) {
        const suffix = km.slice('Role not found:'.length).trim()
        if (i18n.global.te('apiErrors.ROLE_NOT_FOUND_DETAIL')) {
          return String(i18n.global.t('apiErrors.ROLE_NOT_FOUND_DETAIL', { detail: suffix }))
        }
      }
    }
    const bracket = raw.indexOf(']')
    if (bracket !== -1) {
      const detail = raw.slice(bracket + 1).trim()
      if (detail.startsWith('Role not found:')) {
        const suffix = detail.slice('Role not found:'.length).trim()
        if (i18n.global.te('apiErrors.ROLE_NOT_FOUND_DETAIL')) {
          return String(
            i18n.global.t('apiErrors.ROLE_NOT_FOUND_DETAIL', { detail: suffix }),
          )
        }
      }
    }
  }
  if (code === 'IO_ERROR' && (text.includes('host json') || raw.includes('host json'))) {
    const mapped = translateApiError('IO_ERROR_HOST_JSON')
    if (mapped)
      return mapped
  }
  const mapped = translateApiError(code)
  if (mapped)
    return mapped
  if (i18n.global.te('apiErrors.UNKNOWN_WITH_CODE')) {
    return String(i18n.global.t('apiErrors.UNKNOWN_WITH_CODE', { code }))
  }
  return raw
}

export function toFriendlyError(err: unknown): FriendlyError {
  const { code, raw, kernel } = parseBackendError(err)
  return {
    code,
    raw,
    kernel,
    message: toFriendlyErrorMessage(err),
  }
}

export async function invokeWithFriendlyError<T>(
  command: string,
  payload: Record<string, unknown> = {},
): Promise<T> {
  try {
    return await invoke<T>(command, payload)
  }
  catch (err) {
    const friendly = toFriendlyError(err)
    // 友好文案会盖住后端细节；开发排查时看控制台完整 raw
    console.error(`[tauri:${command}]`, friendly.code ?? '?', friendly.raw)
    if (errorReporter) {
      errorReporter(friendly)
    }
    else if (friendly.code) {
      console.warn(`[api-error] code=${friendly.code} msg=${friendly.message}`)
    }
    throw new Error(friendly.message)
  }
}
/** snake_case �?camelCase for a single key (Tauri IPC top-level args). */
export function snakeToCamelKey(key: string): string {
  return key.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase())
}

/** Shallow-recursive camelCase key transform for invoke payloads. */
export function toCamelPayload(value: unknown): unknown {
  if (value === null || value === undefined)
    return value
  if (Array.isArray(value))
    return value.map(v => toCamelPayload(v))
  if (typeof value !== 'object')
    return value
  const out: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
    out[snakeToCamelKey(k)] = toCamelPayload(v)
  }
  return out
}
