import type { Ref } from 'vue'
import { ref } from 'vue'

function unknownToDetail(e: unknown): string | null {
  if (e instanceof Error) {
    return e.stack || e.message
  }
  return typeof e === 'string' ? e : JSON.stringify(e)
}

function unknownToMessage(e: unknown, fallback: string): string {
  if (e instanceof Error) {
    return e.message || fallback
  }
  if (typeof e === 'string' && e.trim()) {
    return e
  }
  return fallback
}

/** Single-region (e.g. full shell) load error + detail. */
export function useSinglePluginError() {
  const message = ref<string | null>(null)
  const detail = ref<string | null>(null)

  function clearError(): void {
    message.value = null
    detail.value = null
  }

  function setError(msg: string, d?: string | null): void {
    message.value = msg
    detail.value = d ?? null
  }

  function setFromUnknown(e: unknown, fallback: string): void {
    setError(unknownToMessage(e, fallback), unknownToDetail(e))
  }

  async function wrapAsync<T>(
    op: () => Promise<T>,
    opts: { onError?: (e: unknown) => void, fallbackMessage: string },
  ): Promise<T | undefined> {
    try {
      return await op()
    }
    catch (e) {
      opts.onError?.(e)
      setFromUnknown(e, opts.fallbackMessage)
      return undefined
    }
  }

  return {
    message,
    detail,
    clearError,
    setError,
    setFromUnknown,
    wrapAsync,
  }
}

/** Per plugin id → message / detail (shared by slot iframe + Vue). */
export function useKeyedPluginErrors() {
  const messages = ref<Record<string, string>>({})
  const details = ref<Record<string, string>>({})

  function clearAll(): void {
    messages.value = {}
    details.value = {}
  }

  function clearKey(pluginId: string): void {
    const m = { ...messages.value }
    delete m[pluginId]
    messages.value = m
    const d = { ...details.value }
    delete d[pluginId]
    details.value = d
  }

  function setKey(pluginId: string, msg: string, detail?: string | null): void {
    messages.value = { ...messages.value, [pluginId]: msg }
    if (detail) {
      details.value = { ...details.value, [pluginId]: detail }
    }
    else {
      const next = { ...details.value }
      delete next[pluginId]
      details.value = next
    }
  }

  function setFromUnknownKey(
    pluginId: string,
    e: unknown,
    fallback: string,
  ): void {
    setKey(pluginId, unknownToMessage(e, fallback), unknownToDetail(e))
  }

  return {
    messages,
    details,
    clearAll,
    clearKey,
    setKey,
    setFromUnknownKey,
  }
}

/** Async wrapper: on failure write to `messages` / `details` (by pluginId). */
export async function wrapKeyedAsync<T>(
  pluginId: string,
  messages: Ref<Record<string, string>>,
  details: Ref<Record<string, string>>,
  op: () => Promise<T>,
  fallbackMessage: string,
): Promise<T | undefined> {
  try {
    return await op()
  }
  catch (e) {
    const msg = unknownToMessage(e, fallbackMessage)
    const det = unknownToDetail(e)
    messages.value = { ...messages.value, [pluginId]: msg }
    if (det) {
      details.value = { ...details.value, [pluginId]: det }
    }
    return undefined
  }
}
