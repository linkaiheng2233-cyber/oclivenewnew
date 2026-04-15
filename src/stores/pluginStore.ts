import { defineStore } from "pinia";
import { setHostEventSubscribedEvents } from "../lib/hostEventBus";
import {
  getDirectoryPluginBootstrap,
  getDirectoryPluginCatalog,
  getPluginState,
  resetPluginStateToRoleDefault,
  savePluginState,
  type DirectoryPluginBootstrap,
  type DirectoryPluginCatalogEntry,
  type RolePluginState,
} from "../utils/tauri-api";
import { useRoleStore } from "./roleStore";

/** 聊天输入区上方工具栏 */
export const SLOT_CHAT_TOOLBAR = "chat_toolbar";
/** 应用内「设置 → 插件扩展」页嵌入 */
export const SLOT_SETTINGS_PANEL = "settings.panel";
/** 左侧角色详情区底部（立绘与名称下方） */
export const SLOT_ROLE_DETAIL = "role.detail";

function emptyState(): RolePluginState {
  return {
    shellPluginId: "",
    disabled_plugins: [],
    slot_order: {},
    disabled_slot_contributions: {},
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
      x.isShell !== y.isShell ||
      !arraysEqual(x.uiSlotNames ?? [], y.uiSlotNames ?? []) ||
      !arraysEqual(x.provides ?? [], y.provides ?? [])
    ) {
      return false;
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
    )
  );
}

export const usePluginStore = defineStore("plugin", {
  state: () => ({
    panelVisible: false,
    loading: false,
    error: null as string | null,
    catalog: [] as DirectoryPluginCatalogEntry[],
    pluginState: emptyState() as RolePluginState,
    /** 与 `get_directory_plugin_bootstrap.developer_mode` 一致（扫描额外插件根等）。 */
    developerMode: false,
    /** 变更后嵌入插槽组件会重新拉 bootstrap */
    bootstrapEpoch: 0,
  }),
  actions: {
    /** 由 bootstrap DTO 更新宿主事件订阅与开发者模式（插槽与 `refresh` / `sync` 共用）。 */
    applyDirectoryBootstrap(boot: DirectoryPluginBootstrap) {
      setHostEventSubscribedEvents(boot.subscribedHostEvents ?? []);
      this.developerMode = boot.developerMode ?? false;
    },
    /** 角色切换或插件启用状态变更后更新宿主事件订阅与开发者模式（不拉 catalog）。 */
    async syncDirectoryPluginBootstrap() {
      const roleId = useRoleStore().currentRoleId;
      const boot = await getDirectoryPluginBootstrap(roleId);
      this.applyDirectoryBootstrap(boot);
    },
    async openPanel() {
      this.panelVisible = true;
      await this.refresh();
    },
    closePanel() {
      this.panelVisible = false;
    },
    togglePanel() {
      if (this.panelVisible) {
        this.closePanel();
      } else {
        void this.openPanel();
      }
    },
    async refresh() {
      this.loading = true;
      this.error = null;
      try {
        const roleId = useRoleStore().currentRoleId;
        const [cat, st, boot] = await Promise.all([
          getDirectoryPluginCatalog(),
          getPluginState(roleId),
          getDirectoryPluginBootstrap(roleId),
        ]);
        const nextState: RolePluginState = {
          shellPluginId: st.shellPluginId ?? "",
          disabled_plugins: [...(st.disabled_plugins ?? [])],
          slot_order: { ...st.slot_order },
          disabled_slot_contributions: { ...st.disabled_slot_contributions },
          force_iframe_mode: st.force_iframe_mode ?? false,
        };
        if (!catalogEqual(this.catalog, cat)) {
          this.catalog = cat;
        }
        if (!rolePluginStateEqual(this.pluginState, nextState)) {
          this.pluginState = nextState;
        }
        this.applyDirectoryBootstrap(boot);
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.loading = false;
      }
    },
    async persist() {
      const roleId = useRoleStore().currentRoleId;
      await savePluginState(roleId, this.pluginState);
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
      const set = new Set(this.pluginState.disabled_plugins);
      if (disabled) {
        set.add(id);
      } else {
        set.delete(id);
      }
      this.pluginState.disabled_plugins = [...set].sort();
    },
    /** 某插槽下、按 manifest 声明了该槽的非整壳插件 id 顺序（含未在 slot_order 中的，字典序补全）。 */
    pluginsOrderedForSlot(slot: string): string[] {
      const candidates = this.catalog
        .filter((p) => !p.isShell && p.uiSlotNames.includes(slot))
        .map((p) => p.id);
      const order = this.pluginState.slot_order[slot] ?? [];
      const seen = new Set<string>();
      const out: string[] = [];
      for (const id of order) {
        if (candidates.includes(id) && !seen.has(id)) {
          out.push(id);
          seen.add(id);
        }
      }
      for (const id of [...candidates].sort()) {
        if (!seen.has(id)) {
          out.push(id);
        }
      }
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
