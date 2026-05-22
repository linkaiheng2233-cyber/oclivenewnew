import { computed, ref } from "vue";
import { Position, type Node } from "@vue-flow/core";
import { layoutOnRing, pointOnRay } from "../lib/radialGraphLayout";
import { normalizeBackendKind, type BackendKind } from "../lib/graphEditorTheme";
import {
  buildBlueprintArchitectureEdges,
  orderedBusFacTypes,
  registryHasComplexEmotionSlot,
  sortSlotsForArchitectureRing,
} from "../lib/archGraphTopology";
import {
  primaryPluginId,
  SLOT_BACKEND_OPTIONS,
  SLOT_TYPE_ICONS,
  SLOT_TYPE_LABEL_KEYS,
  formatSlotZoneLabel,
  sortedSlotRegistryEntries,
  type SlotRegistryEntry,
  type SlotRegistryMap,
  type SlotGroupsMap,
} from "../lib/slotRegistry";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";

export type CoreModule = "memory" | "emotion" | "event" | "prompt" | "llm" | "agent";

export const ARCH_GRAPH_BUS_ID = "__facility_bus__";
export const ARCH_GRAPH_KERNEL_ID = "__kernel__";
export const ARCH_GRAPH_COMPLEX_ID = "__complex_emotion__";

const GROUP_PAD = 24;
const GROUP_HEADER_H = 32;

export function archGroupNodeId(groupId: string): string {
  return `group:${groupId}`;
}

const WORLD_W = 1040;
const WORLD_H = 720;
const NODE_W = 220;
const NODE_H = 112;
const PLUGIN_W = 180;
const PLUGIN_H = 72;
const HUB_CX = WORLD_W / 2;
const HUB_CY = WORLD_H / 2 - 6;
const KERNEL_OUTER_R = 268;
const MODULE_RING = 178;
const PLUGIN_INSET = 88;
const KERNEL_SIZE = 124;
const BUS_W = 240;
const BUS_H = 100;

export const coreModules: {
  key: CoreModule;
  labelKey: string;
  icon: string;
  options: string[];
}[] = [
  { key: "memory", labelKey: "pluginWorkbench.graph.memory", icon: "🧠", options: ["builtin", "builtin_v2", "remote", "local", "directory"] },
  { key: "emotion", labelKey: "pluginWorkbench.graph.emotion", icon: "💭", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "event", labelKey: "pluginWorkbench.graph.event", icon: "⚡", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "prompt", labelKey: "pluginWorkbench.graph.prompt", icon: "📝", options: ["builtin", "builtin_v2", "remote", "directory"] },
  { key: "llm", labelKey: "pluginWorkbench.graph.llm", icon: "🤖", options: ["ollama", "remote", "directory"] },
  { key: "agent", labelKey: "pluginWorkbench.graph.agent", icon: "🛠", options: ["builtin", "remote", "directory"] },
];

export function useArchitectureGraphModel() {
  const roleStore = useRoleStore();
  const pluginStore = usePluginStore();
  const expandedPlugins = ref<Record<string, boolean>>({});
  const collapsedGroups = ref<Record<string, boolean>>({});

  const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
  const pluginBackendsEffective = computed(() => roleStore.roleInfo.pluginBackendsEffective);
  const pluginBackendsSessionOverride = computed(
    () => roleStore.roleInfo.pluginBackendsSessionOverride,
  );

  const slotRegistryPack = computed(() => roleStore.roleInfo.slotRegistryPack);
  const slotRegistryEffective = computed(() => roleStore.roleInfo.slotRegistryEffective);
  const slotSessionOverriddenKeys = computed(
    () => roleStore.roleInfo.slotSessionOverriddenKeys,
  );
  const blueprintGroupsPack = computed(() => roleStore.roleInfo.blueprintGroupsPack);

  const usesBlueprint = computed(() => {
    const eff = slotRegistryEffective.value;
    return eff != null && Object.keys(eff).length > 0;
  });

  const dualCoreEnabled = computed(
    () => roleStore.roleInfo.dualCoreEnabled === true,
  );
  const pipelineExperimentalActions = computed(
    () => roleStore.roleInfo.pipelineExperimentalActions ?? [],
  );

  function effectiveBackend(key: CoreModule): string {
    return String(pluginBackendsEffective.value[key] ?? "");
  }

  function backendKind(key: CoreModule): BackendKind {
    return normalizeBackendKind(effectiveBackend(key));
  }

  function slotBackendKind(entry: SlotRegistryEntry): BackendKind {
    return normalizeBackendKind(entry.backend);
  }

  function isSlotSessionOverridden(slotKey: string): boolean {
    return slotSessionOverriddenKeys.value.includes(slotKey);
  }

  function directoryPluginIdsForSlot(slotKey: string, entry: SlotRegistryEntry): string[] {
    if (slotBackendKind(entry) !== "directory") return [];
    const pid = primaryPluginId(entry);
    if (!pid) return [];
    const extra = (entry.plugins ?? []).map((s) => s.trim()).filter(Boolean);
    const set = new Set<string>([pid, ...extra]);
    return [...set];
  }

  function directoryPluginIds(key: CoreModule): string[] {
    const dp = pluginBackendsEffective.value.directory_plugins;
    const primary = dp?.[key]?.trim() ?? "";
    const set = new Set<string>();
    if (primary) set.add(primary);
    for (const mod of coreModules) {
      const id = dp?.[mod.key]?.trim();
      if (id) set.add(id);
    }
    const list = [...set];
    if (primary) return [primary, ...list.filter((id) => id !== primary)];
    return list;
  }

  function visiblePluginIds(key: string, entry?: SlotRegistryEntry): string[] {
    if (usesBlueprint.value && entry) {
      const all = directoryPluginIdsForSlot(key, entry);
      if (all.length === 0) return [];
      if (expandedPlugins.value[key] || all.length <= 1) return all;
      return [all[0]!];
    }
    const all = directoryPluginIds(key as CoreModule);
    if (backendKind(key as CoreModule) !== "directory" || all.length === 0) return [];
    if (expandedPlugins.value[key] || all.length <= 1) return all;
    return [all[0]!];
  }

  function hiddenPluginCount(key: string, entry?: SlotRegistryEntry): number {
    if (usesBlueprint.value && entry) {
      return Math.max(0, directoryPluginIdsForSlot(key, entry).length - 1);
    }
    return Math.max(0, directoryPluginIds(key as CoreModule).length - 1);
  }

  function togglePluginExpand(key: string) {
    expandedPlugins.value = { ...expandedPlugins.value, [key]: !expandedPlugins.value[key] };
  }

  function buildLegacyNodes(): Node[] {
    const list: Node[] = [];

    list.push({
      id: ARCH_GRAPH_KERNEL_ID,
      type: "archKernel",
      position: { x: HUB_CX - KERNEL_SIZE / 2, y: HUB_CY - KERNEL_OUTER_R - KERNEL_SIZE / 2 },
      data: { labelKey: "pluginWorkbench.graph.kernel", sub: "process_message" },
      draggable: true,
    });

    list.push({
      id: ARCH_GRAPH_BUS_ID,
      type: "archBus",
      position: { x: HUB_CX - BUS_W / 2, y: HUB_CY - BUS_H / 2 },
      data: {
        labelKey: "pluginWorkbench.graph.facilityBus",
        hintKey: "pluginWorkbench.graph.facilityBusHint",
        moduleKeys: coreModules.map((m) => m.key),
      },
      draggable: true,
    });

    coreModules.forEach((m, i) => {
      const ring = layoutOnRing(HUB_CX, HUB_CY, MODULE_RING, i, coreModules.length, NODE_W, NODE_H);
      const kind = backendKind(m.key);
      list.push({
        id: m.key,
        type: "archModule",
        position: { x: ring.x, y: ring.y },
        data: {
          moduleKey: m.key,
          labelKey: m.labelKey,
          icon: m.icon,
          options: m.options,
          backend: effectiveBackend(m.key),
          backendKind: kind,
          packDefault: pluginBackends.value[m.key],
          sessionOverride: pluginBackendsSessionOverride.value?.[m.key] ?? "__pack_default__",
          primaryPlugin: directoryPluginIds(m.key)[0] ?? "",
          hiddenPluginCount: hiddenPluginCount(m.key),
          blueprintV2: false,
          targetPosition: Position.Left,
          sourcePosition: Position.Right,
        },
        draggable: true,
      });

      if (kind === "directory") {
        visiblePluginIds(m.key).forEach((pid, j) => {
          const center = pointOnRay(ring.cx, ring.cy, ring.angle + Math.PI, PLUGIN_INSET + j * (PLUGIN_H + 12));
          list.push({
            id: `plugin:${pid}`,
            type: "archPlugin",
            position: { x: center.x - PLUGIN_W / 2, y: center.y - PLUGIN_H / 2 },
            data: {
              pluginId: pid,
              moduleKey: m.key,
              disabled: pluginStore.isPluginDisabled(pid),
              version: pluginStore.catalog.find((c) => c.id === pid)?.version ?? "?",
            },
            draggable: true,
          });
        });
      }
    });

    const angle = Math.PI / 2 + 0.18;
    const cx0 = HUB_CX + Math.cos(angle) * (MODULE_RING * 0.78);
    const cy0 = HUB_CY + Math.sin(angle) * (MODULE_RING * 0.78);
    list.push({
      id: ARCH_GRAPH_COMPLEX_ID,
      type: "archComplex",
      position: { x: cx0 - 100, y: cy0 - NODE_H / 2 },
      data: { labelKey: "pluginWorkbench.graph.complexEmotion", hintKey: "pluginWorkbench.graph.complexHint" },
      draggable: true,
    });

    return list;
  }

  function buildBlueprintNodes(registry: SlotRegistryMap): Node[] {
    const list: Node[] = [];
    const entries = sortSlotsForArchitectureRing(registry);
    const groups: SlotGroupsMap = blueprintGroupsPack.value ?? {};
    const memberToGroup: Record<string, string> = {};
    for (const [gid, g] of Object.entries(groups)) {
      for (const m of g.members) memberToGroup[m] = gid;
    }

    list.push({
      id: ARCH_GRAPH_KERNEL_ID,
      type: "archKernel",
      position: { x: HUB_CX - KERNEL_SIZE / 2, y: HUB_CY - KERNEL_OUTER_R - KERNEL_SIZE / 2 },
      data: { labelKey: "pluginWorkbench.graph.kernel", sub: "process_message" },
      draggable: true,
    });

    list.push({
      id: ARCH_GRAPH_BUS_ID,
      type: "archBus",
      position: { x: HUB_CX - BUS_W / 2, y: HUB_CY - BUS_H / 2 },
      data: {
        labelKey: "pluginWorkbench.graph.facilityBus",
        hintKey: "pluginWorkbench.graph.facilityBusHintBlueprint",
        moduleKeys: orderedBusFacTypes(registry),
        blueprintV2: true,
        slotCount: entries.length,
      },
      draggable: true,
    });

    type AbsPos = { absX: number; absY: number; ring: ReturnType<typeof layoutOnRing> };
    const absBySlot = new Map<string, AbsPos>();
    const pendingModules: Array<{ slotKey: string; entry: SlotRegistryEntry; i: number }> = [];
    const pendingPlugins: Node[] = [];

    entries.forEach(([slotKey, entry], i) => {
      const ring = layoutOnRing(HUB_CX, HUB_CY, MODULE_RING, i, entries.length, NODE_W, NODE_H);
      absBySlot.set(slotKey, { absX: ring.x, absY: ring.y, ring });
      pendingModules.push({ slotKey, entry, i });
    });

    type GroupBounds = { minX: number; minY: number; maxX: number; maxY: number; type: string; label: string };
    const boundsByGroup = new Map<string, GroupBounds>();

    for (const [gid, g] of Object.entries(groups)) {
      let minX = Infinity;
      let minY = Infinity;
      let maxX = -Infinity;
      let maxY = -Infinity;
      let any = false;
      for (const m of g.members) {
        const pos = absBySlot.get(m);
        const ent = registry[m];
        if (!pos || !ent) continue;
        any = true;
        minX = Math.min(minX, pos.absX);
        minY = Math.min(minY, pos.absY);
        maxX = Math.max(maxX, pos.absX + NODE_W);
        maxY = Math.max(maxY, pos.absY + NODE_H);
        if (slotBackendKind(ent) === "directory") {
          visiblePluginIds(m, ent).forEach((pid, j) => {
            const center = pointOnRay(
              pos.ring.cx,
              pos.ring.cy,
              pos.ring.angle + Math.PI,
              PLUGIN_INSET + j * (PLUGIN_H + 12),
            );
            minX = Math.min(minX, center.x - PLUGIN_W / 2);
            minY = Math.min(minY, center.y - PLUGIN_H / 2);
            maxX = Math.max(maxX, center.x + PLUGIN_W / 2);
            maxY = Math.max(maxY, center.y + PLUGIN_H / 2);
          });
        }
      }
      if (any) {
        boundsByGroup.set(gid, {
          minX: minX - GROUP_PAD,
          minY: minY - GROUP_PAD - GROUP_HEADER_H,
          maxX: maxX + GROUP_PAD,
          maxY: maxY + GROUP_PAD,
          type: g.type,
          label: g.label,
        });
      }
    }

    for (const [gid, b] of boundsByGroup) {
      const g = groups[gid];
      list.push({
        id: archGroupNodeId(gid),
        type: "archGroup",
        position: { x: b.minX, y: b.minY },
        style: {
          width: `${b.maxX - b.minX}px`,
          height: `${b.maxY - b.minY}px`,
          zIndex: 0,
        },
        data: {
          groupId: gid,
          label: b.label,
          slotType: g?.type ?? b.type,
          collapsed: Boolean(collapsedGroups.value[gid]),
        },
        draggable: false,
        selectable: true,
        zIndex: 0,
      });
    }

    for (const { slotKey, entry } of pendingModules) {
      const pos = absBySlot.get(slotKey)!;
      const gid = memberToGroup[slotKey];
      const collapsed = gid ? Boolean(collapsedGroups.value[gid]) : false;
      const bounds = gid ? boundsByGroup.get(gid) : undefined;
      const parentNode = bounds ? archGroupNodeId(gid) : undefined;
      let x = pos.absX;
      let y = pos.absY;
      if (bounds) {
        x = pos.absX - bounds.minX;
        y = pos.absY - bounds.minY;
      }
      const kind = slotBackendKind(entry);
      const pack = slotRegistryPack.value?.[slotKey];
      const overridden = isSlotSessionOverridden(slotKey);
      const labelKey = SLOT_TYPE_LABEL_KEYS[entry.type] ?? "pluginWorkbench.graph.memory";
      const options = SLOT_BACKEND_OPTIONS[entry.type] ?? ["builtin"];
      list.push({
        id: slotKey,
        type: "archModule",
        position: { x, y },
        parentNode,
        hidden: collapsed,
        extent: parentNode ? ("parent" as const) : undefined,
        data: {
          slotKey,
          slotType: entry.type,
          moduleKey: entry.type,
          labelKey,
          slotLabel: entry.label,
          icon: SLOT_TYPE_ICONS[entry.type] ?? "⚙",
          options,
          backend: entry.backend,
          backendKind: kind,
          packDefault: pack?.backend ?? entry.backend,
          sessionOverride: overridden ? entry.backend : "__pack_default__",
          sessionOverridden: overridden,
          effectiveBackend: entry.backend,
          primaryPlugin: primaryPluginId(entry),
          hiddenPluginCount: hiddenPluginCount(slotKey, entry),
          blueprintV2: true,
          zoneLabel: formatSlotZoneLabel(entry.zone),
          groupId: gid,
          targetPosition: Position.Left,
          sourcePosition: Position.Right,
        },
        draggable: !parentNode,
        zIndex: 1,
      });

      if (kind === "directory" && !collapsed) {
        visiblePluginIds(slotKey, entry).forEach((pid, j) => {
          const center = pointOnRay(
            pos.ring.cx,
            pos.ring.cy,
            pos.ring.angle + Math.PI,
            PLUGIN_INSET + j * (PLUGIN_H + 12),
          );
          let px = center.x - PLUGIN_W / 2;
          let py = center.y - PLUGIN_H / 2;
          if (bounds) {
            px -= bounds.minX;
            py -= bounds.minY;
          }
          list.push({
            id: `plugin:${pid}`,
            type: "archPlugin",
            position: { x: px, y: py },
            parentNode,
            extent: parentNode ? ("parent" as const) : undefined,
            data: {
              pluginId: pid,
              moduleKey: entry.type,
              slotKey,
              disabled: pluginStore.isPluginDisabled(pid),
              version: pluginStore.catalog.find((c) => c.id === pid)?.version ?? "?",
            },
            draggable: !parentNode,
            zIndex: 1,
          });
        });
      }
    }

    if (!registryHasComplexEmotionSlot(registry)) {
      const angle = Math.PI / 2 + 0.18;
      const cx0 = HUB_CX + Math.cos(angle) * (MODULE_RING * 0.78);
      const cy0 = HUB_CY + Math.sin(angle) * (MODULE_RING * 0.78);
      list.push({
        id: ARCH_GRAPH_COMPLEX_ID,
        type: "archComplex",
        position: { x: cx0 - 100, y: cy0 - NODE_H / 2 },
        data: {
          labelKey: "pluginWorkbench.graph.complexEmotion",
          hintKey: "pluginWorkbench.graph.complexHint",
        },
        draggable: true,
        zIndex: 1,
      });
    }

    return list;
  }

  function toggleGroupCollapse(groupId: string) {
    collapsedGroups.value = {
      ...collapsedGroups.value,
      [groupId]: !collapsedGroups.value[groupId],
    };
  }

  /** 架构图：展开所有 directory 槽位下的目录插件链（第三层连线可见）。 */
  function expandAllDirectoryPlugins(registry: SlotRegistryMap) {
    const next = { ...expandedPlugins.value };
    for (const [slotKey, entry] of Object.entries(registry)) {
      if (slotBackendKind(entry) === "directory") {
        next[slotKey] = true;
      }
    }
    expandedPlugins.value = next;
  }

  const nodes = computed<Node[]>(() => {
    if (usesBlueprint.value && slotRegistryEffective.value) {
      return buildBlueprintNodes(slotRegistryEffective.value);
    }
    return buildLegacyNodes();
  });

  function buildLegacyEdges() {
    const out = [];

    out.push({
      id: "kernel-bus",
      source: ARCH_GRAPH_KERNEL_ID,
      target: ARCH_GRAPH_BUS_ID,
      sourceHandle: "pipeline",
      targetHandle: "pipeline-in",
      type: "archBackend",
      deletable: false,
      updatable: false,
      data: { kind: "builtin", system: true },
    });

    coreModules.forEach((m) => {
      const kind = backendKind(m.key);
      out.push({
        id: `bus-${m.key}`,
        source: ARCH_GRAPH_BUS_ID,
        target: m.key,
        sourceHandle: `fac-${m.key}`,
        targetHandle: "backend-in",
        type: "archBackend",
        deletable: false,
        updatable: false,
        data: { kind, moduleKey: m.key, system: true },
        animated: kind === "remote",
      });
      for (const pid of visiblePluginIds(m.key)) {
        out.push({
          id: `mod-${m.key}-${pid}`,
          source: m.key,
          target: `plugin:${pid}`,
          sourceHandle: "plugin-out",
          targetHandle: "plugin-in",
          type: "archBackend",
          deletable: true,
          updatable: true,
          data: { kind: "directory", moduleKey: m.key },
        });
      }
    });

    out.push({
      id: "bus-complex",
      source: ARCH_GRAPH_BUS_ID,
      target: ARCH_GRAPH_COMPLEX_ID,
      sourceHandle: "fac-complex",
      targetHandle: "backend-in",
      type: "archBackend",
      deletable: false,
      updatable: false,
      data: { kind: "builtin", system: true },
    });

    return out;
  }

  const edges = computed(() => {
    if (usesBlueprint.value && slotRegistryEffective.value) {
      return buildBlueprintArchitectureEdges(
        slotRegistryEffective.value,
        (slotKey, entry) => visiblePluginIds(slotKey, entry),
      );
    }
    return buildLegacyEdges();
  });

  return {
    nodes,
    edges,
    usesBlueprint,
    slotRegistryPack,
    slotRegistryEffective,
    slotSessionOverriddenKeys,
    pluginBackends,
    pluginBackendsSessionOverride,
    expandedPlugins,
    effectiveBackend,
    backendKind,
    directoryPluginIds,
    visiblePluginIds,
    hiddenPluginCount,
    togglePluginExpand,
    toggleGroupCollapse,
    expandAllDirectoryPlugins,
    blueprintGroupsPack,
    dualCoreEnabled,
    pipelineExperimentalActions,
    worldSize: { w: WORLD_W, h: WORLD_H },
  };
}
