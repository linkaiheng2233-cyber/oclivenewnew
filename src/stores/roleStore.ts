import { defineStore } from "pinia";
import {
  clearSceneUserRelation,
  getRoleInfo,
  listRoles,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setSceneUserRelation as invokeSetSceneUserRelation,
  setUserRelation,
  switchRole as invokeSwitchRole,
  type LifeStateDto,
  type PluginBackends,
  type RoleInfo,
  type UserRelationDto,
} from "../utils/tauri-api";
import {
  normalizeInteractionMode,
  packDefaultFromApi,
} from "../utils/interactionMode";

type RoleOption = { id: string; name: string };

type RoleInfoState = {
  name: string;
  version: string;
  author: string;
  description: string;
  favorability: number;
  currentEmotion: string;
  personality?: number[];
  scenes: string[];
  sceneLabels: Array<{ id: string; label: string }>;
  currentScene: string | null;
  /** 用户叙事场景（与 currentScene 可不同） */
  userPresenceScene: string | null;
  virtualTimeMs: number;
  userRelations: UserRelationDto[];
  defaultRelation: string;
  currentUserRelation: string;
  /** 是否选中「默认身份」（manifest `default_relation`） */
  useManifestDefault: boolean;
  /** 好感度关系阶段，与后端 `RelationState` 一致 */
  relationState: string;
  eventImpactFactor: number;
  effectiveOllamaModel: string;
  /** 与 manifest `identity_binding` 一致 */
  identityBinding: "global" | "per_scene";
  remoteLifeEnabled: boolean;
  remoteLifePackDefault: boolean | null;
  /** 当前交互模式（每角色 DB） */
  interactionMode: "immersive" | "pure_chat";
  /** 角色包 settings 建议默认 */
  interactionModePackDefault: "immersive" | "pure_chat" | null;
  /** 虚拟时间日程推断，无则为 null */
  currentLife: LifeStateDto | null;
  /** 与角色包 `settings.json` → `plugin_backends` 一致 */
  pluginBackends: PluginBackends;
  /** 是否已加载世界观知识索引（`get_role_info`） */
  knowledgeEnabled: boolean;
  /** 知识块条数 */
  knowledgeChunkCount: number;
};

function mapRoleInfo(info: RoleInfo): RoleInfoState {
  return {
    name: info.role_name || info.role_id,
    version: info.version ?? "",
    author: info.author ?? "",
    description: info.description ?? "",
    favorability: info.current_favorability,
    currentEmotion: info.current_emotion,
    personality: info.personality_vector ?? [],
    scenes: info.scenes ?? [],
    sceneLabels: info.scene_labels ?? [],
    currentScene: info.current_scene ?? null,
    userPresenceScene: info.user_presence_scene ?? null,
    virtualTimeMs: info.virtual_time_ms ?? 0,
    userRelations: info.user_relations ?? [],
    defaultRelation: info.default_relation ?? "friend",
    currentUserRelation: info.current_user_relation ?? info.default_relation ?? "friend",
    useManifestDefault: info.use_manifest_default ?? false,
    relationState: info.relation_state ?? "Stranger",
    eventImpactFactor: info.event_impact_factor ?? 1,
    effectiveOllamaModel: info.effective_ollama_model ?? "",
    identityBinding: info.identity_binding ?? "per_scene",
    remoteLifeEnabled: info.remote_life_enabled ?? false,
    remoteLifePackDefault: info.remote_life_pack_default ?? null,
    interactionMode: normalizeInteractionMode(info.interaction_mode),
    interactionModePackDefault: packDefaultFromApi(
      info.interaction_mode_pack_default,
    ),
    currentLife: info.current_life ?? null,
    pluginBackends: info.plugin_backends,
    knowledgeEnabled: info.knowledge_enabled ?? false,
    knowledgeChunkCount: info.knowledge_chunk_count ?? 0,
  };
}

export const useRoleStore = defineStore(
  "role",
  {
    state: () => ({
      currentRoleId: "mumu",
      roles: [] as RoleOption[],
      roleInfo: {
        name: "沐沐",
        version: "",
        author: "",
        description: "",
        favorability: 0,
        currentEmotion: "neutral",
        personality: [],
        scenes: [] as string[],
        sceneLabels: [] as Array<{ id: string; label: string }>,
        currentScene: null as string | null,
        userPresenceScene: null as string | null,
        virtualTimeMs: 0,
        userRelations: [] as UserRelationDto[],
        defaultRelation: "friend",
        currentUserRelation: "friend",
        useManifestDefault: false,
        relationState: "Stranger",
        eventImpactFactor: 1,
        effectiveOllamaModel: "",
        identityBinding: "per_scene",
        remoteLifeEnabled: false,
        remoteLifePackDefault: null,
        interactionMode: "immersive",
        interactionModePackDefault: null,
        currentLife: null,
        pluginBackends: {
          memory: "builtin",
          emotion: "builtin",
          event: "builtin",
          prompt: "builtin",
          llm: "ollama",
        },
        knowledgeEnabled: false,
        knowledgeChunkCount: 0,
      } as RoleInfoState,
    }),
    actions: {
      async loadRoles() {
        this.roles = await listRoles();
        if (!this.roles.find((r) => r.id === this.currentRoleId) && this.roles.length > 0) {
          this.currentRoleId = this.roles[0].id;
        }
      },
      async switchRole(roleId: string) {
        const info = await invokeSwitchRole(roleId);
        this.currentRoleId = roleId;
        this.roleInfo = mapRoleInfo(info);
      },
      async refreshRoleInfo() {
        const info = await getRoleInfo(this.currentRoleId);
        this.roleInfo = mapRoleInfo(info);
      },
      /** 使用已拿到的 `RoleInfo`（如 `switch_scene`）避免多余请求 */
      applyRoleInfo(info: RoleInfo) {
        this.roleInfo = mapRoleInfo(info);
      },
      updateLocalAfterMessage(emotion: string, favorabilityCurrent: number) {
        this.roleInfo.currentEmotion = emotion;
        this.roleInfo.favorability = favorabilityCurrent;
      },
      updateRelationState(relationState: string) {
        this.roleInfo.relationState = relationState;
      },
      async setSceneUserRelation(sceneId: string, relation: string) {
        const info = await invokeSetSceneUserRelation(
          this.currentRoleId,
          sceneId,
          relation,
        );
        this.applyRoleInfo(info);
        return info;
      },
      /**
       * 选择「默认身份」：跟随 manifest `default_relation`。
       * 若传入 `clearSceneId`，先移除该场景的身份覆盖（顶栏场景身份与全局默认一致时使用）。
       */
      async setManifestDefaultIdentity(clearSceneId?: string) {
        if (clearSceneId) {
          await clearSceneUserRelation(this.currentRoleId, clearSceneId);
        }
        const info = await setUserRelation(
          this.currentRoleId,
          OCLIVE_DEFAULT_RELATION_SENTINEL,
        );
        this.applyRoleInfo(info);
        return info;
      },
    },
    persist: true,
    getters: {
      /** 身份下拉框 `:value`：默认身份选项用哨兵，否则用当前解析后的关系键 */
      relationSelectValue(): string {
        return this.roleInfo.useManifestDefault
          ? OCLIVE_DEFAULT_RELATION_SENTINEL
          : this.roleInfo.currentUserRelation;
      },
      /** 沉浸模式：虚拟时间、日程、位移条等 */
      interactionImmersive(): boolean {
        return this.roleInfo.interactionMode === "immersive";
      },
    },
  },
);
