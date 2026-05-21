import { computed, ref } from "vue";
import { Position, type Node } from "@vue-flow/core";
import { layoutOnRing, pointOnRay } from "../lib/radialGraphLayout";
import { normalizeBackendKind, type BackendKind } from "../lib/graphEditorTheme";
import { useRoleStore } from "../stores/roleStore";
import { usePluginStore } from "../stores/pluginStore";

export type CoreModule = "memory" | "emotion" | "event" | "prompt" | "llm" | "agent";

export const ARCH_GRAPH_BUS_ID = "__facility_bus__";
export const ARCH_GRAPH_KERNEL_ID = "__kernel__";
export const ARCH_GRAPH_COMPLEX_ID = "__complex_emotion__";

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

  const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);
  const pluginBackendsEffective = computed(() => roleStore.roleInfo.pluginBackendsEffective);
  const pluginBackendsSessionOverride = computed(
    () => roleStore.roleInfo.pluginBackendsSessionOverride,
  );

  function effectiveBackend(key: CoreModule): string {
    return String(pluginBackendsEffective.value[key] ?? "");
  }

  function backendKind(key: CoreModule): BackendKind {
    return normalizeBackendKind(effectiveBackend(key));
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

  function visiblePluginIds(key: CoreModule): string[] {
    const all = directoryPluginIds(key);
    if (backendKind(key) !== "directory" || all.length === 0) return [];
    if (expandedPlugins.value[key] || all.length <= 1) return all;
    return [all[0]!];
  }

  function hiddenPluginCount(key: CoreModule): number {
    return Math.max(0, directoryPluginIds(key).length - 1);
  }

  function togglePluginExpand(key: CoreModule) {
    expandedPlugins.value = { ...expandedPlugins.value, [key]: !expandedPlugins.value[key] };
  }

  const nodes = computed<Node[]>(() => {
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
  });

  const edges = computed(() => {
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
  });

  return {
    nodes,
    edges,
    pluginBackends,
    pluginBackendsSessionOverride,
    expandedPlugins,
    effectiveBackend,
    backendKind,
    directoryPluginIds,
    visiblePluginIds,
    hiddenPluginCount,
    togglePluginExpand,
    worldSize: { w: WORLD_W, h: WORLD_H },
  };
}
