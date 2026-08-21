import { invokeWithFriendlyError } from './helpers'

export interface EnvironmentDiagnostics {
  ollamaBaseUrl: string
  ollamaReachable: boolean
  ollamaDetail: string
  rolesDir: string
  rolesDirExists: boolean
  rolesDirReadable: boolean
  appDataDir: string
  appDataWritable: boolean
  appDataDetail: string
}

export type RepairSeverity = 'info' | 'warning' | 'error'
export type RepairActionStatus = 'repaired' | 'unchanged' | 'failed'
export type RepairIssueScope
  = | 'installation'
    | 'resources'
    | 'storage'
    | 'roles'
    | 'plugins'
    | 'kernel'
    | 'model_service'
    | 'voice'
    | 'network'
    | 'reporting'
    | 'unknown'
export type RepairIssueCategory
  = | 'missing'
    | 'access'
    | 'invalid'
    | 'conflict'
    | 'compatibility'
    | 'unreachable'
    | 'cleanup'
    | 'reporting'
    | 'unknown'

export interface InstallationRepairIssue {
  code: string
  scope: RepairIssueScope
  category: RepairIssueCategory
  severity: RepairSeverity
  summary: string
  detail: string
  path: string
  repairable: boolean
}

export interface InstallationRepairAction {
  code: string
  status: RepairActionStatus
  summary: string
  detail: string
}

export interface InstallationRepairReport {
  appVersion: string
  operatingSystem: string
  architecture: string
  generatedAtEpochMs: number
  success: boolean
  changed: boolean
  restartRequired: boolean
  resourceDir: string
  rolesDir: string
  appDataDir: string
  roleCount: number
  pluginCount: number
  pluginIds: string[]
  actions: InstallationRepairAction[]
  issues: InstallationRepairIssue[]
  reportPath: string
}

export async function runEnvironmentDiagnostics(): Promise<EnvironmentDiagnostics> {
  return invokeWithFriendlyError<EnvironmentDiagnostics>('run_environment_diagnostics')
}

export async function runEnvironmentRepair(): Promise<InstallationRepairReport> {
  return invokeWithFriendlyError<InstallationRepairReport>('run_environment_repair')
}

export interface RemoteFallbackAppSettings {
  remoteFallbackToBuiltin: string
  remoteFallbackEnvOverrideActive: boolean
}

export async function getRemoteFallbackAppSettings(): Promise<RemoteFallbackAppSettings> {
  return invokeWithFriendlyError<RemoteFallbackAppSettings>('get_remote_fallback_app_settings')
}

export async function setRemoteFallbackToBuiltin(allow: boolean): Promise<void> {
  return invokeWithFriendlyError<void>('set_remote_fallback_to_builtin', { allow })
}
