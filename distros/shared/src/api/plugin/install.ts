import { invokeWithFriendlyError } from '../helpers'

/** `check_plugin_updates` per-plugin online update probe result. */
export interface PluginUpdateInfo {
  hasUpdate: boolean
  latestVersion?: string | null
  message?: string | null
}

export async function checkPluginUpdates(
  pluginIds: string[],
): Promise<Record<string, PluginUpdateInfo>> {
  return invokeWithFriendlyError<Record<string, PluginUpdateInfo>>(
    'check_plugin_updates',
    { pluginIds },
  )
}

export async function extractPluginZip(
  zipPath: string,
  pluginId: string,
): Promise<void> {
  return invokeWithFriendlyError<void>('extract_plugin_zip', {
    zipPath,
    pluginId,
  })
}

/** Install from zip; returns `manifest.id` and on-disk path. */
export async function installPluginFromZip(
  zipPath: string,
): Promise<InstallPluginFromMarketResponseDto> {
  return invokeWithFriendlyError<InstallPluginFromMarketResponseDto>(
    'install_plugin_from_zip',
    { zipPath },
  )
}

/** One plugin row from index (`plugin_installer::PluginIndexEntry`, camelCase DTO). */
export interface PluginIndexEntryDto {
  id: string
  name: string
  description: string
  author: string
  version: string
  git: string
  permissions: string[]
  tags: string[]
  category?: string | null
  source?: string | null
  changelog?: string | null
  dependencies: Record<string, string>
}

export interface PluginMarketEntryDto extends PluginIndexEntryDto {
  installed: boolean
  installedVersion?: string | null
  hasUpdate: boolean
  missingDependencies: string[]
}

export interface PluginMarketSnapshotDto {
  plugins: PluginMarketEntryDto[]
  offlineMode: boolean
  source: string
  warning?: string | null
}

export interface PendingProtocolInstallDto {
  gitUrl: string
}

export interface InstallPluginFromMarketResponseDto {
  installedPluginId: string
  installPath: string
}

export async function syncPluginIndexCommand(
  indexUrl?: string | null,
): Promise<PluginMarketSnapshotDto> {
  return invokeWithFriendlyError<PluginMarketSnapshotDto>(
    'sync_plugin_index_command',
    { indexUrl: indexUrl ?? null },
  )
}

export async function getCachedPluginIndex(): Promise<PluginMarketSnapshotDto> {
  return invokeWithFriendlyError<PluginMarketSnapshotDto>(
    'get_cached_plugin_index',
    {},
  )
}

export async function installPluginFromMarket(
  pluginId: string,
  gitUrl?: string | null,
): Promise<InstallPluginFromMarketResponseDto> {
  return invokeWithFriendlyError<InstallPluginFromMarketResponseDto>(
    'install_plugin_from_market',
    { pluginId, gitUrl: gitUrl ?? null },
  )
}

export async function installPluginFromGit(
  gitUrl: string,
): Promise<InstallPluginFromMarketResponseDto> {
  return invokeWithFriendlyError<InstallPluginFromMarketResponseDto>(
    'install_plugin_from_git',
    { req: { gitUrl } },
  )
}

export async function updatePluginFromMarket(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>('update_plugin_from_market', {
    pluginId,
  })
}

export async function uninstallPluginFromMarket(pluginId: string): Promise<void> {
  return invokeWithFriendlyError<void>('uninstall_plugin_from_market', {
    pluginId,
  })
}

export async function batchUpdatePlugins(pluginIds: string[]): Promise<void> {
  return invokeWithFriendlyError<void>('batch_update_plugins', { pluginIds })
}

export async function batchUninstallPlugins(pluginIds: string[]): Promise<void> {
  return invokeWithFriendlyError<void>('batch_uninstall_plugins', { pluginIds })
}

export async function consumePendingProtocolInstalls(): Promise<
  PendingProtocolInstallDto[]
> {
  return invokeWithFriendlyError<PendingProtocolInstallDto[]>(
    'consume_pending_protocol_installs',
    {},
  )
}

export interface CreatePluginScaffoldRequest {
  pluginId: string
  pluginName: string
  language: 'node' | 'python' | 'rust'
  pluginType: 'skill' | 'agent' | 'module_ext'
  baseDir?: string
}

export interface CreatePluginScaffoldResponse {
  plugin_dir: string
}

export async function createPluginScaffold(
  req: CreatePluginScaffoldRequest,
): Promise<CreatePluginScaffoldResponse> {
  return invokeWithFriendlyError<CreatePluginScaffoldResponse>(
    'create_plugin_scaffold',
    {
      req: {
        pluginId: req.pluginId,
        pluginName: req.pluginName,
        language: req.language,
        pluginType: req.pluginType,
        baseDir: req.baseDir ?? null,
      },
    },
  )
}

export interface PackPluginResponse {
  archive_path: string
  signature_path: string
  sha256: string
}

export async function packPlugin(
  pluginId: string,
  outputDir?: string | null,
): Promise<PackPluginResponse> {
  return invokeWithFriendlyError<PackPluginResponse>('pack_plugin', {
    req: {
      pluginId,
      outputDir: outputDir ?? null,
    },
  })
}
