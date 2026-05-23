import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'
import type { RoleInfo } from './role'

export interface DirectoryPluginSlots {
  memory?: string | null
  emotion?: string | null
  event?: string | null
  prompt?: string | null
  llm?: string | null
  agent?: string | null
}

/** �?`settings.json` �?`plugin_backends` 一致（snake_case，与后端 serde 对齐�?*/

export interface PluginBackends {
  memory: 'builtin' | 'builtin_v2' | 'remote' | 'local' | 'directory'
  /** `memory === "local"` 时可选：�?`_local_plugins` �?descriptor �?`provider_id` 一�?*/
  local_memory_provider_id?: string | null
  emotion: 'builtin' | 'builtin_v2' | 'remote' | 'directory'
  event: 'builtin' | 'builtin_v2' | 'remote' | 'directory'
  prompt: 'builtin' | 'builtin_v2' | 'remote' | 'directory'
  llm: 'ollama' | 'remote' | 'directory'
  agent: 'builtin' | 'remote' | 'directory'
  /** 各模块为 `directory` 时对应的 manifest `id`（见 DIRECTORY_PLUGINS.md�?*/
  directory_plugins?: DirectoryPluginSlots
}


export interface PluginBackendsOverride {
  memory?: PluginBackends['memory'] | null
  local_memory_provider_id?: string | null
  emotion?: PluginBackends['emotion'] | null
  event?: PluginBackends['event'] | null
  prompt?: PluginBackends['prompt'] | null
  llm?: PluginBackends['llm'] | null
  agent?: PluginBackends['agent'] | null
  /** 会话级与包内按槽合并（当�?UI 未编辑；仅展示与调试�?*/
  directory_plugins?: DirectoryPluginSlots | null
}


export type PluginBackendSource = 'pack_default' | 'session_override' | 'env_override'


export interface PluginBackendsSourceMap {
  memory: PluginBackendSource
  emotion: PluginBackendSource
  event: PluginBackendSource
  prompt: PluginBackendSource
  llm: PluginBackendSource
  agent: PluginBackendSource
}


export interface PluginResolutionDebugInfo {
  app_version: string
  api_version: number
  schema_version: number
  role_id: string
  session_namespace: string
  plugin_backends_pack_default: PluginBackends
  plugin_backends_session_override?: PluginBackendsOverride | null
  plugin_backends_effective: PluginBackends
  plugin_backends_effective_sources: PluginBackendsSourceMap
  llm_env_override?: string | null
  remote_plugin_url_configured: boolean
  remote_llm_url_configured: boolean
  local_provider_ids: string[]
  local_provider_count: number
}

/**
 * `load_role` 返回的扁平快照�?
 * 身份相关：`default_relation` 来自角色包；`current_user_relation` 为解析后的有效键（`identity_binding: per_scene` 时场景覆盖优先，否则为全局 manifest 默认�?DB）；
 * `use_manifest_default` 仅表示用户是否选了「默认身份」选项；好�?阶段与当前有效身份一致�?
 */
/** `evolution.personality_source` */

export async function setSessionPluginBackend(
  roleId: string,
  module: 'memory' | 'emotion' | 'event' | 'prompt' | 'llm' | 'agent',
  /** 与后�?`parse_backend_wire` 一致，�?`builtin_v2`、`directory`、`remote` */
  backend?: string | null,
  localMemoryProviderId?: string,
  sessionId?: string | null,
  directoryId?: string | null,
): Promise<RoleInfo> {
  const req: Record<string, unknown> = {
    role_id: roleId,
    module,
    session_id: sessionId ?? null,
  }
  if (backend !== undefined) {
    req.backend = backend
  }
  if (localMemoryProviderId !== undefined) {
    req.local_memory_provider_id = localMemoryProviderId
  }
  if (directoryId !== undefined) {
    req.directory_id = directoryId
  }
  return invokeWithFriendlyError<RoleInfo>('set_session_plugin_backend', {
    req,
  })
}


export async function setSessionSlotOverride(
  roleId: string,
  slotKey: string,
  patch: {
    backend?: string | null
    plugin?: string | null
    plugins?: string[] | null
    model?: string | null
    localMemoryProviderId?: string | null
  },
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_session_slot_override', {
    req: {
      role_id: roleId,
      slot_key: slotKey,
      backend: patch.backend ?? null,
      plugin: patch.plugin ?? null,
      plugins: patch.plugins ?? null,
      model: patch.model ?? null,
      local_memory_provider_id: patch.localMemoryProviderId ?? null,
      session_id: sessionId ?? null,
    },
  })
}


export async function clearSessionSlotOverride(
  roleId: string,
  slotKey: string,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('clear_session_slot_override', {
    req: {
      role_id: roleId,
      slot_key: slotKey,
      session_id: sessionId ?? null,
    },
  })
}


export async function clearAllSessionSlotOverrides(
  roleId: string,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('clear_all_session_slot_overrides', {
    req: {
      role_id: roleId,
      session_id: sessionId ?? null,
    },
  })
}

/** 将完�?`slot_registry` 写回 `pipeline.ocblueprint`（蓝�?v2 架构图写盘）�?*/

export async function saveRoleSlotRegistry(
  roleId: string,
  slotRegistry: import('../lib/slotRegistry').SlotRegistryMap,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('save_role_slot_registry', {
    req: {
      role_id: roleId,
      slot_registry: slotRegistry,
    },
  })
}

/** �?`author.json` �?`suggested_plugin_backends` 写入当前会话后端覆盖�?*/

export async function applyAuthorSuggestedPluginBackends(
  roleId: string,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('apply_author_suggested_plugin_backends', {
    req: {
      role_id: roleId,
      session_id: sessionId ?? null,
    },
  })
}


export async function getPluginResolutionDebug(
  roleId: string,
  sessionId?: string | null,
): Promise<PluginResolutionDebugInfo> {
  return invokeWithFriendlyError<PluginResolutionDebugInfo>(
    'get_plugin_resolution_debug',
    {
      req: {
        role_id: roleId,
        session_id: sessionId ?? null,
      },
    },
  )
}


export type HotkeyAction
  = | {
    type: 'openPluginSlot'
    pluginId: string
    slot: string
    appearanceId?: string
  }
  | { type: 'openLauncherList' }


export interface HotkeyBinding {
  id: string
  accelerator: string
  enabled: boolean
  action: HotkeyAction
}


export interface HotkeyBindingsFile {
  schemaVersion: number
  bindings: HotkeyBinding[]
}


export async function getHotkeyBindings(): Promise<HotkeyBindingsFile> {
  return invokeWithFriendlyError<HotkeyBindingsFile>('get_hotkey_bindings', {})
}


export async function saveHotkeyBindings(
  file: HotkeyBindingsFile,
): Promise<void> {
  return invokeWithFriendlyError<void>('save_hotkey_bindings', {
    bindings: file,
  })
}

/** B2：对指定目录插件懒启动后透传 JSON-RPC（方法名�?params 由插件定义）�?*/
