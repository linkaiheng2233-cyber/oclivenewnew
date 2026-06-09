import { getKernelConnectionStatus } from '../api/kernel'

interface HealthJson {
  startupWarnings?: string[]
  startup_warnings?: string[]
}

/** Non-fatal kernel startup warnings from `GET /health` (V-SLOT-HONEST-01). */
export async function fetchKernelStartupWarnings(): Promise<string[]> {
  try {
    const status = await getKernelConnectionStatus()
    if (!status.healthy) {
      return []
    }
    const res = await fetch(`${status.baseUrl}/health`, {
      headers: { Accept: 'application/json' },
      signal: AbortSignal.timeout(4000),
    })
    if (!res.ok) {
      return []
    }
    const json = await res.json() as HealthJson
    const raw = json.startupWarnings ?? json.startup_warnings ?? []
    return raw.filter(w => typeof w === 'string' && w.trim().length > 0)
  }
  catch {
    return []
  }
}
