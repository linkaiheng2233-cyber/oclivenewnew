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


export async function runEnvironmentDiagnostics(): Promise<EnvironmentDiagnostics> {
  return invokeWithFriendlyError<EnvironmentDiagnostics>('run_environment_diagnostics')
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

