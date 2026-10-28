import type { RoleInfo } from './role'
import { recordDiagnostic } from '@oclive/shared/composables/useDiagnosticHistory'
import { invoke } from '@tauri-apps/api/core'
import { invokeWithFriendlyError, toErrorPresentation } from './helpers'

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
  display_metrics?: {
    favor: number
    relation_summary: string
    traits: number[]
  } | null
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

let snapshotFailure = ''

/** Background projection of the existing authenticated native role-info transport. */
export async function fetchRoleSnapshot(
  roleId: string,
  _sceneId?: string | null,
): Promise<RoleSnapshot | null> {
  try {
    // Unlike foreground actions, a failed poll records one diagnostic instead of a toast
    // every eight seconds. Rust owns the HTTP token; it never enters the renderer.
    const info = await invoke<RoleInfo>('get_role_info', {
      req: { role_id: roleId, session_id: null },
    })
    snapshotFailure = ''
    return {
      role_id: info.role_id,
      current_favorability: info.current_favorability,
      current_emotion: info.current_emotion,
      portrait_emotion: info.current_emotion,
      relation_state: info.relation_state,
      display_metrics: info.display_metrics,
      personality_source: info.personality_source ?? 'vector',
      current_scene: info.current_scene,
      user_presence_scene: info.user_presence_scene,
    }
  }
  catch (err) {
    const display = toErrorPresentation(err)
    const key = `${roleId}:${display.summary}`
    if (snapshotFailure !== key) {
      snapshotFailure = key
      recordDiagnostic({ level: 'warning', source: 'role-snapshot', ...display })
    }
    return null
  }
}
