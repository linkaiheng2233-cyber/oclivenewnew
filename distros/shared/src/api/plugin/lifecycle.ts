import { invokeWithFriendlyError } from '../helpers'
import type { PackUiConfig, PackUiSlotConfig, PackUiSlots } from '../role'

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

/** Aligned with `models::author_pack::AuthorPackFile` (snake_case fields). */
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
  /** manifest `ui_slots[].appearance_id`; empty string is default appearance */
  appearanceId?: string
  /** manifest `ui_slots[].label` */
  label?: string | null
  /** manifest `ui_slots[].entry`, path relative to plugin root */
  entry: string
  /** manifest `vueComponent`; host compiles Vue when set; on failure falls back to `url` iframe */
  vueComponent?: string | null
  url: string
}

/** Read a text asset from plugin root (e.g. host-compiled `.vue` source). */
export async function readPluginAssetText(
  pluginId: string,
  rel: string,
): Promise<string> {
  return invokeWithFriendlyError<string>('read_plugin_asset_text', {
    pluginId,
    rel,
  })
}

/** Directory-plugin bootstrap: shell URL, scanned plugin ids, dev mode, UI slot list. */
export interface DirectoryPluginBootstrap {
  shellUrl?: string | null
  shellPluginId?: string | null
  /** Dev-only shell Vue entry; release builds always use `shellUrl` HTML. */
  shellVueEntry?: string | null
  /** Persisted preference; release builds enforce HTML regardless of false. */
  forceIframeMode?: boolean
  pluginIds: string[]
  developerMode: boolean
  /** Union of enabled plugins' `manifest.bridge.events` host event names */
  subscribedHostEvents: string[]
  uiSlots: PluginUiSlotInfo[]
}

/** Per `role_id` in-flight coalescing for bootstrap IPC (avoids duplicate slot loads on rapid remount). */
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
  if (existing)
    return existing
  const p = invokeWithFriendlyError<DirectoryPluginBootstrap>(
    'get_directory_plugin_bootstrap',
    { roleId: roleId ?? null },
  ).finally(() => {
    if (directoryBootstrapInflight.get(key) === p)
      directoryBootstrapInflight.delete(key)
  })
  directoryBootstrapInflight.set(key, p)
  return p
}

/** `app_data/plugin_state.json` slot sections (serde snake_case on disk). */
export interface PluginStateFile {
  disabled_plugins: string[]
  slot_order: Record<string, string[]>
  disabled_slot_contributions: Record<string, string[]>
  /** `plugin_id` → `slot` → `appearance_id` */
  slot_appearance?: Record<string, Record<string, string>>
  /** Persisted preference; release builds force HTML/iframe regardless of false. */
  force_iframe_mode?: boolean
}

/** Role-scoped plugin UI state; extends file shape with shell id (backend `RolePluginStateDto`). */
export interface RolePluginState extends PluginStateFile {
  shellPluginId: string
}

export interface PluginStateGetResponse {
  role: RolePluginState
  /** Backend `serde(rename_all = "camelCase")` → `globalDefaults` */
  globalDefaults: RolePluginState
}

/** Coalesce `get_plugin_state(role_id)` per role_id dimension. */
const pluginStateInflight = new Map<string, Promise<PluginStateGetResponse>>()

function pluginStateCacheKey(roleId: string): string {
  const t = roleId.trim()
  return t.length > 0 ? t : '__default__'
}

/** Role-pack root `ui.json` slot order/visibility (backend `UiConfig` shape). */
export interface SlotConfig {
  order: string[]
  visible: string[]
  /** plugin id → default `appearance_id` for that slot */
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
  /** manifest has `uiTemplate` or `uiSchema.fields` */
  hasUiSettings?: boolean
  /** manifest declares `process`; settings panel can spawn JSON-RPC child process */
  hasRpcProcess: boolean
  /** manifest declares `rpcMethods`; settings panel can fill method names */
  declaresRpcMethods?: boolean
  isShell: boolean
  uiSlotNames: string[]
  /** One entry per manifest `ui_slots` variant (slot + appearance) */
  uiSlotVariants?: UiSlotVariantInfo[]
  provides: string[]
  description?: string | null
  author?: string | null
  permissions?: string[]
  /** `ok` / `missing` / `mismatch` */
  dependencyStatus: string
  dependencyIssues: string[]
}

/** Coalesce `get_directory_plugin_catalog` IPC (global in-flight dedup). */
const directoryCatalogInflight = new Map<
  string,
  Promise<DirectoryPluginCatalogEntry[]>
>()
const DIRECTORY_CATALOG_COALESCE_KEY = '__global__'

export async function getDirectoryPluginCatalog(): Promise<DirectoryPluginCatalogEntry[]> {
  const existing = directoryCatalogInflight.get(DIRECTORY_CATALOG_COALESCE_KEY)
  if (existing)
    return existing
  const p = invokeWithFriendlyError<DirectoryPluginCatalogEntry[]>(
    'get_directory_plugin_catalog',
    {},
  ).finally(() => {
    if (directoryCatalogInflight.get(DIRECTORY_CATALOG_COALESCE_KEY) === p)
      directoryCatalogInflight.delete(DIRECTORY_CATALOG_COALESCE_KEY)
  })
  directoryCatalogInflight.set(DIRECTORY_CATALOG_COALESCE_KEY, p)
  return p
}

export async function getPluginState(
  roleId: string,
): Promise<PluginStateGetResponse> {
  const key = pluginStateCacheKey(roleId)
  const existing = pluginStateInflight.get(key)
  if (existing)
    return existing
  const p = invokeWithFriendlyError<PluginStateGetResponse>('get_plugin_state', {
    roleId,
  }).finally(() => {
    if (pluginStateInflight.get(key) === p)
      pluginStateInflight.delete(key)
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

/** Reset persisted state to role-pack default `ui.json` (clears per-role plugin UI overrides). */
export async function resetPluginStateToRoleDefault(
  roleId: string,
): Promise<void> {
  pluginStateInflight.delete(pluginStateCacheKey(roleId))
  return invokeWithFriendlyError<void>('reset_plugin_state_to_role_default', {
    roleId,
  })
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

/**
 * manifest `shell.bridge.invoke` may declare **command tokens** or **legacy method names**
 * (e.g. `get_conversation` vs `read:conversation`).
 * Bridge dispatch requires **`type`: `"ocliveplugin"`** and source **`shell.entry`** (HTML) or **`shell.vueEntry`** (Vue shell).
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
    | 'plugin_rpc_invoke'

/** Shell `OclivePluginBridge.invoke('update_memory', params)` */
export interface PluginBridgeUpdateMemoryParams {
  role_id: string
  content: string
  /** 0–1, default 0.5 */
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

/** Host prompt fragment update; may return `not_implemented`. */
export interface PluginBridgeUpdatePromptParams {
  role_id: string
  /** Optional host contract key */
  fragment_key?: string
  content?: string
}

/** `plugin_bridge_invoke` for `send_message`; params may use `text` instead of `user_message`. */
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

/** `export_conversation` / `export_chat_logs`; `format`: `json` | `txt`. */
export interface PluginBridgeExportConversationParams {
  role_id: string
  format?: string
  session_id?: string | null
}

export interface PluginBridgeExportConversationResult {
  content: string
  suggested_filename: string
}

/** `import_role` from `.zip` / `.ocpak` or extracted directory path. */
export interface PluginBridgeImportRoleParams {
  path: string
  /** Alias for `src_path` */
  src_path?: string
  overwrite?: boolean
}

export interface PluginBridgeImportRoleResult {
  role_id: string
  ok: boolean
}

/** `delete_role`; removes role pack and associated data. */
export interface PluginBridgeDeleteRoleParams {
  role_id?: string
  roleId?: string
}

export interface PluginBridgeDeleteRoleResult {
  ok: boolean
  role_id: string
}

/** `update_settings`; allowed keys include theme and shell-renamed fields. */
export interface PluginBridgeUpdateSettingsParams {
  /** Alias for `ui_theme` */
  theme?: 'light' | 'dark' | 'system'
  ui_theme?: 'light' | 'dark' | 'system'
  interaction_mode?: string
  /** App-level remote fallback; backend stores `"0"` / `"1"` strings */
  remote_fallback_to_builtin?: string
  [key: string]: unknown
}

/** `get_conversation_list`; session list metadata. */
export interface PluginBridgeConversationListItem {
  session_namespace: string
  turn_count: number
  last_at: string | null
}

export interface PluginBridgeGetConversationListResult {
  items: PluginBridgeConversationListItem[]
}

/** Directory-plugin `OclivePluginBridge.invoke` passthrough to host commands (same surface as main UI). */
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
