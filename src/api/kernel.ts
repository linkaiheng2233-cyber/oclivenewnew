import { invokeWithFriendlyError } from './helpers'

export type DesktopKernelMode = 'attached' | 'spawned' | 'offline' | 'reconnecting'

export interface KernelConnectionStatus {
  mode: DesktopKernelMode
  baseUrl: string
  port: number
  binaryPath: string | null
  kernelTier: string | null
  healthy: boolean
  degraded?: boolean | null
  statusMessage?: string | null
  profileHintKey?: string | null
}

export interface KernelDiagnostics {
  status: KernelConnectionStatus
  sharedRuntimePath: string
  sharedRuntimeExists: boolean
  sharedRuntimeModifiedMs: number | null
  healthJson: Record<string, unknown> | null
}

export interface RoleSnapshot {
  role_id: string
  current_favorability: number
  current_emotion: string
  portrait_emotion: string
  relation_state: string
  personality_source: string
  current_scene: string | null
  user_presence_scene: string | null
}

export async function getKernelConnectionStatus(): Promise<KernelConnectionStatus> {
  return invokeWithFriendlyError<KernelConnectionStatus>('get_kernel_connection_status')
}

export async function reconnectKernel(): Promise<KernelConnectionStatus> {
  return invokeWithFriendlyError<KernelConnectionStatus>('reconnect_kernel')
}

export async function getKernelDiagnostics(): Promise<KernelDiagnostics> {
  return invokeWithFriendlyError<KernelDiagnostics>('get_kernel_diagnostics')
}

/** Poll role snapshot via kernel HTTP (through Tauri when available). */
export async function fetchRoleSnapshot(
  roleId: string,
  sceneId?: string | null,
): Promise<RoleSnapshot | null> {
  try {
    const status = await getKernelConnectionStatus()
    if (!status.healthy) {
      return null
    }
    const params = new URLSearchParams({ role_id: roleId })
    if (sceneId) {
      params.set('scene_id', sceneId)
    }
    const res = await fetch(`${status.baseUrl}/role_snapshot?${params.toString()}`, {
      signal: AbortSignal.timeout(5000),
    })
    if (!res.ok) {
      return null
    }
    return (await res.json()) as RoleSnapshot
  }
  catch {
    return null
  }
}
