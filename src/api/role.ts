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
  /** �?�?��??�?�置�??�?��?好�??度�?0�?00�?�?�??换顶栏身份�?��?�?步�?��?�?�好�??*/
  initial_favorability: number
}

/** �??�??�?��?� + manifest `life_schedule` �?��?��??�?�?�活�??*/

export interface LifeStateDto {
  label: string
  activity_key: string
  busy_level: number
  preferred_scene_id: string | null
}

/** �?`plugin_backends.directory_plugins` �?�?��?snake_case JSON �?段�?*/

export type PersonalitySource = 'vector' | 'profile'

/** �?�?��??根�?��? `ui.json`�?�?�?端 `UiConfig` 对齐�?�?槽�?�含�?�号�? */

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
  /** �?�否�??中�??�?认身份�?��?�?�?� manifest `default_relation`�?*/
  use_manifest_default: boolean
  /** �?�?��?声�?�?��?DB�?*/
  remote_life_enabled: boolean
  /** �?�?��??建议�?认�?�否�?启�?�?��?声�?settings.json �??remote_presence.default_enabled�?*/
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** `evolution.personality_source`�?缺�?�为 vector */
  personality_source?: PersonalitySource
  /** manifest �??OLLAMA_MODEL �??�?��?�?认 */
  effective_ollama_model: string
  /** �?��?�?�?身份 vs �??�?��?��?�??�?manifest `identity_binding`�?*/
  identity_binding: 'global' | 'per_scene'
  /** �?�?�交�?模式�?DB�?�?`immersive` | `pure_chat` */
  interaction_mode: 'immersive' | 'pure_chat'
  /** �?�?��??settings.json 建议�?认�?可�??�? */
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** �?�?��?��?�?��?��?�?��?�置�??�?��?�中�?�段�?�为 null�?*/
  current_life: LifeStateDto | null
  /** 模�?�??子系�?�?端�?�? `PluginHost` 解�?��?�?��? */
  plugin_backends: PluginBackends
  /** �?�?��?话�?�??�?�?��?�??�?�为 null�?*/
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** 叠�?��?话�?�??�?�??�??�??�?端 */
  plugin_backends_effective?: PluginBackends
  /** 叠�?��?�??�?端来源�?pack/session/env�?*/
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** �?�?��??`ui.json`�?主�?�?��?�?�?��?槽�? */
  pack_ui_config: PackUiConfig
  /** `author.suggested_ui` �?�??�?��??�??�?? UI �?�线�?�?�?端 `pack_ui_baseline` �?�?��? */
  pack_ui_baseline?: PackUiConfig
  /** 可�??`author.json` */
  author_pack?: AuthorPackFile | null
  /** v2 �?��?� `slot_registry`�?legacy �?null�?*/
  slot_registry_pack?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_registry_effective?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_session_overridden_keys?: string[]
  blueprint_groups_pack?: import('../lib/slotRegistry').SlotGroupsMap | null
}


export interface SceneLabelEntry {
  id: string
  label: string
}

/** `get_role_info` / `switch_scene` �?�? UI �?步�??快�?��?身份�?段语�?�?{@link RoleData}�??*/

export interface RoleInfo {
  role_id: string
  role_name: string
  version: string
  author: string
  description: string
  current_favorability: number
  current_emotion: string
  personality_vector: number[]
  /** `evolution.personality_source`�?缺�?�为 vector */
  personality_source?: PersonalitySource
  last_interaction?: string | null
  scenes: string[]
  /** �?scenes 顺序�?�?��?label 来�?��?�?��??scene.json �??�??置�?��?*/
  scene_labels: SceneLabelEntry[]
  current_scene: string | null
  /** �?��?��?�?�?��?��?DB�?�?�?current_scene 可不�?*/
  user_presence_scene: string | null
  virtual_time_ms: number
  user_relations: UserRelationDto[]
  default_relation: string
  current_user_relation: string
  /** �?�否�??中�??�?认身份�?��?�?�??�?�?��?`OCLIVE_DEFAULT_RELATION_SENTINEL`�?*/
  use_manifest_default: boolean
  /** �?�系�?�段�?`role_runtime.relation_state`�?*/
  relation_state: string
  remote_life_enabled: boolean
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** manifest �??OLLAMA_MODEL �??�?��?�?认 */
  effective_ollama_model: string
  /** �?��?�?�?身份 vs �??�?��?��?�??�?manifest `identity_binding`�?*/
  identity_binding: 'global' | 'per_scene'
  interaction_mode: 'immersive' | 'pure_chat'
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** �?�?��?��?�?��?��?�?��?�置�??�?��?�中�?�段�?�为 null�?*/
  current_life: LifeStateDto | null
  /** 模�?�??子系�?�?端�?�? `load_role` �?�?��? */
  plugin_backends: PluginBackends
  /** �?�?��?话�?�??�?�?��?�??�?�为 null�?*/
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** 叠�?��?话�?�??�?�??�??�??�?端 */
  plugin_backends_effective?: PluginBackends
  /** 叠�?��?�??�?端来源�?pack/session/env�?*/
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** �?�否已�?磁�??�??建�?�??�?�?��?索�?*/
  knowledge_enabled?: boolean
  /** �?��?�?条�?��?�?��?�载索�?�?��?0 */
  knowledge_chunk_count?: number
  /** �?�?��??`ui.json`�?主�?�?��?�?�?��?槽�? */
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

/** `switch_scene` �?�平�??�?�??�?RoleInfo �?段 + 可�??�?��?�欢�?语 */

export async function loadRole(roleId: string): Promise<RoleData> {
  return invokeWithFriendlyError<RoleData>('load_role', { roleId })
}

/** �?��??件�?�?��??�?�??绝对路�?�?否�??`null`�?不�??�??�?�??*/

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

/** `sessionId` �?�?�?息�??�?�?id �?�?��?��?�?�??该�?�名空�?��?�?? `plugin_backends_*` �?快�?��??*/

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

/** 移�?��?�?��?��?��??身份�?�??�?�?��? `set_user_relation` �?��?可恢复�??�?认身份�?��?��?�?��??*/

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
    role_id: roleId,
    dest_path: destPath,
  })
}


export interface RolePackPeek {
  id: string
  name: string
  version: string
}

/** �?�?�?�?��??�?`srcPath` 可为 `.ocpak` / `.zip` �??已解�??�?��?�?�? `roles/{id}/` �?�?��?�??*/

export async function peekRolePack(srcPath: string): Promise<RolePackPeek> {
  return invokeWithFriendlyError<RolePackPeek>('peek_role_pack_command', {
    src_path: srcPath,
  })
}

/** 导�?��?�?��??�?�?�?�?�?��?��??缩�??�??已解�??�?��?�??*/

export async function importRolePack(
  srcPath: string,
  overwrite: boolean,
): Promise<string> {
  return invokeWithFriendlyError<string>('import_role_pack_command', {
    src_path: srcPath,
    overwrite,
  })
}

