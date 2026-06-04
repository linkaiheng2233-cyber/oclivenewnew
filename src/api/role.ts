export * from './role/expert'
import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'
import type {
  PluginBackends,
  PluginBackendsOverride,
  PluginBackendsSourceMap,
} from './settings'

export interface UserRelationDto {
  id: string
  name: string
  prompt_hint: string
  favor_multiplier: number
  /** Role pack configured initial favorability (0–100); switching top-bar identity re-seeds favorability */
  initial_favorability: number
}

/** Virtual-time schedule inference from manifest `life_schedule` */

export interface LifeStateDto {
  label: string
  activity_key: string
  busy_level: number
  preferred_scene_id: string | null
}

/** Matches `plugin_backends.directory_plugins` snake_case JSON fields */

export type PersonalitySource = 'vector' | 'profile'

/** Normalized role pack root `ui.json`; aligned with frontend `UiConfig`; slot keys use dots */

export interface PackUiSlotConfig {
  order: string[]
  visible: string[]
}


export interface PackUiSlots {
  'chat_toolbar': PackUiSlotConfig
  'settings.panel': PackUiSlotConfig
  'role.detail': PackUiSlotConfig
  'sidebar': PackUiSlotConfig
  'chat.header': PackUiSlotConfig
}


export interface PackUiTheme {
  primaryColor?: string
  backgroundColor?: string
  fontFamily?: string
}


export interface PackUiLayout {
  sidebar?: string
  chatInput?: string
}


export interface PackUiConfig {
  shell: string
  theme: PackUiTheme
  layout: PackUiLayout
  slots: PackUiSlots
}


export interface AuthorRecommendedPlugin {
  id: string
  version_range?: string | null
  slots?: string[]
  for_backends?: string[]
  optional?: boolean
  note?: string | null
}


export interface AuthorPackFile {
  schema_version: number
  summary?: string
  detail_markdown?: string
  recommended_plugins?: AuthorRecommendedPlugin[]
  suggested_ui?: PackUiConfig | null
  suggested_plugin_backends?: PluginBackends | null
}


export interface RoleData {
  role_id: string
  name: string
  version: string
  author: string
  description: string
  personality_vector: number[]
  current_favorability: number
  current_emotion: string
  memory_count: number
  event_count: number
  user_relations: UserRelationDto[]
  default_relation: string
  relation_state: string
  current_user_relation: string
  /** Whether "default identity" is selected (manifest `default_relation`) */
  use_manifest_default: boolean
  /** Remote life enabled (DB) */
  remote_life_enabled: boolean
  /** Role pack suggested default for remote life (`settings.json` → `remote_presence.default_enabled`) */
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** `evolution.personality_source`; defaults to vector */
  personality_source?: PersonalitySource
  /** Effective model: manifest `OLLAMA_MODEL` or user default */
  effective_ollama_model: string
  /** Global identity vs per-scene (`manifest identity_binding`) */
  identity_binding: 'global' | 'per_scene'
  /** Interaction mode in DB: `immersive` | `pure_chat` */
  interaction_mode: 'immersive' | 'pure_chat'
  /** Suggested default from role pack settings; may be null */
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** Virtual-time schedule inference from life placement; null when none */
  current_life: LifeStateDto | null
  /** Module subsystem backends resolved by `PluginHost` */
  plugin_backends: PluginBackends
  /** Session conversation override; null when none */
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** Effective backends after session override */
  plugin_backends_effective?: PluginBackends
  /** Effective backend sources (pack/session/env) */
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** Role pack `ui.json` shell / theme / layout / slots */
  pack_ui_config: PackUiConfig
  /** `author.suggested_ui` or role pack baseline UI; frontend `pack_ui_baseline` */
  pack_ui_baseline?: PackUiConfig
  /** Optional `author.json` */
  author_pack?: AuthorPackFile | null
  /** v2 blueprint `slot_registry`; null for legacy */
  slot_registry_pack?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_registry_effective?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_session_overridden_keys?: string[]
  blueprint_groups_pack?: import('../lib/slotRegistry').SlotGroupsMap | null
}


export interface SceneLabelEntry {
  id: string
  label: string
}

/** Flat UI snapshot from `get_role_info` / `switch_scene`; identity fields match {@link RoleData} */

export interface RoleInfo {
  role_id: string
  role_name: string
  version: string
  author: string
  description: string
  current_favorability: number
  current_emotion: string
  personality_vector: number[]
  /** `evolution.personality_source`; defaults to vector */
  personality_source?: PersonalitySource
  last_interaction?: string | null
  scenes: string[]
  /** Labels in `scenes` order; from role pack `scene.json` or manifest */
  scene_labels: SceneLabelEntry[]
  current_scene: string | null
  /** User narrative scene in DB; may differ from `current_scene` */
  user_presence_scene: string | null
  virtual_time_ms: number
  user_relations: UserRelationDto[]
  default_relation: string
  current_user_relation: string
  /** Whether "default identity" is selected; sentinel is `OCLIVE_DEFAULT_RELATION_SENTINEL` */
  use_manifest_default: boolean
  /** Relation stage from `role_runtime.relation_state` */
  relation_state: string
  remote_life_enabled: boolean
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** Effective model: manifest `OLLAMA_MODEL` or user default */
  effective_ollama_model: string
  /** Global identity vs per-scene (`manifest identity_binding`) */
  identity_binding: 'global' | 'per_scene'
  interaction_mode: 'immersive' | 'pure_chat'
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** Virtual-time schedule inference; null when none */
  current_life: LifeStateDto | null
  /** Module subsystem backends from `load_role` snapshot */
  plugin_backends: PluginBackends
  /** Session override; null when none */
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** Effective backends after session override */
  plugin_backends_effective?: PluginBackends
  /** Effective backend sources (pack/session/env) */
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** Whether worldview knowledge index is loaded from disk */
  knowledge_enabled?: boolean
  /** Knowledge chunk count in loaded index; 0 when disabled */
  knowledge_chunk_count?: number
  /** Role pack `ui.json` shell / theme / layout / slots */
  pack_ui_config: PackUiConfig
  pack_ui_baseline?: PackUiConfig
  author_pack?: AuthorPackFile | null
  slot_registry_pack?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_registry_effective?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_session_overridden_keys?: string[]
  blueprint_groups_pack?: import('../lib/slotRegistry').SlotGroupsMap | null
  dual_core_enabled?: boolean
  pipeline_experimental_actions?: string[]
}

/** `switch_scene` flattened `RoleInfo` fields plus optional scene welcome line */

export async function loadRole(roleId: string): Promise<RoleData> {
  return invokeWithFriendlyError<RoleData>('load_role', { roleId })
}

/** Resolve role asset absolute path; returns `null` when missing (does not throw) */

export async function resolveRoleAssetPath(
  roleId: string,
  relative: string,
): Promise<string | null> {
  return invoke<string | null>('resolve_role_asset_path', {
    roleId,
    relative,
  })
}

export async function readRoleAssetBytes(
  roleId: string,
  relative: string,
): Promise<number[] | null> {
  return invokeWithFriendlyError<number[] | null>('read_role_asset_bytes', {
    roleId,
    relative,
  })
}

/** When `sessionId` is passed, scopes `get_role_info` to that conversation namespace for faster `plugin_backends_*` */

export async function getRoleInfo(
  roleId: string,
  sessionId?: string | null,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('get_role_info', {
    req: {
      role_id: roleId,
      session_id: sessionId ?? null,
    },
  })
}


export async function listRoles(): Promise<Array<{ id: string, name: string }>> {
  const rows = await invokeWithFriendlyError<
    Array<{ id: string, name: string, version: string, author: string }>
  >('list_roles', {})
  return rows.map(r => ({ id: r.id, name: r.name }))
}


export async function switchRole(roleId: string): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('switch_role', { roleId })
}


export async function setUserRelation(
  roleId: string,
  relation: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_user_relation', {
    req: { role_id: roleId, relation },
  })
}


export async function setEvolutionFactor(
  roleId: string,
  eventImpactFactor: number,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_evolution_factor', {
    req: { role_id: roleId, event_impact_factor: eventImpactFactor },
  })
}


export async function setRemoteLifeEnabled(
  roleId: string,
  enabled: boolean,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_remote_life_enabled', {
    req: { role_id: roleId, enabled },
  })
}


export async function setRoleInteractionMode(
  roleId: string,
  mode: 'immersive' | 'pure_chat',
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_role_interaction_mode', {
    req: { role_id: roleId, mode },
  })
}


export async function setSceneUserRelation(
  roleId: string,
  sceneId: string,
  relation: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('set_scene_user_relation', {
    req: { role_id: roleId, scene_id: sceneId, relation },
  })
}

/** Remove per-scene identity override; use `set_user_relation` sentinel to restore default identity option */

export async function clearSceneUserRelation(
  roleId: string,
  sceneId: string,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('clear_scene_user_relation', {
    req: { role_id: roleId, scene_id: sceneId },
  })
}


export async function exportRolePack(
  roleId: string,
  destPath: string,
): Promise<void> {
  return invokeWithFriendlyError<void>('export_role_pack_command', {
    roleId,
    destPath,
  })
}


export interface RolePackPeek {
  id: string
  name: string
  version: string
}

/** Import role pack: `srcPath` may be `.ocpak`, `.zip`, or extracted directory matching `roles/{id}/` layout */

export async function peekRolePack(srcPath: string): Promise<RolePackPeek> {
  return invokeWithFriendlyError<RolePackPeek>('peek_role_pack_command', {
    srcPath,
  })
}

/** Export role pack to path (ZIP archive or extracted directory) */

export async function importRolePack(
  srcPath: string,
  overwrite: boolean,
): Promise<string> {
  return invokeWithFriendlyError<string>('import_role_pack_command', {
    srcPath,
    overwrite,
  })
}

