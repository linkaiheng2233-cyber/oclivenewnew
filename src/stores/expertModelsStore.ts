import { defineStore } from "pinia";
import {
  expertModelsApplyToSession,
  expertModelsClearRoleDefault,
  expertModelsClearSessionOverride,
  expertModelsGetEffective,
  expertModelsImportBaseGguf,
  expertModelsImportLoraGguf,
  expertModelsListLocalBaseModels,
  expertModelsListLocalLoras,
  expertModelsListRuns,
  expertModelsClearRuns,
  expertModelsGetRunDetail,
  expertModelsRollbackLastRun,
  expertModelsRollbackToRun,
  expertModelsSetRunPinned,
  expertModelsSetRoleDefault,
  expertModelsSetSessionOverride,
  expertWorkflowsDelete,
  expertWorkflowsGet,
  expertWorkflowsList,
  expertWorkflowsSave,
  getPluginPermissionGrants,
  type ExpertConfigSource,
  type ExpertGraph,
  type ExpertModelsApplyResult,
  type LocalModelFileDto,
  type ExpertWorkflowDto,
  type ExpertWorkflowSummaryDto,
  type ExpertModelsRunSummaryDto,
  type ExpertModelsRunDetailDto,
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
    canRollbackLastRun: false,
    runs: [] as ExpertModelsRunSummaryDto[],

    draftGraph: emptyGraph() as ExpertGraph,
    draftPromptStyle: null as PromptStyleOverride | null,

    workflows: [] as ExpertWorkflowSummaryDto[],
    pickedWorkflowId: "" as string,

    /** `get_effective` 返回的角色级默认（与当前会话是否覆盖无关） */
    roleDefaultGraph: null as ExpertGraph | null,
    roleDefaultPromptStyle: null as PromptStyleOverride | null,
    /** 用于未保存检测：与 draft 序列化快照比较 */
    workbenchDraftBaseline: "" as string,
  }),
  getters: {
    workbenchDraftDirty(state): boolean {
      const snap = JSON.stringify({
        graph: state.draftGraph,
        ps: state.draftPromptStyle,
      });
      return state.workbenchDraftBaseline !== "" && snap !== state.workbenchDraftBaseline;
    },
  },
  actions: {
    async refresh(): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) return;
      this.loading = true;
      this.error = null;
      try {
        const [eff, bases, loras, runs, grants] = await Promise.all([
          expertModelsGetEffective({ roleId, sessionId: null }),
          expertModelsListLocalBaseModels(),
          expertModelsListLocalLoras(),
          expertModelsListRuns({ roleId, sessionId: null }).catch(() => ({ items: [] })),
          getPluginPermissionGrants(LLAMA_LOCAL_PLUGIN_ID).catch(() => ({ grants: [] })),
        ]);
        this.baseModels = bases ?? [];
        this.loras = loras ?? [];
        this.effectiveGraph = eff.graph ?? emptyGraph();
        this.effectivePromptStyle = eff.promptStyle ?? null;
        this.graphSource = eff.graphSource;
        this.promptStyleSource = eff.promptStyleSource;
        this.roleDefaultGraph =
          eff.roleDefaultGraph && (eff.roleDefaultGraph.nodes?.length ?? 0) > 0
            ? (JSON.parse(JSON.stringify(eff.roleDefaultGraph)) as ExpertGraph)
            : null;
        this.roleDefaultPromptStyle = eff.roleDefaultPromptStyle
          ? (JSON.parse(JSON.stringify(eff.roleDefaultPromptStyle)) as PromptStyleOverride)
          : null;
        this.canRollbackLastRun = eff.canRollbackLastRun === true;
        this.runs = (Array.isArray((runs as any)?.items) ? (runs as any).items : []) as ExpertModelsRunSummaryDto[];
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
        this.syncWorkbenchBaseline();
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.loading = false;
      }
    },

    syncWorkbenchBaseline(): void {
      this.workbenchDraftBaseline = JSON.stringify({
        graph: this.draftGraph,
        ps: this.draftPromptStyle,
      });
    },

    /**
     * 打开专家工作台后按导航意图同步草稿：`effective` = 当前生效图；`role_default` = 角色级默认（若有）。
     */
    applyWorkbenchNavigationDraft(mode: "effective" | "role_default"): void {
      if (
        mode === "role_default" &&
        this.roleDefaultGraph &&
        (this.roleDefaultGraph.nodes?.length ?? 0) > 0
      ) {
        this.draftGraph = JSON.parse(JSON.stringify(this.roleDefaultGraph)) as ExpertGraph;
        this.draftPromptStyle = this.roleDefaultPromptStyle
          ? (JSON.parse(JSON.stringify(this.roleDefaultPromptStyle)) as PromptStyleOverride)
          : null;
      }
      // `effective`：`refresh()` 已将草稿对齐为当前生效图，此处只落基线。
      this.syncWorkbenchBaseline();
    },

    async refreshWorkflows(): Promise<void> {
      const res = await expertWorkflowsList();
      this.workflows = res.items ?? [];
      if (this.pickedWorkflowId && !this.workflows.some((w) => w.id === this.pickedWorkflowId)) {
        this.pickedWorkflowId = "";
      }
    },

    async loadWorkflow(id: string): Promise<ExpertWorkflowDto> {
      const wid = (id ?? "").trim();
      if (!wid) throw new Error("请选择一个工作流。");
      const wf = await expertWorkflowsGet(wid);
      this.pickedWorkflowId = wf.id;
      this.draftGraph = JSON.parse(JSON.stringify(wf.graph ?? emptyGraph())) as ExpertGraph;
      this.draftPromptStyle = wf.promptStyle ? { ...(wf.promptStyle as any) } : null;
      this.syncWorkbenchBaseline();
      return wf;
    },

    async saveWorkflow(name: string, overwriteId?: string | null): Promise<ExpertWorkflowDto> {
      const n = (name ?? "").trim();
      if (!n) throw new Error("工作流名称不能为空。");
      const wf = await expertWorkflowsSave({
        id: overwriteId ?? null,
        name: n,
        graph: this.draftGraph,
        promptStyle: this.draftPromptStyle,
      });
      await this.refreshWorkflows();
      this.pickedWorkflowId = wf.id;
      this.syncWorkbenchBaseline();
      return wf;
    },

    async saveWorkflowFromConfig(
      name: string,
      graph: ExpertGraph,
      promptStyle: PromptStyleOverride | null,
      overwriteId?: string | null,
    ): Promise<ExpertWorkflowDto> {
      const n = (name ?? "").trim();
      if (!n) throw new Error("工作流名称不能为空。");
      const wf = await expertWorkflowsSave({
        id: overwriteId ?? null,
        name: n,
        graph,
        promptStyle: promptStyle ?? null,
      });
      await this.refreshWorkflows();
      this.pickedWorkflowId = wf.id;
      this.syncWorkbenchBaseline();
      return wf;
    },

    async deleteWorkflow(id: string): Promise<void> {
      const wid = (id ?? "").trim();
      if (!wid) return;
      await expertWorkflowsDelete(wid);
      await this.refreshWorkflows();
      if (this.pickedWorkflowId === wid) this.pickedWorkflowId = "";
    },

    setDraftFromEffective() {
      this.draftGraph = JSON.parse(JSON.stringify(this.effectiveGraph)) as ExpertGraph;
      this.draftPromptStyle = this.effectivePromptStyle
        ? JSON.parse(JSON.stringify(this.effectivePromptStyle))
        : null;
      this.syncWorkbenchBaseline();
    },

    /** 将草稿切换为角色级默认图（若存在）；用于工作台空状态快捷入口。 */
    loadRoleDefaultIntoDraft() {
      if (this.roleDefaultGraph && (this.roleDefaultGraph.nodes?.length ?? 0) > 0) {
        this.draftGraph = JSON.parse(JSON.stringify(this.roleDefaultGraph)) as ExpertGraph;
        this.draftPromptStyle = this.roleDefaultPromptStyle
          ? (JSON.parse(JSON.stringify(this.roleDefaultPromptStyle)) as PromptStyleOverride)
          : null;
        this.syncWorkbenchBaseline();
      }
    },

    async applyToSession(): Promise<ExpertModelsApplyResult> {
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
      return r;
    },

    async rollbackLastRun(): Promise<ExpertModelsApplyResult> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      const r = await expertModelsRollbackLastRun({ roleId, sessionId: null });
      await this.refresh();
      return r;
    },

    async rollbackToRun(indexFromLatest: number): Promise<ExpertModelsApplyResult> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      const r = await expertModelsRollbackToRun({ roleId, sessionId: null, indexFromLatest });
      await this.refresh();
      return r;
    },

    async clearRuns(): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsClearRuns({ roleId, sessionId: null });
      await this.refresh();
    },

    async clearRunsWithMode(mode: "all" | "ok" | "failed" | "unpinned", keepPinned: boolean): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsClearRuns({ roleId, sessionId: null, mode, keepPinned });
      await this.refresh();
    },

    async setRunPinned(indexFromLatest: number, pinned: boolean): Promise<void> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsSetRunPinned({ roleId, sessionId: null, indexFromLatest, pinned });
      await this.refresh();
    },

    async getRunDetail(indexFromLatest: number): Promise<ExpertModelsRunDetailDto> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      const res = await expertModelsGetRunDetail({ roleId, sessionId: null, indexFromLatest });
      return res.item;
    },

    async applySpecificToSession(
      graph: ExpertGraph,
      promptStyle: PromptStyleOverride | null,
    ): Promise<ExpertModelsApplyResult> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsSetSessionOverride({
        roleId,
        sessionId: null,
        graph,
        promptStyle,
      });
      const r = await expertModelsApplyToSession({ roleId, sessionId: null });
      await this.refresh();
      return r;
    },

    async clearSessionOverrideAndApply(): Promise<ExpertModelsApplyResult> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsClearSessionOverride({ roleId, sessionId: null });
      const r = await expertModelsApplyToSession({ roleId, sessionId: null });
      await this.refresh();
      return r;
    },

    /** 清除会话覆盖与角色级专家默认，并重新应用，使生效图回到角色包内置。 */
    async resetExpertGraphToPackDefaultFully(): Promise<ExpertModelsApplyResult> {
      const roleStore = useRoleStore();
      const roleId = (roleStore.currentRoleId ?? "").trim();
      if (!roleId) throw new Error("当前未选择角色。");
      await expertModelsClearSessionOverride({ roleId, sessionId: null });
      await expertModelsClearRoleDefault({ roleId });
      const r = await expertModelsApplyToSession({ roleId, sessionId: null });
      await this.refresh();
      return r;
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

    async importBaseGguf(sourcePath: string): Promise<void> {
      const p = (sourcePath ?? "").trim();
      if (!p) return;
      await expertModelsImportBaseGguf(p);
      await this.refresh();
    },

    async importLoraGguf(sourcePath: string): Promise<void> {
      const p = (sourcePath ?? "").trim();
      if (!p) return;
      await expertModelsImportLoraGguf(p);
      await this.refresh();
    },
  },
});

