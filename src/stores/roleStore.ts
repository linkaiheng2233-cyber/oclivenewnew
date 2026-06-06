import type { AuthorPackFile, LifeStateDto, PackUiConfig, PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap, RoleInfo, UserRelationDto } from '../api'
import { defineStore } from 'pinia'
import {

  clearSceneUserRelation,
  emptyPackUiConfig,
  getRoleInfo,
  setSceneUserRelation as invokeSetSceneUserRelation,
  switchRole as invokeSwitchRole,

  listRoles,
  normalizePackUiConfig,
  OCLIVE_DEFAULT_RELATION_SENTINEL,

  setUserRelation,

  toastAsyncError,
} from '../api'
import { rt } from '../i18n/runtimeT'
import { hostEventBus } from '../lib/hostEventBus'
import {
  normalizeInteractionMode,
  packDefaultFromApi,
} from '../utils/interactionMode'

interface RoleOption { id: string, name: string }

interface RoleInfoState {
  name: string
  version: string
  author: string
  description: string
  favorability: number
  currentEmotion: string
  personality?: number[]
  scenes: string[]
  sceneLabels: Array<{ id: string, label: string }>
  currentScene: string | null
  /** User narrative scene (may differ from currentScene) */
  userPresenceScene: string | null
  virtualTimeMs: number
  userRelations: UserRelationDto[]
  defaultRelation: string
  currentUserRelation: string
  /** Whether "default identity" is selected (manifest `default_relation`) */
  useManifestDefault: boolean
  /** Favorability relation stage; matches backend `RelationState` */
  relationState: string
  eventImpactFactor: number
  /** `evolution.personality_source` */
  personalitySource: 'vector' | 'profile'
  effectiveOllamaModel: string
  /** Matches manifest `identity_binding` */
  identityBinding: 'global' | 'per_scene'
  remoteLifeEnabled: boolean
  remoteLifePackDefault: boolean | null
  /** Current interaction mode (per-role DB) */
  interactionMode: 'immersive' | 'pure_chat'
  /** Suggested default from role pack settings */
  interactionModePackDefault: 'immersive' | 'pure_chat' | null
  /** Virtual-time schedule inference; null when none */
  currentLife: LifeStateDto | null
  /** Matches role pack `settings.json` → `plugin_backends` */
  pluginBackends: PluginBackends
  /** Session-level override (null = no override) */
  pluginBackendsSessionOverride: PluginBackendsOverride | null
  /** Effective backends after session override */
  pluginBackendsEffective: PluginBackends
  /** Source snapshot after session override (pack/session/env) */
  pluginBackendsEffectiveSources: PluginBackendsSourceMap
  /** Whether worldview knowledge index is loaded (`get_role_info`) */
  knowledgeEnabled: boolean
  /** Knowledge chunk count */
  knowledgeChunkCount: number
  /** Normalized role pack `ui.json` snapshot (theme / layout / slots) */
  packUiConfig: PackUiConfig
  /** Optional `author.json` (recommended plugins, suggested backends, etc.) */
  authorPack: AuthorPackFile | null
  slotRegistryPack: import('../lib/slotRegistry').SlotRegistryMap | null
  slotRegistryEffective: import('../lib/slotRegistry').SlotRegistryMap | null
  slotSessionOverriddenKeys: string[]
  blueprintGroupsPack: import('../lib/slotRegistry').SlotGroupsMap | null
  dualCoreEnabled: boolean
  pipelineExperimentalActions: string[]
  replyPostProcessorEnabled: boolean
  replyPostProcessorBackend: string
  replyPostProcessorProfile: string | null
}

function mapRoleInfo(info: RoleInfo): RoleInfoState {
  return {
    name: info.role_name || info.role_id,
    version: info.version ?? '',
    author: info.author ?? '',
    description: info.description ?? '',
    favorability: info.current_favorability,
    currentEmotion: info.current_emotion,
    personality: info.personality_vector ?? [],
    scenes: info.scenes ?? [],
    sceneLabels: info.scene_labels ?? [],
    currentScene: info.current_scene ?? null,
    userPresenceScene: info.user_presence_scene ?? null,
    virtualTimeMs: info.virtual_time_ms ?? 0,
    userRelations: info.user_relations ?? [],
    defaultRelation: info.default_relation ?? 'friend',
    currentUserRelation: info.current_user_relation ?? info.default_relation ?? 'friend',
    useManifestDefault: info.use_manifest_default ?? false,
    relationState: info.relation_state ?? 'Stranger',
    eventImpactFactor: info.event_impact_factor ?? 1,
    personalitySource:
      info.personality_source === 'profile' ? 'profile' : 'vector',
    effectiveOllamaModel: info.effective_ollama_model ?? '',
    identityBinding: info.identity_binding ?? 'per_scene',
    remoteLifeEnabled: info.remote_life_enabled ?? false,
    remoteLifePackDefault: info.remote_life_pack_default ?? null,
    interactionMode: normalizeInteractionMode(info.interaction_mode),
    interactionModePackDefault: packDefaultFromApi(
      info.interaction_mode_pack_default,
    ),
    currentLife: info.current_life ?? null,
    pluginBackends: info.plugin_backends,
    pluginBackendsSessionOverride: info.plugin_backends_session_override ?? null,
    pluginBackendsEffective:
      info.plugin_backends_effective ?? info.plugin_backends,
    pluginBackendsEffectiveSources: info.plugin_backends_effective_sources ?? {
      memory: 'pack_default',
      emotion: 'pack_default',
      event: 'pack_default',
      prompt: 'pack_default',
      llm: 'pack_default',
      agent: 'pack_default',
    },
    knowledgeEnabled: info.knowledge_enabled ?? false,
    knowledgeChunkCount: info.knowledge_chunk_count ?? 0,
    packUiConfig: normalizePackUiConfig(
      info.pack_ui_baseline ?? info.pack_ui_config,
    ),
    authorPack: info.author_pack ?? null,
    slotRegistryPack: info.slot_registry_pack ?? null,
    slotRegistryEffective: info.slot_registry_effective ?? null,
    slotSessionOverriddenKeys: info.slot_session_overridden_keys ?? [],
    blueprintGroupsPack: info.blueprint_groups_pack ?? null,
    dualCoreEnabled: info.dual_core_enabled ?? false,
    pipelineExperimentalActions: info.pipeline_experimental_actions ?? [],
    replyPostProcessorEnabled: info.reply_post_processor_enabled ?? false,
    replyPostProcessorBackend: info.reply_post_processor_backend ?? 'off',
    replyPostProcessorProfile: info.reply_post_processor_profile ?? null,
  }
}

export const useRoleStore = defineStore(
  'role',
  {
    state: () => ({
      currentRoleId: 'mumu',
      roles: [] as RoleOption[],
      roleInfo: {
        name: rt('app.defaultRoleName'),
        version: '',
        author: '',
        description: '',
        favorability: 0,
        currentEmotion: 'neutral',
        personality: [],
        scenes: [] as string[],
        sceneLabels: [] as Array<{ id: string, label: string }>,
        currentScene: null as string | null,
        userPresenceScene: null as string | null,
        virtualTimeMs: 0,
        userRelations: [] as UserRelationDto[],
        defaultRelation: 'friend',
        currentUserRelation: 'friend',
        useManifestDefault: false,
        relationState: 'Stranger',
        eventImpactFactor: 1,
        personalitySource: 'vector',
        effectiveOllamaModel: '',
        identityBinding: 'per_scene',
        remoteLifeEnabled: false,
        remoteLifePackDefault: null,
        interactionMode: 'immersive',
        interactionModePackDefault: null,
        currentLife: null,
        pluginBackends: {
          memory: 'builtin',
          emotion: 'builtin',
          event: 'builtin',
          prompt: 'builtin',
          llm: 'ollama',
          agent: 'builtin',
          directory_plugins: {},
        },
        pluginBackendsSessionOverride: null,
        pluginBackendsEffective: {
          memory: 'builtin',
          emotion: 'builtin',
          event: 'builtin',
          prompt: 'builtin',
          llm: 'ollama',
          agent: 'builtin',
          directory_plugins: {},
        },
        pluginBackendsEffectiveSources: {
          memory: 'pack_default',
          emotion: 'pack_default',
          event: 'pack_default',
          prompt: 'pack_default',
          llm: 'pack_default',
          agent: 'pack_default',
        },
        knowledgeEnabled: false,
        knowledgeChunkCount: 0,
        packUiConfig: emptyPackUiConfig(),
        authorPack: null,
        slotRegistryPack: null,
        slotRegistryEffective: null,
        slotSessionOverriddenKeys: [],
        blueprintGroupsPack: null,
        dualCoreEnabled: false,
        pipelineExperimentalActions: [],
        replyPostProcessorEnabled: false,
        replyPostProcessorBackend: 'off',
        replyPostProcessorProfile: null,
      } as RoleInfoState,
    }),
    actions: {
      async loadRoles() {
        this.roles = await listRoles()
        if (this.roles.length === 0) {
          this.currentRoleId = ''
          return
        }
        if (!this.roles.some(r => r.id === this.currentRoleId)) {
          this.currentRoleId = this.roles[0].id
        }
      },
      async switchRole(roleId: string) {
        const info = await invokeSwitchRole(roleId)
        this.currentRoleId = roleId
        this.applyRoleInfo(info)
      },
      async refreshRoleInfo() {
        try {
          const info = await getRoleInfo(this.currentRoleId)
          this.applyRoleInfo(info)
        }
        catch (err) {
          toastAsyncError(err)
          throw err
        }
      },
      /** Apply already-fetched `RoleInfo` (e.g. from `switch_scene`) to avoid an extra request */
      applyRoleInfo(info: RoleInfo) {
        this.roleInfo = mapRoleInfo(info)
        const rid = (info.role_id ?? this.currentRoleId ?? '').trim()
        if (rid) {
          hostEventBus.emitBuiltin('role:info:updated', { roleId: rid })
        }
      },
      updateLocalAfterMessage(emotion: string, favorabilityCurrent: number) {
        this.roleInfo.currentEmotion = emotion
        this.roleInfo.favorability = favorabilityCurrent
      },
      updateRelationState(relationState: string) {
        this.roleInfo.relationState = relationState
      },
      async setSceneUserRelation(sceneId: string, relation: string) {
        const info = await invokeSetSceneUserRelation(
          this.currentRoleId,
          sceneId,
          relation,
        )
        this.applyRoleInfo(info)
        return info
      },
      /**
       * Select "default identity": follow manifest `default_relation`.
       * When `clearSceneId` is passed, remove identity override for that scene first (top-bar scene identity aligned with global default).
       */
      async setManifestDefaultIdentity(clearSceneId?: string) {
        if (clearSceneId) {
          await clearSceneUserRelation(this.currentRoleId, clearSceneId)
        }
        const info = await setUserRelation(
          this.currentRoleId,
          OCLIVE_DEFAULT_RELATION_SENTINEL,
        )
        this.applyRoleInfo(info)
        return info
      },
    },
    persist: {
      pick: ['currentRoleId'],
    },
    getters: {
      /** Identity dropdown `:value`: sentinel for default identity option, else resolved relation key */
      relationSelectValue(): string {
        return this.roleInfo.useManifestDefault
          ? OCLIVE_DEFAULT_RELATION_SENTINEL
          : this.roleInfo.currentUserRelation
      },
      /** Immersive mode: virtual time, schedule, movement bar, etc. */
      interactionImmersive(): boolean {
        return this.roleInfo.interactionMode === 'immersive'
      },
    },
  },
)
