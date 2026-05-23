import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'

export const OCLIVE_DEFAULT_RELATION_SENTINEL = '__oclive_default__'


export function emptyPackUiConfig(): PackUiConfig {
  return {
    shell: '',
    theme: { primaryColor: '', backgroundColor: '', fontFamily: '' },
    layout: { sidebar: '', chatInput: '' },
    slots: {
      'chat_toolbar': { order: [], visible: [] },
      'settings.panel': { order: [], visible: [] },
      'role.detail': { order: [], visible: [] },
      'sidebar': { order: [], visible: [] },
      'chat.header': { order: [], visible: [] },
    },
  }
}

/** 与后�?`models::author_pack::AuthorPackFile` 对齐（snake_case 字段）�?*/

export function normalizePackUiConfig(
  raw: PackUiConfig | undefined | null,
): PackUiConfig {
  const e = emptyPackUiConfig()
  if (!raw)
    return e
  const slots = raw.slots
  const slot = (k: keyof PackUiSlots): PackUiSlotConfig => {
    const s = slots?.[k]
    return {
      order: Array.isArray(s?.order) ? s!.order.map(String) : [],
      visible: Array.isArray(s?.visible) ? s!.visible.map(String) : [],
    }
  }
  return {
    shell: typeof raw.shell === 'string' ? raw.shell : e.shell,
    theme: {
      primaryColor: raw.theme?.primaryColor?.trim() ?? '',
      backgroundColor: raw.theme?.backgroundColor?.trim() ?? '',
      fontFamily: raw.theme?.fontFamily?.trim() ?? '',
    },
    layout: {
      sidebar: (raw.layout?.sidebar ?? '').trim().toLowerCase(),
      chatInput: (raw.layout?.chatInput ?? '').trim().toLowerCase(),
    },
    slots: {
      'chat_toolbar': slot('chat_toolbar'),
      'settings.panel': slot('settings.panel'),
      'role.detail': slot('role.detail'),
      'sidebar': slot('sidebar'),
      'chat.header': slot('chat.header'),
    },
  }
}


export interface PluginUiSlotInfo {
  pluginId: string
  slot: string
  /** manifest `ui_slots[].appearance_id`；空字符串为默认变体 */
  appearanceId?: string
  /** manifest `ui_slots[].label` */
  label?: string | null
  /** manifest `ui_slots[].entry`（相对插件根�?*/
  entry: string
  /** manifest `vueComponent`；存在时优先原生 Vue，失败则回退 `url` iframe */
  vueComponent?: string | null
  url: string
}

/** 读取目录插件根下文本文件（宿主编�?`.vue` 等）�?*/

export async function readPluginAssetText(
  pluginId: string,
  rel: string,
): Promise<string> {
  return invokeWithFriendlyError<string>('read_plugin_asset_text', {
    pluginId,
    rel,
  })
}

/** 目录插件启动引导（整�?URL、已扫描插件 id、开发者模式、UI 插槽）�?*/

export interface DirectoryPluginBootstrap {
  shellUrl?: string | null
  shellPluginId?: string | null
  /** 整壳 `manifest.shell.vueEntry`（相对插件根）；�?`forceIframeMode` 决定是否走宿�?Vue 整壳�?*/
  shellVueEntry?: string | null
  /** �?`plugin_state.force_iframe_mode` 一致；为真时忽�?Vue 整壳与插�?Vue�?*/
  forceIframeMode?: boolean
  pluginIds: string[]
  developerMode: boolean
  /** 当前角色下已启用插件�?manifest `bridge.events` 中声明的宿主事件名�?*/
  subscribedHostEvents: string[]
  uiSlots: PluginUiSlotInfo[]
}

/** `check_plugin_updates` 单插件结果（在线检查预留）�?*/

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
    { plugin_ids: pluginIds },
  )
}


export async function extractPluginZip(
  zipPath: string,
  pluginId: string,
): Promise<void> {
  return invokeWithFriendlyError<void>('extract_plugin_zip', {
    zip_path: zipPath,
    plugin_id: pluginId,
  })
}

/** �?zip 安装目录插件；返�?manifest.id�?*/

export async function installPluginFromZip(zipPath: string): Promise<string> {
  return invokeWithFriendlyError<string>('install_plugin_from_zip', {
    zipPath,
  })
}

/** 同一 `role_id` 上并发的 bootstrap 合并为单�?IPC，避免多插槽同时挂载时重复打后端�?*/
const directoryBootstrapInflight = new Map<
  string,
  Promise<DirectoryPluginBootstrap>
>()

function directoryBootstrapCacheKey(roleId?: string | null): string {
  const t = (roleId ?? '').trim()
  return t.length > 0 ? t : '__default__'
}


export async function getDirectoryPluginBootstrap(
  roleId?: string | null,
): Promise<DirectoryPluginBootstrap> {
  const key = directoryBootstrapCacheKey(roleId)
  const existing = directoryBootstrapInflight.get(key)
  if (existing) {
    return existing
  }
  const p = invokeWithFriendlyError<DirectoryPluginBootstrap>(
    'get_directory_plugin_bootstrap',
    { role_id: roleId ?? null },
  ).finally(() => {
    if (directoryBootstrapInflight.get(key) === p) {
      directoryBootstrapInflight.delete(key)
    }
  })
  directoryBootstrapInflight.set(key, p)
  return p
}

/** �?`app_data/plugin_state.json` 中单角色 slots 段一致（snake_case）�?*/

export interface PluginStateFile {
  disabled_plugins: string[]
  slot_order: Record<string, string[]>
  disabled_slot_contributions: Record<string, string[]>
  /** `plugin_id` �?`slot` �?`appearance_id` */
  slot_appearance?: Record<string, Record<string, string>>
  /** 为真时忽�?`vueComponent`，嵌入插槽仅�?iframe�?*/
  force_iframe_mode?: boolean
}

/** 单角色的目录插件 UI 状态（含整�?id，与后端 `RolePluginStateDto` 一致）�?*/

export interface RolePluginState extends PluginStateFile {
  shellPluginId: string
}


export interface PluginStateGetResponse {
  role: RolePluginState
  /** 后端 `serde(rename_all = "camelCase")` �?`globalDefaults` */
  globalDefaults: RolePluginState
}

/** 并发 `get_plugin_state(role_id)` 合并为单�?IPC（按 role_id 维度）�?*/
const pluginStateInflight = new Map<string, Promise<PluginStateGetResponse>>()

function pluginStateCacheKey(roleId: string): string {
  const t = roleId.trim()
  return t.length > 0 ? t : '__default__'
}

/** 角色包根目录 `ui.json`（与编写�?/ 后端 `UiConfig` 一致）�?*/

export interface SlotConfig {
  order: string[]
  visible: string[]
  /** 插件 id �?默认 `appearance_id`（该槽内�?*/
  appearance?: Record<string, string>
}


export interface UiConfig {
  shell: string
  slots: {
    'chat_toolbar': SlotConfig
    'settings.panel': SlotConfig
    'role.detail': SlotConfig
    'sidebar': SlotConfig
    'chat.header': SlotConfig
    'settings.plugins': SlotConfig
    'settings.advanced': SlotConfig
    'overlay.floating': SlotConfig
    'launcher.palette': SlotConfig
    'debug.dock': SlotConfig
  }
}


export interface UiSlotVariantInfo {
  slot: string
  appearanceId: string
  label?: string | null
}


export interface DirectoryPluginCatalogEntry {
  id: string
  version: string
  pluginType?: string | null
  /** manifest �?`uiTemplate` �?`uiSchema.fields` */
  hasUiSettings?: boolean
  /** manifest 是否�?`process`（可在此面板启动 JSON-RPC 子进程） */
  hasRpcProcess: boolean
  /** manifest 是否声明 `rpcMethods`（调试面板可预填方法名） */
  declaresRpcMethods?: boolean
  isShell: boolean
  uiSlotNames: string[]
  /** 每条 manifest `ui_slots`（嵌入槽）一�?*/
  uiSlotVariants?: UiSlotVariantInfo[]
  provides: string[]
  description?: string | null
  author?: string | null
  permissions?: string[]
  /** `ok` / `missing` / `mismatch` */
  dependencyStatus: string
  dependencyIssues: string[]
}

/** 并发 `get_directory_plugin_catalog` 合并为单�?IPC（无 role 参数，全局共用一�?in-flight）�?*/
const directoryCatalogInflight = new Map<
  string,
  Promise<DirectoryPluginCatalogEntry[]>
>()
const DIRECTORY_CATALOG_COALESCE_KEY = '__global__'


export async function getDirectoryPluginCatalog(): Promise<DirectoryPluginCatalogEntry[]> {
  const existing = directoryCatalogInflight.get(DIRECTORY_CATALOG_COALESCE_KEY)
  if (existing) {
    return existing
  }
  const p = invokeWithFriendlyError<DirectoryPluginCatalogEntry[]>(
    'get_directory_plugin_catalog',
    {},
  ).finally(() => {
    if (directoryCatalogInflight.get(DIRECTORY_CATALOG_COALESCE_KEY) === p) {
      directoryCatalogInflight.delete(DIRECTORY_CATALOG_COALESCE_KEY)
    }
  })
  directoryCatalogInflight.set(DIRECTORY_CATALOG_COALESCE_KEY, p)
  return p
}


export async function getPluginState(
  roleId: string,
): Promise<PluginStateGetResponse> {
  const key = pluginStateCacheKey(roleId)
  const existing = pluginStateInflight.get(key)
  if (existing) {
    return existing
  }
  const p = invokeWithFriendlyError<PluginStateGetResponse>('get_plugin_state', {
    roleId,
  }).finally(() => {
    if (pluginStateInflight.get(key) === p) {
      pluginStateInflight.delete(key)
    }
  })
  pluginStateInflight.set(key, p)
  return p
}


export async function savePluginState(
  roleId: string,
  state: RolePluginState,
): Promise<void> {
  pluginStateInflight.delete(pluginStateCacheKey(roleId))
  return invokeWithFriendlyError<void>('save_plugin_state', {
    roleId,
    state,
  })
}


export async function saveGlobalPluginState(
  state: RolePluginState,
): Promise<void> {
  pluginStateInflight.clear()
  return invokeWithFriendlyError<void>('save_global_plugin_state', {
    state,
  })
}

/** 用磁盘上�?`ui.json` 覆盖该角色的本地插件 UI 状态�?*/

export async function resetPluginStateToRoleDefault(
  roleId: string,
): Promise<void> {
  pluginStateInflight.delete(pluginStateCacheKey(roleId))
  return invokeWithFriendlyError<void>('reset_plugin_state_to_role_default', {
    roleId,
  })
}

/** 网页索引中的单条插件（与 `plugin_installer::PluginIndexEntry` 一致，camelCase）�?*/

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


export interface UiSchemaFieldDto {
  key: string
  label: string
  type: string
  required: boolean
  default?: unknown
}


export interface PluginUiSettingsDto {
  uiTemplate?: string | null
  fields: UiSchemaFieldDto[]
  config: Record<string, unknown>
}


export async function getPluginSettingsUi(
  pluginId: string,
): Promise<PluginUiSettingsDto> {
  return invokeWithFriendlyError<PluginUiSettingsDto>('get_plugin_settings_ui', {
    pluginId,
  })
}


export async function setPluginSettingsConfig(
  pluginId: string,
  config: Record<string, unknown>,
): Promise<void> {
  return invokeWithFriendlyError<void>('set_plugin_settings_config', {
    pluginId,
    config,
  })
}


export async function directoryPluginInvoke(
  pluginId: string,
  method: string,
  params: unknown = {},
): Promise<unknown> {
  return invokeWithFriendlyError<unknown>('directory_plugin_invoke', {
    req: {
      pluginId,
      method,
      params,
    },
  })
}

/** 开发者调试：目录插件 RPC 子进程快照（与后�?`PluginProcessDebugInfo` 一致）�?*/

export interface PluginProcessDebugInfo {
  pluginId: string
  pid: number
  rpcUrl: string
  startedAtMs: number
  cpuPercent?: number | null
  memoryKb?: number | null
}

/** 扁平 Tauri command 参数�?IPC 上为 camelCase（与 Rust �?`snake_case` 形参对应）�?*/

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
        plugin_id: req.pluginId,
        plugin_name: req.pluginName,
        language: req.language,
        plugin_type: req.pluginType,
        base_dir: req.baseDir ?? null,
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
      plugin_id: pluginId,
      output_dir: outputDir ?? null,
    },
  })
}

/**
 * manifest `shell.bridge.invoke` 可声�?**命令�?* �?**权限别名**（后者用�?`get_conversation` �?`read:conversation` 等）�?
 * 敏感命令（聊�?角色切换）还要求 **`type`: `"ocliveplugin"`** 且来源为 **`shell.entry`** HTML �?**`shell.vueEntry`** Vue 整壳�?
 */

export type PluginBridgeManifestToken
  = | 'send_message'
    | 'read:conversation'
    | 'switch_role'
    | 'read:roles'
    | 'read:current_role'
    | 'get_role_info'
    | 'list_roles'
    | 'get_time_state'
    | 'get_directory_plugin_bootstrap'
    | 'get_conversation'
    | 'get_roles'
    | 'get_current_role'
    | 'update_memory'
    | 'delete_memory'
    | 'update_emotion'
    | 'update_event'
    | 'update_prompt'
    | 'write:memory'
    | 'write:emotion'
    | 'write:event'
    | 'write:prompt'
    | 'export_conversation'
    | 'import_role'
    | 'export:conversation'
    | 'import:role'
    | 'delete_role'
    | 'update_settings'
    | 'get_conversation_list'
    | 'delete:role'
    | 'write:settings'
    | 'read:conversations'

/** 整壳 `OclivePluginBridge.invoke('update_memory', params)` */

export interface PluginBridgeUpdateMemoryParams {
  role_id: string
  content: string
  /** 0�?，默�?0.5 */
  importance?: number
}


export interface PluginBridgeDeleteMemoryParams {
  role_id: string
  memory_id: string
}


export interface PluginBridgeUpdateEmotionParams {
  role_id: string
  emotion: string
}


export interface PluginBridgeUpdateEventParams {
  role_id: string
  event_type: string
  description?: string | null
}

/** 预留；宿主未实现动态提示词片段时返�?`not_implemented`�?*/

export interface PluginBridgeUpdatePromptParams {
  role_id: string
  /** 由后续宿主契约定�?*/
  fragment_key?: string
  content?: string
}

/** `plugin_bridge_invoke` �?`send_message`（字段与 {@link SendMessageRequest} 一致；可提�?`text` 代替 `user_message`�?*/

export interface PluginBridgeGetConversationParams {
  role_id: string
  session_id?: string | null
  limit?: number
  offset?: number
}


export interface PluginBridgeConversationTurn {
  user_input: string
  bot_reply: string
  emotion: string
  scene: string | null
  created_at: string
}


export interface PluginBridgeGetConversationResult {
  role_id: string
  session_namespace: string
  total: number
  limit: number
  offset: number
  items: PluginBridgeConversationTurn[]
}

/** `export_conversation` �?�?`export_chat_logs` 单角色导出一致（`format`: `json` | `txt`）�?*/

export interface PluginBridgeExportConversationParams {
  role_id: string
  format?: string
  session_id?: string | null
}


export interface PluginBridgeExportConversationResult {
  content: string
  suggested_filename: string
}

/** `import_role`：从 `.zip` / `.ocpak` 或已解压目录导入角色包�?*/

export interface PluginBridgeImportRoleParams {
  path: string
  /** �?`src_path` 二选一 */
  src_path?: string
  overwrite?: boolean
}


export interface PluginBridgeImportRoleResult {
  role_id: string
  ok: boolean
}

/** `delete_role`：删除本地角色包及相关数据�?*/

export interface PluginBridgeDeleteRoleParams {
  role_id?: string
  roleId?: string
}


export interface PluginBridgeDeleteRoleResult {
  ok: boolean
  role_id: string
}

/** `update_settings`：更新允许的应用设置（整壳白名单字段）�?*/

export interface PluginBridgeUpdateSettingsParams {
  /** �?`ui_theme` 二选一 */
  theme?: 'light' | 'dark' | 'system'
  ui_theme?: 'light' | 'dark' | 'system'
  interaction_mode?: string
  /** 与主应用设置「远端失败自动降级内置」一致：`"0"` / `"1"`�?*/
  remote_fallback_to_builtin?: string
  [key: string]: unknown
}

/** `get_conversation_list`：列出本地会话元数据�?*/

export interface PluginBridgeConversationListItem {
  session_namespace: string
  turn_count: number
  last_at: string | null
}


export interface PluginBridgeGetConversationListResult {
  items: PluginBridgeConversationListItem[]
}

/** 目录插件�?`OclivePluginBridge.invoke` 对应的后端入口（一般无需在主 UI 调用）�?*/

export async function pluginBridgeInvoke(req: {
  pluginId: string
  assetRel: string
  command: string
  params?: unknown
}): Promise<unknown> {
  return invokeWithFriendlyError<unknown>('plugin_bridge_invoke', {
    req: {
      pluginId: req.pluginId,
      assetRel: req.assetRel,
      command: req.command,
      params: req.params ?? {},
    },
  })
}

/** A2.2：环境自检（Ollama、角色根目录、应用数据可写）�?*/
