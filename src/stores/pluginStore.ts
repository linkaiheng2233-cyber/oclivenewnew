import { defineStore } from "pinia";
import { setHostEventSubscribedEvents } from "../lib/hostEventBus";
import {
  batchUpdatePlugins,
  checkPluginUpdates,
  extractPluginZip,
  getCachedPluginIndex,
  getDirectoryPluginBootstrap,
  getDirectoryPluginCatalog,
  getPluginState,
  installPluginFromMarket,
  resetPluginStateToRoleDefault,
  saveGlobalPluginState,
  savePluginState,
  syncPluginIndexCommand,
  uninstallPluginFromMarket,
  updatePluginFromMarket,
  type DirectoryPluginBootstrap,
  type DirectoryPluginCatalogEntry,
  type PluginMarketSnapshotDto,
  type PluginUpdateInfo,
  type PluginUiSlotInfo,
  type RolePluginState,
  type UiSlotVariantInfo,
} from "../utils/tauri-api";
import { useRoleStore } from "./roleStore";

type SlotOrderMemo = {
  signature: string;
  value: string[];
};

const slotOrderMemo = new Map<string, SlotOrderMemo>();

/** 并发 `refresh()` 合并为单次执行（共享 Promise）。 */
let refreshPromise: Promise<void> | null = null;

/** 聊天输入区上方工具栏 */
export const SLOT_CHAT_TOOLBAR = "chat_toolbar";
/** 应用内「设置 → 插件扩展」页嵌入 */
export const SLOT_SETTINGS_PANEL = "settings.panel";
/** 左侧角色详情区底部（立绘与名称下方） */
export const SLOT_ROLE_DETAIL = "role.detail";
/** 左侧栏角色块下方（好感度条上方），整列侧栏扩展区 */
export const SLOT_SIDEBAR = "sidebar";
/** 右侧聊天列顶部（消息列表上方） */
export const SLOT_CHAT_HEADER = "chat.header";
/** 插件管理面板内嵌 */
export const SLOT_SETTINGS_PLUGINS = "settings.plugins";
/** 设置对话框 · 扩展区（常规） */
export const SLOT_SETTINGS_ADVANCED = "settings.advanced";
/** 全局浮层模板区 */
export const SLOT_OVERLAY_FLOATING = "overlay.floating";
/** 快捷键说明 / 启动器聚合 */
export const SLOT_LAUNCHER_PALETTE = "launcher.palette";
/** 调试面板扩展 */
export const SLOT_DEBUG_DOCK = "debug.dock";

/** 与后端 `EMBEDDED_UI_SLOT_NAMES` 顺序一致（用于遍历）。 */
export const ALL_EMBEDDED_SLOT_NAMES: readonly string[] = [
  SLOT_CHAT_TOOLBAR,
  SLOT_SETTINGS_PANEL,
  SLOT_ROLE_DETAIL,
  SLOT_SIDEBAR,
  SLOT_CHAT_HEADER,
  SLOT_SETTINGS_PLUGINS,
  SLOT_SETTINGS_ADVANCED,
  SLOT_OVERLAY_FLOATING,
  SLOT_LAUNCHER_PALETTE,
  SLOT_DEBUG_DOCK,
];

function emptyState(): RolePluginState {
  return {
    shellPluginId: "",
    disabled_plugins: [],
    slot_order: {},
    disabled_slot_contributions: {},
    slot_appearance: {},
    force_iframe_mode: false,
  };
}

function arraysEqual(a: string[] = [], b: string[] = []): boolean {
  if (a === b) return true;
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

function recordOfArraysEqual(
  a: Record<string, string[]> = {},
  b: Record<string, string[]> = {},
): boolean {
  const ka = Object.keys(a);
  const kb = Object.keys(b);
  if (ka.length !== kb.length) return false;
  for (const k of ka) {
    if (!(k in b)) return false;
    if (!arraysEqual(a[k], b[k])) return false;
  }
  return true;
}

function uiSlotVariantsEqual(
  a: UiSlotVariantInfo[] | undefined,
  b: UiSlotVariantInfo[] | undefined,
): boolean {
  const x = a ?? [];
  const y = b ?? [];
  if (x.length !== y.length) return false;
  for (let i = 0; i < x.length; i += 1) {
    const p = x[i];
    const q = y[i];
    if (
      p.slot !== q.slot ||
      p.appearanceId !== q.appearanceId ||
      (p.label ?? "") !== (q.label ?? "")
    ) {
      return false;
    }
  }
  return true;
}

function catalogEqual(
  a: DirectoryPluginCatalogEntry[],
  b: DirectoryPluginCatalogEntry[],
): boolean {
  if (a === b) return true;
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    const x = a[i];
    const y = b[i];
    if (
      x.id !== y.id ||
      x.version !== y.version ||
      (x.pluginType ?? null) !== (y.pluginType ?? null) ||
      (x.hasUiSettings ?? false) !== (y.hasUiSettings ?? false) ||
      x.hasRpcProcess !== y.hasRpcProcess ||
      (x.declaresRpcMethods ?? false) !== (y.declaresRpcMethods ?? false) ||
      x.isShell !== y.isShell ||
      (x.dependencyStatus ?? "ok") !== (y.dependencyStatus ?? "ok") ||
      !arraysEqual(x.uiSlotNames ?? [], y.uiSlotNames ?? []) ||
      !uiSlotVariantsEqual(x.uiSlotVariants, y.uiSlotVariants) ||
      !arraysEqual(x.provides ?? [], y.provides ?? []) ||
      !arraysEqual(x.dependencyIssues ?? [], y.dependencyIssues ?? [])
    ) {
      return false;
    }
  }
  return true;
}

function slotAppearanceEqual(
  a: Record<string, Record<string, string>> | undefined,
  b: Record<string, Record<string, string>> | undefined,
): boolean {
  const aa = a ?? {};
  const bb = b ?? {};
  const keysA = Object.keys(aa).sort();
  const keysB = Object.keys(bb).sort();
  if (keysA.length !== keysB.length) return false;
  for (let i = 0; i < keysA.length; i += 1) {
    if (keysA[i] !== keysB[i]) return false;
    const pid = keysA[i]!;
    const ia = aa[pid]!;
    const ib = bb[pid]!;
    const skA = Object.keys(ia).sort();
    const skB = Object.keys(ib).sort();
    if (skA.length !== skB.length) return false;
    for (let j = 0; j < skA.length; j += 1) {
      if (skA[j] !== skB[j]) return false;
      const slot = skA[j]!;
      if (ia[slot] !== ib[slot]) return false;
    }
  }
  return true;
}

function rolePluginStateEqual(a: RolePluginState, b: RolePluginState): boolean {
  return (
    a.shellPluginId === b.shellPluginId &&
    (a.force_iframe_mode ?? false) === (b.force_iframe_mode ?? false) &&
    arraysEqual(a.disabled_plugins ?? [], b.disabled_plugins ?? []) &&
    recordOfArraysEqual(a.slot_order ?? {}, b.slot_order ?? {}) &&
    recordOfArraysEqual(
      a.disabled_slot_contributions ?? {},
      b.disabled_slot_contributions ?? {},
    ) &&
    slotAppearanceEqual(a.slot_appearance, b.slot_appearance)
  );
}

function buildSlotOrderSignature(candidates: string[], order: string[]): string {
  return `${candidates.join("\u001f")}\u001e${order.join("\u001f")}`;
}

export type PluginPersistScope = "role" | "global";

function clonePluginState(s: RolePluginState): RolePluginState {
  const sa = s.slot_appearance ?? {};
  const slot_appearance: Record<string, Record<string, string>> = {};
  for (const pid of Object.keys(sa)) {
    slot_appearance[pid] = { ...sa[pid] };
  }
  return {
    shellPluginId: s.shellPluginId ?? "",
    disabled_plugins: [...(s.disabled_plugins ?? [])],
    slot_order: { ...s.slot_order },
    disabled_slot_contributions: { ...s.disabled_slot_contributions },
    slot_appearance,
    force_iframe_mode: s.force_iframe_mode ?? false,
  };
}

export type PluginPanelMainTab = "plugins" | "backends" | "slots";

/** 专业模式顶层：已装插件/后端/插槽 vs 社区索引市场 */
export type PluginPanelRootTab = "manage" | "market";

export type OpenPluginPanelArg =
  | PluginPanelMainTab
  | { tab?: PluginPanelMainTab; root?: PluginPanelRootTab };

export const usePluginStore = defineStore("plugin", {
  state: () => ({
    panelVisible: false,
    /** 插件管理（界面插件 / 后端 / 插槽）与插件市场（社区索引）分栏 */
    panelRootTab: "manage" as PluginPanelRootTab,
    panelMainTab: "plugins" as PluginPanelMainTab,
    loading: false,
    error: null as string | null,
    catalog: [] as DirectoryPluginCatalogEntry[],
    /** 目录插件 catalog 预计算：各 slot 对应的非整壳插件 id（已排序）。 */
    catalogCandidatesBySlot: {} as Record<string, string[]>,
    pluginState: emptyState() as RolePluginState,
    /** 最近一次 `get_plugin_state` 的按角色 / 全局原始行（切换「保存到」时回填）。 */
    pluginStateBundle: null as {
      role: RolePluginState;
      globalDefaults: RolePluginState;
    } | null,
    /** 保存目标：`role` 写入当前角色；`global` 写入跨角色默认。 */
    persistScope: "role" as PluginPersistScope,
    /** 与 `get_directory_plugin_bootstrap.developer_mode` 一致（扫描额外插件根等）。 */
    developerMode: false,
    /** 最近一次 bootstrap 的嵌入插槽列表（与 `get_directory_plugin_bootstrap.uiSlots` 一致）。 */
    bootstrapUiSlots: [] as PluginUiSlotInfo[],
    /** 变更后嵌入插槽组件会重新拉 bootstrap */
    bootstrapEpoch: 0,
    /** `check_plugin_updates` 最近一次结果（按插件 id）。 */
    pluginUpdateById: {} as Record<string, PluginUpdateInfo>,
    pluginUpdatesCheckLoading: false,
    extractingPluginId: null as string | null,
    /** 最近一次 `get_cached_plugin_index` / `sync_plugin_index_command` 快照 */
    pluginMarketSnapshot: null as PluginMarketSnapshotDto | null,
    pluginMarketSyncing: false,
    pluginMarketError: null as string | null,
  }),
  actions: {
    /** 由 bootstrap DTO 更新宿主事件订阅与开发者模式（插槽与 `refresh` / `sync` 共用）。 */
    applyDirectoryBootstrap(boot: DirectoryPluginBootstrap) {
      setHostEventSubscribedEvents(boot.subscribedHostEvents ?? []);
      this.developerMode = boot.developerMode ?? false;
      this.bootstrapUiSlots = boot.uiSlots ?? [];
    },
    /** 角色切换或插件启用状态变更后更新宿主事件订阅与开发者模式（不拉 catalog）。 */
    async syncDirectoryPluginBootstrap() {
      const roleId = useRoleStore().currentRoleId;
      try {
        const boot = await getDirectoryPluginBootstrap(roleId);
        this.applyDirectoryBootstrap(boot);
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      }
    },
    async openPanel(arg?: OpenPluginPanelArg) {
      if (arg === undefined) {
        this.panelVisible = true;
        await this.refresh();
        return;
      }
      if (typeof arg === "string") {
        this.panelMainTab = arg;
        this.panelRootTab = "manage";
      } else {
        if (arg.tab !== undefined) {
          this.panelMainTab = arg.tab;
        }
        if (arg.root !== undefined) {
          this.panelRootTab = arg.root;
        }
      }
      this.panelVisible = true;
      await this.refresh();
    },
    async loadCachedPluginMarket() {
      this.pluginMarketError = null;
      try {
        this.pluginMarketSnapshot = await getCachedPluginIndex();
      } catch (e) {
        this.pluginMarketError = e instanceof Error ? e.message : String(e);
      }
    },
    async syncPluginMarket(indexUrl?: string | null) {
      this.pluginMarketSyncing = true;
      this.pluginMarketError = null;
      try {
        this.pluginMarketSnapshot = await syncPluginIndexCommand(
          indexUrl ?? undefined,
        );
        await this.refresh();
      } catch (e) {
        this.pluginMarketError = e instanceof Error ? e.message : String(e);
        throw e;
      } finally {
        this.pluginMarketSyncing = false;
      }
    },
    async installFromPluginMarket(pluginId: string, gitUrl?: string | null) {
      await installPluginFromMarket(pluginId, gitUrl ?? null);
      await this.refresh();
      this.bootstrapEpoch += 1;
    },
    async updateInstalledPluginFromGit(pluginId: string) {
      await updatePluginFromMarket(pluginId);
      await this.refresh();
      this.bootstrapEpoch += 1;
    },
    async uninstallPluginFromGitIndex(pluginId: string) {
      await uninstallPluginFromMarket(pluginId);
      await this.refresh();
      this.bootstrapEpoch += 1;
    },
    async batchUpdatePluginsFromGitIndex(pluginIds: string[]) {
      await batchUpdatePlugins(pluginIds);
      await this.refresh();
      this.bootstrapEpoch += 1;
    },
    async checkPluginUpdatesFromRegistry() {
      this.pluginUpdatesCheckLoading = true;
      this.error = null;
      try {
        const ids = this.catalog.map((c) => c.id);
        this.pluginUpdateById = await checkPluginUpdates(ids);
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.pluginUpdatesCheckLoading = false;
      }
    },
    async installPluginFromLocalZip(pluginId: string, zipPath: string) {
      this.extractingPluginId = pluginId;
      this.error = null;
      try {
        await extractPluginZip(zipPath, pluginId);
        await this.refresh();
        this.bootstrapEpoch += 1;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        throw e;
      } finally {
        this.extractingPluginId = null;
      }
    },
    closePanel() {
      this.panelVisible = false;
    },
    setPersistScope(scope: PluginPersistScope) {
      if (this.persistScope === scope) {
        return;
      }
      const b = this.pluginStateBundle;
      if (b) {
        this.pluginState =
          scope === "role"
            ? clonePluginState(b.role)
            : clonePluginState(b.globalDefaults);
      }
      this.persistScope = scope;
    },
    togglePanel() {
      if (this.panelVisible) {
        this.closePanel();
      } else {
        void this.openPanel();
      }
    },
    async refresh() {
      if (refreshPromise) {
        return refreshPromise;
      }
      this.loading = true;
      this.error = null;
      refreshPromise = (async () => {
        try {
          const roleId = useRoleStore().currentRoleId;
          const [cat, bundle, boot] = await Promise.all([
            getDirectoryPluginCatalog(),
            getPluginState(roleId),
            getDirectoryPluginBootstrap(roleId),
          ]);
          this.pluginStateBundle = {
            role: clonePluginState(bundle.role),
            globalDefaults: clonePluginState(bundle.globalDefaults),
          };
          const st =
            this.persistScope === "role" ? bundle.role : bundle.globalDefaults;
          const nextState = clonePluginState(st);
          if (!catalogEqual(this.catalog, cat)) {
            this.catalog = cat;
            slotOrderMemo.clear();
            const bySlot: Record<string, string[]> = {};
            for (const p of cat) {
              if (p.isShell) continue;
              for (const slotName of p.uiSlotNames ?? []) {
                if (!bySlot[slotName]) bySlot[slotName] = [];
                bySlot[slotName].push(p.id);
              }
            }
            for (const slotName of Object.keys(bySlot)) {
              bySlot[slotName].sort();
            }
            this.catalogCandidatesBySlot = bySlot;
          }
          if (!rolePluginStateEqual(this.pluginState, nextState)) {
            this.pluginState = nextState;
          }
          this.applyDirectoryBootstrap(boot);
        } catch (e) {
          this.error = e instanceof Error ? e.message : String(e);
        } finally {
          this.loading = false;
          refreshPromise = null;
        }
      })();
      return refreshPromise;
    },
    async persist() {
      const roleId = useRoleStore().currentRoleId;
      if (this.persistScope === "global") {
        await saveGlobalPluginState(this.pluginState);
        if (this.pluginStateBundle) {
          this.pluginStateBundle.globalDefaults = clonePluginState(
            this.pluginState,
          );
        }
      } else {
        await savePluginState(roleId, this.pluginState);
        if (this.pluginStateBundle) {
          this.pluginStateBundle.role = clonePluginState(this.pluginState);
        }
      }
      this.bootstrapEpoch += 1;
      await this.syncDirectoryPluginBootstrap();
    },
    async resetToRolePackDefault() {
      const roleId = useRoleStore().currentRoleId;
      await resetPluginStateToRoleDefault(roleId);
      await this.refresh();
      this.bootstrapEpoch += 1;
    },
    isPluginDisabled(id: string): boolean {
      return this.pluginState.disabled_plugins.includes(id);
    },
    setPluginDisabled(id: string, disabled: boolean) {
      if (!disabled) {
        const entry = this.catalog.find((c) => c.id === id);
        if (entry && entry.dependencyStatus !== "ok") {
          throw new Error(
            `插件「${id}」依赖未满足，无法启用。${(entry.dependencyIssues ?? []).join("；")}`,
          );
        }
      }
      const set = new Set(this.pluginState.disabled_plugins);
      if (disabled) {
        set.add(id);
      } else {
        set.delete(id);
      }
      this.pluginState.disabled_plugins = [...set].sort();
    },
    /** 开发者模式：文件监听触发后刷新 catalog 与 bootstrap（整壳/插槽热重载）。 */
    async onPluginFilesChanged() {
      await this.refresh();
      this.bootstrapEpoch += 1;
      await this.syncDirectoryPluginBootstrap();
    },
    batchDisablePluginIds(ids: string[]) {
      const set = new Set(this.pluginState.disabled_plugins);
      for (const id of ids) {
        set.add(id);
      }
      this.pluginState.disabled_plugins = [...set].sort();
    },
    batchEnablePluginIds(ids: string[]) {
      for (const id of ids) {
        const entry = this.catalog.find((c) => c.id === id);
        if (entry && entry.dependencyStatus !== "ok") {
          throw new Error(
            `插件「${id}」依赖未满足，无法启用。${(entry.dependencyIssues ?? []).join("；")}`,
          );
        }
      }
      const set = new Set(this.pluginState.disabled_plugins);
      for (const id of ids) {
        set.delete(id);
      }
      this.pluginState.disabled_plugins = [...set].sort();
    },
    /** 对「检测到有更新」的插件提示需 zip 导入（在线静默更新未接入）。 */
    async batchUpdatePluginIds(
      ids: string[],
    ): Promise<{ count: number; targets: string[] }> {
      await this.checkPluginUpdatesFromRegistry();
      const targets = ids.filter((id) => this.pluginUpdateById[id]?.hasUpdate);
      if (targets.length === 0) {
        return { count: 0, targets: [] };
      }
      return { count: targets.length, targets };
    },
    /** 某插槽下、按 manifest 声明了该槽的非整壳插件 id 顺序（含未在 slot_order 中的，字典序补全）。 */
    pluginsOrderedForSlot(slot: string): string[] {
      const candidates = this.catalogCandidatesBySlot[slot] ?? [];
      const order = this.pluginState.slot_order[slot] ?? [];
      const signature = buildSlotOrderSignature(candidates, order);
      const memo = slotOrderMemo.get(slot);
      if (memo && memo.signature === signature) {
        return [...memo.value];
      }
      const candidateSet = new Set(candidates);
      const seen = new Set<string>();
      const out: string[] = [];
      for (const id of order) {
        if (candidateSet.has(id) && !seen.has(id)) {
          out.push(id);
          seen.add(id);
        }
      }
      for (const id of candidates) {
        if (!seen.has(id)) {
          out.push(id);
        }
      }
      slotOrderMemo.set(slot, { signature, value: out });
      return out;
    },
    isSlotContributionDisabled(slot: string, pluginId: string): boolean {
      const list = this.pluginState.disabled_slot_contributions[slot] ?? [];
      return list.includes(pluginId);
    },
    setSlotContributionDisabled(slot: string, pluginId: string, disabled: boolean) {
      const cur = [...(this.pluginState.disabled_slot_contributions[slot] ?? [])];
      const i = cur.indexOf(pluginId);
      if (disabled && i < 0) {
        cur.push(pluginId);
      } else if (!disabled && i >= 0) {
        cur.splice(i, 1);
      }
      this.pluginState.disabled_slot_contributions = {
        ...this.pluginState.disabled_slot_contributions,
        [slot]: cur,
      };
    },
    /** 设置某插件在某槽的选中外观（`appearance_id`）；空字符串表示清除为 manifest 默认。 */
    setSlotAppearance(pluginId: string, slot: string, appearanceId: string) {
      const pid = pluginId.trim();
      const sl = slot.trim();
      if (!pid || !sl) return;
      const nextOuter: Record<string, Record<string, string>> = {
        ...(this.pluginState.slot_appearance ?? {}),
      };
      const inner = { ...(nextOuter[pid] ?? {}) };
      const aid = appearanceId.trim();
      if (aid === "") {
        delete inner[sl];
      } else {
        inner[sl] = aid;
      }
      if (Object.keys(inner).length === 0) {
        delete nextOuter[pid];
      } else {
        nextOuter[pid] = inner;
      }
      this.pluginState = {
        ...this.pluginState,
        slot_appearance: nextOuter,
      };
    },
    movePluginInSlotOrder(slot: string, fromIndex: number, toIndex: number) {
      const ids = [...this.pluginsOrderedForSlot(slot)];
      if (
        fromIndex < 0 ||
        toIndex < 0 ||
        fromIndex >= ids.length ||
        toIndex >= ids.length
      ) {
        return;
      }
      const [m] = ids.splice(fromIndex, 1);
      if (m === undefined) {
        return;
      }
      ids.splice(toIndex, 0, m);
      this.pluginState.slot_order = {
        ...this.pluginState.slot_order,
        [slot]: ids,
      };
    },
    // --- 兼容旧名（chat_toolbar）---
    toolbarPluginsOrdered(): string[] {
      return this.pluginsOrderedForSlot(SLOT_CHAT_TOOLBAR);
    },
    moveToolbarPlugin(fromIndex: number, toIndex: number) {
      this.movePluginInSlotOrder(SLOT_CHAT_TOOLBAR, fromIndex, toIndex);
    },
    isToolbarContributionDisabled(pluginId: string): boolean {
      return this.isSlotContributionDisabled(SLOT_CHAT_TOOLBAR, pluginId);
    },
    setToolbarContributionDisabled(pluginId: string, disabled: boolean) {
      this.setSlotContributionDisabled(SLOT_CHAT_TOOLBAR, pluginId, disabled);
    },
  },
});
