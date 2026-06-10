import { normalizeSlotBackendWire } from '../lib/slotRegistry'
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

/** Matches `settings.json` → `plugin_backends` (snake_case, aligned with backend serde).
 *  Read path may include legacy `builtin_v2` wire alias; call `normalizePluginBackends` after load. */

export interface PluginBackends {
  memory: 'builtin' | 'builtin_v2' | 'remote' | 'local' | 'directory'
  /** When `memory === "local"`: optional `_local_plugins` descriptor `provider_id` */
  local_memory_provider_id?: string | null
  emotion: 'builtin' | 'builtin_v2' | 'remote' | 'directory'
  event: 'builtin' | 'builtin_v2' | 'remote' | 'directory'
  prompt: 'builtin' | 'builtin_v2' | 'remote' | 'directory'
  llm: 'ollama' | 'remote' | 'directory'
  agent: 'builtin' | 'remote' | 'directory'
  /** Manifest `id` for each module when backend is `directory` (see DIRECTORY_PLUGINS.md) */
  directory_plugins?: DirectoryPluginSlots
}

const SLOT_BACKEND_KEYS = ['memory', 'emotion', 'event', 'prompt'] as const

/** Normalize legacy `builtin_v2` wire values to `builtin` for display. */
export function normalizePluginBackends<T extends PluginBackends | PluginBackendsOverride>(backends: T): T {
  const next = { ...backends }
  for (const key of SLOT_BACKEND_KEYS) {
    const value = next[key]
    if (typeof value === 'string')
      next[key] = normalizeSlotBackendWire(value) as T[typeof key]
  }
  return next
}


export interface PluginBackendsOverride {
  memory?: PluginBackends['memory'] | null
  local_memory_provider_id?: string | null
  emotion?: PluginBackends['emotion'] | null
  event?: PluginBackends['event'] | null
  prompt?: PluginBackends['prompt'] | null
  llm?: PluginBackends['llm'] | null
  agent?: PluginBackends['agent'] | null
  /** Session-level merged with pack per slot (current UI may not edit; display/debug only) */
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
 * Flat snapshot from `load_role`.
 * Identity: `default_relation` from role pack; `current_user_relation` is the resolved effective key
 * (`identity_binding: per_scene` prefers scene override, else global manifest default / DB);
 * `use_manifest_default` means user picked the "default identity" option; relation stage matches effective identity.
 */
/** `evolution.personality_source` */

export async function setSessionPluginBackend(
  roleId: string,
  module: 'memory' | 'emotion' | 'event' | 'prompt' | 'llm' | 'agent',
  /** Matches backend `parse_backend_wire`; legacy `builtin_v2` is normalized to `builtin` on save */
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
    req.backend = backend == null ? null : normalizeSlotBackendWire(backend)
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
      backend: patch.backend == null ? null : normalizeSlotBackendWire(patch.backend),
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

/** Write full `slot_registry` back to `pipeline.ocblueprint` (blueprint v2 architecture graph persist) */

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

/** Write `author.json` → `suggested_plugin_backends` into current session backend override */

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

/** B2: lazy-start directory plugin then passthrough JSON-RPC (method/params defined by plugin) */
