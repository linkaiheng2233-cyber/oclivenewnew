import type { AuthorPackFile, LifeStateDto, PackUiConfig, PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap, RoleInfo, UserRelationDto } from '@oclive/shared/api'
import { normalizePluginBackends } from '@oclive/shared/api/settings'
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
} from '@oclive/shared/api'
import { rt } from '@oclive/shared/i18n/runtimeT'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { normalizeSlotBackendWire, type SlotRegistryMap } from '@oclive/shared/lib/slotRegistry'
import {
  normalizeInteractionMode,
  packDefaultFromApi,
} from '@oclive/shared/utils/interactionMode'
import {
  resolveDefaultRoleId,
  shouldShowPresetPicker,
  type PresetRoleOption,
} from '@oclive/shared/utils/presetRolePicker'

export interface RoleOption extends PresetRoleOption {
  id: string
  name: string
}

interface RoleInfoState {
  name: string
  version: string
  author: string
  description: string
  favorability: number
  currentEmotion: string
  /** Latest catalog visual_state_id from send_message (optional). */
  visualStateId?: string | null
  /** Resolved portrait asset path from performance_directive or catalog. */
  portraitAssetPath?: string | null
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
  slotRegistryPack: import('@oclive/shared/lib/slotRegistry').SlotRegistryMap | null
  slotRegistryEffective: import('@oclive/shared/lib/slotRegistry').SlotRegistryMap | null
  slotSessionOverriddenKeys: string[]
  blueprintGroupsPack: import('@oclive/shared/lib/slotRegistry').SlotGroupsMap | null
  dualCoreEnabled: boolean
  pipelineExperimentalActions: string[]
  replyPostProcessorEnabled: boolean
  replyPostProcessorBackend: string
  replyPostProcessorProfile: string | null
}

function normalizeSlotRegistryBackends(
  registry: SlotRegistryMap | null | undefined,
): SlotRegistryMap | null {
  if (!registry)
    return null
  const next: SlotRegistryMap = {}
  for (const [key, entry] of Object.entries(registry)) {
    next[key] = {
      ...entry,
      backend: normalizeSlotBackendWire(entry.backend),
    }
  }
  return next
}

function mapRoleInfo(info: RoleInfo): RoleInfoState {
  const metrics = info.display_metrics
  return {
    name: info.role_name || info.role_id,
    version: info.version ?? '',
    author: info.author ?? '',
    description: info.description ?? '',
    favorability: metrics?.favor ?? info.current_favorability,
    currentEmotion: info.current_emotion,
    personality: metrics?.traits ?? info.personality_vector ?? [],
    scenes: info.scenes ?? [],
    sceneLabels: info.scene_labels ?? [],
    currentScene: info.current_scene ?? null,
    userPresenceScene: info.user_presence_scene ?? null,
    virtualTimeMs: info.virtual_time_ms ?? 0,
    userRelations: info.user_relations ?? [],
    defaultRelation: info.default_relation ?? 'friend',
    currentUserRelation: info.current_user_relation ?? info.default_relation ?? 'friend',
    useManifestDefault: info.use_manifest_default ?? false,
    relationState: metrics?.relation_summary ?? info.relation_state ?? 'Stranger',
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
    pluginBackends: normalizePluginBackends(info.plugin_backends),
    pluginBackendsSessionOverride: info.plugin_backends_session_override
      ? normalizePluginBackends(info.plugin_backends_session_override)
      : null,
    pluginBackendsEffective: normalizePluginBackends(
      info.plugin_backends_effective ?? info.plugin_backends,
    ),
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
    slotRegistryPack: normalizeSlotRegistryBackends(info.slot_registry_pack),
    slotRegistryEffective: normalizeSlotRegistryBackends(info.slot_registry_effective),
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
      currentRoleId: '',
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
        interactionMode: 'pure_chat',
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
        const rows = await listRoles()
        this.roles = rows.map(r => ({
          id: r.id,
          name: r.name,
          description: r.description ?? '',
          featured: r.featured ?? false,
          preset_order: r.preset_order ?? 999,
          interaction_mode_suggestion: r.interaction_mode_suggestion ?? null,
        }))
        if (this.roles.length === 0) {
          this.currentRoleId = ''
          return
        }
        if (shouldShowPresetPicker(this.roles, this.currentRoleId))
          return
        if (!this.roles.some(r => r.id === this.currentRoleId)) {
          this.currentRoleId = resolveDefaultRoleId(this.roles)
        }
      },
      needsPresetPicker(): boolean {
        return shouldShowPresetPicker(this.roles, this.currentRoleId)
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
      updateLocalAfterMessage(
        emotion: string,
        favorabilityCurrent: number,
        visual?: {
          visualStateId?: string | null
          portraitAssetPath?: string | null
        },
      ) {
        this.roleInfo.currentEmotion = emotion
        this.roleInfo.favorability = favorabilityCurrent
        if (visual?.visualStateId !== undefined)
          this.roleInfo.visualStateId = visual.visualStateId
        if (visual?.portraitAssetPath !== undefined)
          this.roleInfo.portraitAssetPath = visual.portraitAssetPath
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
       * Select manifest default relation (top-bar relation sentinel).
       * When `clearSceneId` is passed, remove per-scene relation override first.
       */
      async setManifestDefaultRelation(clearSceneId?: string) {
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
