import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'

export interface UserRelationDto {
  id: string
  name: string
  prompt_hint: string
  favor_multiplier: number
  /** è§?è?²å??é?ç½®ç??å?å§?å¥½æ??åº¦ï¼?0ï½?00ï¼?ï¼?å??æ¢é¡¶æ èº«ä»½æ?¶ä¼?å?æ­¥å?°å½?å?å¥½æ??*/
  initial_favorability: number
}

/** è??æ??æ?¶é?´ + manifest `life_schedule` æ?¨æ?­ç??å½?å?æ´»å??*/

export interface LifeStateDto {
  label: string
  activity_key: string
  busy_level: number
  preferred_scene_id: string | null
}

/** ä¸?`plugin_backends.directory_plugins` ä¸?è?´ï¼?snake_case JSON å­?æ®µï¼?*/

export type PersonalitySource = 'vector' | 'profile'

/** è§?è?²å??æ ¹ç?®å½? `ui.json`ï¼?ä¸?å?ç«¯ `UiConfig` å¯¹é½ï¼?æ?æ§½é?®å«ç?¹å·ï¼? */

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
  /** æ?¯å¦é??ä¸­ã??é»?è®¤èº«ä»½ã?ï¼?è·?é? manifest `default_relation`ï¼?*/
  use_manifest_default: boolean
  /** å¼?å?°å¿?å£°å¼?å?³ï¼?DBï¼?*/
  remote_life_enabled: boolean
  /** è§?è?²å??å»ºè®®é»?è®¤æ?¯å¦å¼?å¯å¼?å?°å¿?å£°ï¼?settings.json â??remote_presence.default_enabledï¼?*/
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** `evolution.personality_source`ï¼?ç¼ºç?ä¸º vector */
  personality_source?: PersonalitySource
  /** manifest â??OLLAMA_MODEL â??å?¨å±?é»?è®¤ */
  effective_ollama_model: string
  /** å?¨å±?å?ä¸?èº«ä»½ vs æ??å?ºæ?¯è¦?ç??ï¼?manifest `identity_binding`ï¼?*/
  identity_binding: 'global' | 'per_scene'
  /** å½?å?äº¤äº?æ¨¡å¼ï¼?DBï¼?ï¼?`immersive` | `pure_chat` */
  interaction_mode: 'immersive' | 'pure_chat'
  /** è§?è?²å??settings.json å»ºè®®é»?è®¤ï¼?å¯é??ï¼? */
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** å½?å?æ?¥ç¨?æ?¨æ?­ï¼?æ? é?ç½®æ??æ?ªå?½ä¸­æ?¶æ®µæ?¶ä¸º nullï¼?*/
  current_life: LifeStateDto | null
  /** æ¨¡å?å??å­ç³»ç»?å?ç«¯ï¼?ä¸? `PluginHost` è§£æ?ä¸?è?´ï¼? */
  plugin_backends: PluginBackends
  /** å½?å?ä¼?è¯è¦?ç??ï¼?æ? è¦?ç??æ?¶ä¸º nullï¼?*/
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** å å? ä¼?è¯è¦?ç??å?ç??æ??æ??å?ç«¯ */
  plugin_backends_effective?: PluginBackends
  /** å å? å?ç??å?ç«¯æ¥æºï¼?pack/session/envï¼?*/
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** è§?è?²å??`ui.json`ï¼?ä¸»é¢?ã?å¸?å±?ã?æ?æ§½ï¼? */
  pack_ui_config: PackUiConfig
  /** `author.suggested_ui` ä¼?å??æ?¶ç??æ??æ?? UI å?ºçº¿ï¼?ä¸?å?ç«¯ `pack_ui_baseline` ä¸?è?´ï¼? */
  pack_ui_baseline?: PackUiConfig
  /** å¯é??`author.json` */
  author_pack?: AuthorPackFile | null
  /** v2 è?å?¾ `slot_registry`ï¼?legacy ä¸?nullï¼?*/
  slot_registry_pack?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_registry_effective?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_session_overridden_keys?: string[]
  blueprint_groups_pack?: import('../lib/slotRegistry').SlotGroupsMap | null
}


export interface SceneLabelEntry {
  id: string
  label: string
}

/** `get_role_info` / `switch_scene` ç­?ä¸? UI å?æ­¥ç??å¿«ç?§ï¼?èº«ä»½å­?æ®µè¯­ä¹?å?{@link RoleData}ã??*/

export interface RoleInfo {
  role_id: string
  role_name: string
  version: string
  author: string
  description: string
  current_favorability: number
  current_emotion: string
  personality_vector: number[]
  /** `evolution.personality_source`ï¼?ç¼ºç?ä¸º vector */
  personality_source?: PersonalitySource
  last_interaction?: string | null
  scenes: string[]
  /** ä¸?scenes é¡ºåºä¸?è?´ï¼?label æ¥è?ªè§?è?²å??scene.json æ??å??ç½®æ? å°?*/
  scene_labels: SceneLabelEntry[]
  current_scene: string | null
  /** ç?¨æ?·å?äº?å?ºæ?¯ï¼?DBï¼?ï¼?ä¸?current_scene å¯ä¸å?*/
  user_presence_scene: string | null
  virtual_time_ms: number
  user_relations: UserRelationDto[]
  default_relation: string
  current_user_relation: string
  /** æ?¯å¦é??ä¸­ã??é»?è®¤èº«ä»½ã?ï¼?ä¸?æ??åº?æ?¾ç¤?`OCLIVE_DEFAULT_RELATION_SENTINEL`ï¼?*/
  use_manifest_default: boolean
  /** å?³ç³»é?¶æ®µï¼?`role_runtime.relation_state`ï¼?*/
  relation_state: string
  remote_life_enabled: boolean
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** manifest â??OLLAMA_MODEL â??å?¨å±?é»?è®¤ */
  effective_ollama_model: string
  /** å?¨å±?å?ä¸?èº«ä»½ vs æ??å?ºæ?¯è¦?ç??ï¼?manifest `identity_binding`ï¼?*/
  identity_binding: 'global' | 'per_scene'
  interaction_mode: 'immersive' | 'pure_chat'
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** å½?å?æ?¥ç¨?æ?¨æ?­ï¼?æ? é?ç½®æ??æ?ªå?½ä¸­æ?¶æ®µæ?¶ä¸º nullï¼?*/
  current_life: LifeStateDto | null
  /** æ¨¡å?å??å­ç³»ç»?å?ç«¯ï¼?ä¸? `load_role` ä¸?è?´ï¼? */
  plugin_backends: PluginBackends
  /** å½?å?ä¼?è¯è¦?ç??ï¼?æ? è¦?ç??æ?¶ä¸º nullï¼?*/
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** å å? ä¼?è¯è¦?ç??å?ç??æ??æ??å?ç«¯ */
  plugin_backends_effective?: PluginBackends
  /** å å? å?ç??å?ç«¯æ¥æºï¼?pack/session/envï¼?*/
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** æ?¯å¦å·²ä»?ç£ç??æ??å»ºä¸?ç??è§?ç?¥è¯?ç´¢å¼?*/
  knowledge_enabled?: boolean
  /** ç?¥è¯?å?æ¡æ?°ï¼?æ?ªå? è½½ç´¢å¼?æ?¶ä¸?0 */
  knowledge_chunk_count?: number
  /** è§?è?²å??`ui.json`ï¼?ä¸»é¢?ã?å¸?å±?ã?æ?æ§½ï¼? */
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

/** `switch_scene` æ?å¹³å??è¿?å??ï¼?RoleInfo å­?æ®µ + å¯é??å?ºæ?¯æ¬¢è¿?è¯­ */

export async function loadRole(roleId: string): Promise<RoleData> {
  return invokeWithFriendlyError<RoleData>('load_role', { roleId })
}

/** è?¥æ??ä»¶å­?å?¨å??è¿?å??ç»å¯¹è·¯å¾?ï¼?å¦å??`null`ï¼?ä¸æ??é??ï¼?ã??*/

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

/** `sessionId` ä¸?å?æ¶?æ¯ç??ä¼?è¯?id ä¸?è?´æ?¶ï¼?è¿?å??è¯¥å?½åç©ºé?´ä¸?ç?? `plugin_backends_*` ç­?å¿«ç?§ã??*/

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

/** ç§»é?¤å½?å?å?ºæ?¯ç??èº«ä»½è¦?ç??ï¼?å?ä¸? `set_user_relation` é?å?å¯æ¢å¤ã??é»?è®¤èº«ä»½ã?å?¨å±?ç?¶æ??*/

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

/** é¢?è§?è§?è?²å??ï¼?`srcPath` å¯ä¸º `.ocpak` / `.zip` æ??å·²è§£å??ç?®å½?ï¼?ä¸? `roles/{id}/` ä¸?è?´ï¼?ã??*/

export async function peekRolePack(srcPath: string): Promise<RolePackPeek> {
  return invokeWithFriendlyError<RolePackPeek>('peek_role_pack_command', {
    src_path: srcPath,
  })
}

/** å¯¼å?¥è§?è?²å??ï¼?å?ä¸?ï¼?æ?¯æ?å??ç¼©å??æ??å·²è§£å??ç?®å½?ã??*/

export async function importRolePack(
  srcPath: string,
  overwrite: boolean,
): Promise<string> {
  return invokeWithFriendlyError<string>('import_role_pack_command', {
    src_path: srcPath,
    overwrite,
  })
}

