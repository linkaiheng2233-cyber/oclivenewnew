import { defineStore } from "pinia";
import {
  expertModelsApplyToSession,
  expertModelsClearRoleDefault,
  expertModelsClearSessionOverride,
  expertModelsGetEffective,
  expertModelsListLocalBaseModels,
  expertModelsListLocalLoras,
  expertModelsSetRoleDefault,
  expertModelsSetSessionOverride,
  getPluginPermissionGrants,
  type ExpertConfigSource,
  type ExpertGraph,
  type LocalModelFileDto,
  type PromptStyleOverride,
} from "../utils/tauri-api";
import { useRoleStore } from "./roleStore";

const emptyGraph = (): ExpertGraph => ({ version: 1, nodes: [], edges: [] });
const LLAMA_LOCAL_PLUGIN_ID = "com.oclive.llama.local";
const REQUIRED_MECH_PERMS = ["process:spawn"];

export const useExpertModelsStore = defineStore("expertModels", {
  state: () => ({
    loading: false,
    error: null as string | null,
    baseModels: [] as LocalModelFileDto[],
    loras: [] as LocalModelFileDto[],
    llamaMissingMechanismPerms: [] as string[],

    effectiveGraph: emptyGraph() as ExpertGraph,
    effectivePromptStyle: null as PromptStyleOverride | null,
    graphSource: "pack_default" as ExpertConfigSource,
    promptStyleSource: "pack_default" as ExpertConfigSource,

    draftGraph: emptyGraph() as ExpertGraph,
    draftPromptStyle: null as PromptStyleOverride | null,
  }),
  actions: {
    async refresh(): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) return;
      this.loading = true;
      this.error = null;
      try {
        const [eff, bases, loras, grants] = await Promise.all([
          expertModelsGetEffective({ roleId, sessionId: null }),
          expertModelsListLocalBaseModels(),
          expertModelsListLocalLoras(),
          getPluginPermissionGrants(LLAMA_LOCAL_PLUGIN_ID).catch(() => ({ grants: [] })),
        ]);
        this.baseModels = bases ?? [];
        this.loras = loras ?? [];
        this.effectiveGraph = eff.graph ?? emptyGraph();
        this.effectivePromptStyle = eff.promptStyle ?? null;
        this.graphSource = eff.graphSource;
        this.promptStyleSource = eff.promptStyleSource;
        const enabled = new Set(
          (grants?.grants ?? [])
            .filter((x) => x?.enabled === true)
            .map((x) => String(x.permission ?? "").trim())
            .filter(Boolean),
        );
        this.llamaMissingMechanismPerms = REQUIRED_MECH_PERMS.filter((p) => !enabled.has(p));

        // Start draft from effective for edit UX.
        this.draftGraph = JSON.parse(JSON.stringify(this.effectiveGraph)) as ExpertGraph;
        this.draftPromptStyle = this.effectivePromptStyle
          ? JSON.parse(JSON.stringify(this.effectivePromptStyle))
          : null;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.loading = false;
      }
    },

    setDraftFromEffective() {
      this.draftGraph = JSON.parse(JSON.stringify(this.effectiveGraph)) as ExpertGraph;
      this.draftPromptStyle = this.effectivePromptStyle
        ? JSON.parse(JSON.stringify(this.effectivePromptStyle))
        : null;
    },

    async applyToSession(): Promise<{ modelPath?: string | null; llamaArgs?: string | null }> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsSetSessionOverride({
        roleId,
        sessionId: null,
        graph: this.draftGraph,
        promptStyle: this.draftPromptStyle,
      });
      const r = await expertModelsApplyToSession({ roleId, sessionId: null });
      await this.refresh();
      return { modelPath: r.modelPath, llamaArgs: r.llamaArgs };
    },

    async clearSessionOverrideAndApply(): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsClearSessionOverride({ roleId, sessionId: null });
      await expertModelsApplyToSession({ roleId, sessionId: null });
      await this.refresh();
    },

    async setRoleDefault(): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsSetRoleDefault({
        roleId,
        graph: this.draftGraph,
        promptStyle: this.draftPromptStyle,
      });
      await this.refresh();
    },

    async clearRoleDefault(): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsClearRoleDefault({ roleId });
      await this.refresh();
    },
  },
});

