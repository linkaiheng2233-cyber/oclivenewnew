import { invokeWithFriendlyError } from '../helpers'

/** Spawn directory-plugin RPC child for quick test (`PluginProcessDebugInfo` DTO). */
export interface PluginProcessDebugInfo {
  pluginId: string
  pid: number
  rpcUrl: string
  startedAtMs: number
  cpuPercent?: number | null
  memoryKb?: number | null
}

/** Flat Tauri command args use camelCase IPC keys; Rust handlers use `snake_case` params. */
export async function spawnPluginForTest(
  pluginId: string,
  configJson?: string | null,
): Promise<PluginProcessDebugInfo> {
  return invokeWithFriendlyError<PluginProcessDebugInfo>('spawn_plugin_for_test', {
    pluginId,
    configJson: configJson ?? null,
  })
}

export async function killPluginProcess(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>('kill_plugin_process', { pluginId })
}

export async function listPluginProcesses(): Promise<PluginProcessDebugInfo[]> {
  return invokeWithFriendlyError<PluginProcessDebugInfo[]>('list_plugin_processes', {})
}

export async function getPluginLogs(
  pluginId: string,
  lines: number,
): Promise<string[]> {
  return invokeWithFriendlyError<string[]>('get_plugin_logs', {
    pluginId,
    lines,
  })
}

export async function clearPluginLogs(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>('clear_plugin_logs', { pluginId })
}

export async function testPluginMethod(
  pluginId: string,
  method: string,
  params: unknown = {},
): Promise<unknown> {
  return invokeWithFriendlyError<unknown>('test_plugin_method', {
    req: {
      pluginId,
      method,
      params,
    },
  })
}

export async function discoverPluginMethods(pluginId: string): Promise<string[]> {
  return invokeWithFriendlyError<string[]>('discover_plugin_methods', {
    pluginId,
  })
}
