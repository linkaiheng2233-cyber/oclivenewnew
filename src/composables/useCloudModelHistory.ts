const STORAGE_KEY = 'oclive-cloud-model-history-v1'
const MAX_ITEMS = 24

function readRaw(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw)
      return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed))
      return []
    return parsed.filter((v): v is string => typeof v === 'string' && v.trim().length > 0)
  }
  catch {
    return []
  }
}

function writeRaw(items: string[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
  }
  catch {
    /* ignore */
  }
}

/** Recently used cloud model ids on this device (newest first). */
export function getCloudModelHistory(): string[] {
  return readRaw()
}

/** Remember a cloud model id after a successful save. */
export function rememberCloudModel(model: string): void {
  const trimmed = model.trim()
  if (!trimmed)
    return
  const next = [trimmed, ...readRaw().filter(m => m !== trimmed)].slice(0, MAX_ITEMS)
  writeRaw(next)
}

/** Merge provider list, local history, and current value into one deduped list. */
export function mergeCloudModelOptions(
  providerModels: string[],
  history: string[],
  current?: string,
): string[] {
  const out: string[] = []
  const seen = new Set<string>()
  const push = (value: string) => {
    const v = value.trim()
    if (!v)
      return
    const key = v.toLowerCase()
    if (seen.has(key))
      return
    seen.add(key)
    out.push(v)
  }
  if (current)
    push(current)
  for (const m of history)
    push(m)
  for (const m of providerModels)
    push(m)
  return out
}
