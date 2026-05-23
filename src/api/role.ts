import { invoke } from '@tauri-apps/api/tauri'
import { invokeWithFriendlyError } from './helpers'

export interface UserRelationDto {
  id: string
  name: string
  prompt_hint: string
  favor_multiplier: number
  /** 角色包配置的初始好感度（0�?00）；切换顶栏身份时会同步到当前好�?*/
  initial_favorability: number
}

/** 虚拟时间 + manifest `life_schedule` 推断的当前活�?*/

export interface LifeStateDto {
  label: string
  activity_key: string
  busy_level: number
  preferred_scene_id: string | null
}

/** �?`plugin_backends.directory_plugins` 一致（snake_case JSON 字段�?*/

export type PersonalitySource = 'vector' | 'profile'

/** 角色包根目录 `ui.json`（与后端 `UiConfig` 对齐；插槽键含点号） */

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
  /** 是否选中「默认身份」（跟随 manifest `default_relation`�?*/
  use_manifest_default: boolean
  /** 异地心声开关（DB�?*/
  remote_life_enabled: boolean
  /** 角色包建议默认是否开启异地心声（settings.json �?remote_presence.default_enabled�?*/
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** `evolution.personality_source`；缺省为 vector */
  personality_source?: PersonalitySource
  /** manifest �?OLLAMA_MODEL �?全局默认 */
  effective_ollama_model: string
  /** 全局单一身份 vs 按场景覆盖（manifest `identity_binding`�?*/
  identity_binding: 'global' | 'per_scene'
  /** 当前交互模式（DB）：`immersive` | `pure_chat` */
  interaction_mode: 'immersive' | 'pure_chat'
  /** 角色�?settings.json 建议默认（可选） */
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** 当前日程推断（无配置或未命中时段时为 null�?*/
  current_life: LifeStateDto | null
  /** 模块化子系统后端（与 `PluginHost` 解析一致） */
  plugin_backends: PluginBackends
  /** 当前会话覆盖（无覆盖时为 null�?*/
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** 叠加会话覆盖后的有效后端 */
  plugin_backends_effective?: PluginBackends
  /** 叠加后的后端来源（pack/session/env�?*/
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** 角色�?`ui.json`（主题、布局、插槽） */
  pack_ui_config: PackUiConfig
  /** `author.suggested_ui` 优先时的有效 UI 基线（与后端 `pack_ui_baseline` 一致） */
  pack_ui_baseline?: PackUiConfig
  /** 可�?`author.json` */
  author_pack?: AuthorPackFile | null
  /** v2 蓝图 `slot_registry`（legacy �?null�?*/
  slot_registry_pack?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_registry_effective?: import('../lib/slotRegistry').SlotRegistryMap | null
  slot_session_overridden_keys?: string[]
  blueprint_groups_pack?: import('../lib/slotRegistry').SlotGroupsMap | null
}


export interface SceneLabelEntry {
  id: string
  label: string
}

/** `get_role_info` / `switch_scene` 等与 UI 同步的快照；身份字段语义�?{@link RoleData}�?*/

export interface RoleInfo {
  role_id: string
  role_name: string
  version: string
  author: string
  description: string
  current_favorability: number
  current_emotion: string
  personality_vector: number[]
  /** `evolution.personality_source`；缺省为 vector */
  personality_source?: PersonalitySource
  last_interaction?: string | null
  scenes: string[]
  /** �?scenes 顺序一致；label 来自角色�?scene.json 或内置映�?*/
  scene_labels: SceneLabelEntry[]
  current_scene: string | null
  /** 用户叙事场景（DB）；�?current_scene 可不�?*/
  user_presence_scene: string | null
  virtual_time_ms: number
  user_relations: UserRelationDto[]
  default_relation: string
  current_user_relation: string
  /** 是否选中「默认身份」（下拉应显�?`OCLIVE_DEFAULT_RELATION_SENTINEL`�?*/
  use_manifest_default: boolean
  /** 关系阶段（`role_runtime.relation_state`�?*/
  relation_state: string
  remote_life_enabled: boolean
  remote_life_pack_default: boolean | null
  event_impact_factor: number
  /** manifest �?OLLAMA_MODEL �?全局默认 */
  effective_ollama_model: string
  /** 全局单一身份 vs 按场景覆盖（manifest `identity_binding`�?*/
  identity_binding: 'global' | 'per_scene'
  interaction_mode: 'immersive' | 'pure_chat'
  interaction_mode_pack_default: 'immersive' | 'pure_chat' | null
  /** 当前日程推断（无配置或未命中时段时为 null�?*/
  current_life: LifeStateDto | null
  /** 模块化子系统后端（与 `load_role` 一致） */
  plugin_backends: PluginBackends
  /** 当前会话覆盖（无覆盖时为 null�?*/
  plugin_backends_session_override?: PluginBackendsOverride | null
  /** 叠加会话覆盖后的有效后端 */
  plugin_backends_effective?: PluginBackends
  /** 叠加后的后端来源（pack/session/env�?*/
  plugin_backends_effective_sources?: PluginBackendsSourceMap
  /** 是否已从磁盘构建世界观知识索�?*/
  knowledge_enabled?: boolean
  /** 知识块条数；未加载索引时�?0 */
  knowledge_chunk_count?: number
  /** 角色�?`ui.json`（主题、布局、插槽） */
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

/** `switch_scene` 扁平化返回：RoleInfo 字段 + 可选场景欢迎语 */

export async function loadRole(roleId: string): Promise<RoleData> {
  return invokeWithFriendlyError<RoleData>('load_role', { roleId })
}

/** 若文件存在则返回绝对路径，否�?`null`（不抛错）�?*/

export async function resolveRoleAssetPath(
  roleId: string,
  relative: string,
): Promise<string | null> {
  return invoke<string | null>('resolve_role_asset_path', {
    roleId,
    relative,
  })
}

/** `sessionId` 与发消息的会�?id 一致时，返回该命名空间下的 `plugin_backends_*` 等快照�?*/

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

/** 移除当前场景的身份覆盖，再与 `set_user_relation` 配合可恢复「默认身份」全局状�?*/

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

/** 预览角色包：`srcPath` 可为 `.ocpak` / `.zip` 或已解压目录（与 `roles/{id}/` 一致）�?*/

export async function peekRolePack(srcPath: string): Promise<RolePackPeek> {
  return invokeWithFriendlyError<RolePackPeek>('peek_role_pack_command', {
    src_path: srcPath,
  })
}

/** 导入角色包：同上，支持压缩包或已解压目录�?*/

export async function importRolePack(
  srcPath: string,
  overwrite: boolean,
): Promise<string> {
  return invokeWithFriendlyError<string>('import_role_pack_command', {
    src_path: srcPath,
    overwrite,
  })
}

