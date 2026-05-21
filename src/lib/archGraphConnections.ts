import { addEdge, type Connection, type Edge } from "@vue-flow/core";
import {
  ARCH_GRAPH_BUS_ID,
  ARCH_GRAPH_COMPLEX_ID,
  ARCH_GRAPH_KERNEL_ID,
  type CoreModule,
} from "../composables/useArchitectureGraphModel";
import type { BackendKind } from "./graphEditorTheme";

export type ArchConnectReason =
  | "selfLoop"
  | "wrongDirection"
  | "unknownPort"
  | "pluginModuleMismatch"
  | "pluginBackendRequired"
  | "targetOccupied"
  | "systemEdgeLocked";

export type ArchConnectCheck = { valid: true } | { valid: false; reason: ArchConnectReason };

const CORE_MODULE_IDS = new Set<string>([
  "memory",
  "emotion",
  "event",
  "prompt",
  "llm",
  "agent",
]);

/** Runtime-derived edges; reconnect/delete blocked (ComfyUI fixed graph chrome). */
export const SYSTEM_EDGE_IDS = new Set([
  "kernel-bus",
  "bus-complex",
  "bus-memory",
  "bus-emotion",
  "bus-event",
  "bus-prompt",
  "bus-llm",
  "bus-agent",
]);

export function isSystemEdge(edge: Pick<Edge, "id" | "data" | "deletable">): boolean {
  if (edge.deletable === false) return true;
  if (SYSTEM_EDGE_IDS.has(String(edge.id))) return true;
  return Boolean(edge.data?.system);
}

function pluginIdFromNode(nodeId: string): string | null {
  if (!nodeId.startsWith("plugin:")) return null;
  return nodeId.slice("plugin:".length);
}

function moduleFromPluginNode(nodeId: string, nodesById: Map<string, { data?: Record<string, unknown> }>): CoreModule | null {
  const n = nodesById.get(nodeId);
  const key = n?.data?.moduleKey;
  return typeof key === "string" && CORE_MODULE_IDS.has(key) ? (key as CoreModule) : null;
}

/**
 * ComfyUI-style rules: output → input only, fixed port vocabulary, one wire per input handle.
 */
export function validateArchConnection(
  connection: Connection,
  edges: Edge[],
  nodes: Array<{ id: string; type?: string; data?: Record<string, unknown> }>,
): ArchConnectCheck {
  const { source, target, sourceHandle, targetHandle } = connection;
  if (!source || !target || !sourceHandle || !targetHandle) {
    return { valid: false, reason: "unknownPort" };
  }
  if (source === target) return { valid: false, reason: "selfLoop" };

  const nodesById = new Map(nodes.map((n) => [n.id, n]));

  if (source === ARCH_GRAPH_KERNEL_ID && target === ARCH_GRAPH_BUS_ID) {
    if (sourceHandle === "pipeline" && targetHandle === "pipeline-in") return { valid: true };
    return { valid: false, reason: "unknownPort" };
  }

  if (source === ARCH_GRAPH_BUS_ID && CORE_MODULE_IDS.has(target)) {
    const mod = target as CoreModule;
    if (sourceHandle === `fac-${mod}` && targetHandle === "backend-in") return { valid: true };
    return { valid: false, reason: "unknownPort" };
  }

  if (source === ARCH_GRAPH_BUS_ID && target === ARCH_GRAPH_COMPLEX_ID) {
    if (sourceHandle === "fac-complex" && targetHandle === "backend-in") return { valid: true };
    return { valid: false, reason: "unknownPort" };
  }

  if (CORE_MODULE_IDS.has(source) && target.startsWith("plugin:")) {
    const mod = source as CoreModule;
    const srcNode = nodesById.get(source);
    if (srcNode?.data?.backendKind !== "directory") {
      return { valid: false, reason: "pluginBackendRequired" };
    }
    if (sourceHandle !== "plugin-out" || targetHandle !== "plugin-in") {
      return { valid: false, reason: "unknownPort" };
    }
    const pluginMod = moduleFromPluginNode(target, nodesById);
    if (pluginMod !== mod) return { valid: false, reason: "pluginModuleMismatch" };
    return { valid: true };
  }

  return { valid: false, reason: "wrongDirection" };
}

export function archConnectReasonKey(reason: ArchConnectReason): string {
  const map: Record<ArchConnectReason, string> = {
    selfLoop: "pluginWorkbench.graph.connectSelfLoop",
    wrongDirection: "pluginWorkbench.graph.connectWrongDirection",
    unknownPort: "pluginWorkbench.graph.connectUnknownPort",
    pluginModuleMismatch: "pluginWorkbench.graph.connectPluginModule",
    pluginBackendRequired: "pluginWorkbench.graph.connectDirectoryOnly",
    targetOccupied: "pluginWorkbench.graph.connectTargetOccupied",
    systemEdgeLocked: "pluginWorkbench.graph.connectSystemLocked",
  };
  return map[reason];
}

/** One input handle → one edge (ComfyUI); replaces existing wire to same target handle. */
export function upsertArchEdge(connection: Connection, edges: Edge[], kind: BackendKind = "directory"): Edge[] {
  const withoutTarget = edges.filter(
    (e) => !(e.target === connection.target && e.targetHandle === connection.targetHandle),
  );
  const id = inferEdgeId(connection);
  const edge: Edge = {
    id,
    source: connection.source,
    target: connection.target,
    sourceHandle: connection.sourceHandle ?? undefined,
    targetHandle: connection.targetHandle ?? undefined,
    type: "archBackend",
    data: {
      kind,
      moduleKey: CORE_MODULE_IDS.has(connection.source) ? connection.source : undefined,
      system: SYSTEM_EDGE_IDS.has(id),
    },
  };
  return addEdge(edge, withoutTarget);
}

function inferEdgeId(connection: Connection): string {
  if (connection.source === ARCH_GRAPH_KERNEL_ID) return "kernel-bus";
  if (connection.source === ARCH_GRAPH_BUS_ID && connection.target === ARCH_GRAPH_COMPLEX_ID) {
    return "bus-complex";
  }
  if (connection.source === ARCH_GRAPH_BUS_ID && CORE_MODULE_IDS.has(connection.target ?? "")) {
    return `bus-${connection.target}`;
  }
  if (CORE_MODULE_IDS.has(connection.source ?? "") && connection.target?.startsWith("plugin:")) {
    const pid = pluginIdFromNode(connection.target);
    return `mod-${connection.source}-${pid}`;
  }
  return `wire-${connection.source}-${connection.sourceHandle}-${connection.target}-${connection.targetHandle}`;
}

export function edgeKindForConnection(connection: Connection, nodes: Array<{ id: string; data?: Record<string, unknown> }>): BackendKind {
  if (connection.source === ARCH_GRAPH_KERNEL_ID) return "builtin";
  if (connection.source === ARCH_GRAPH_BUS_ID && connection.target === ARCH_GRAPH_COMPLEX_ID) {
    return "builtin";
  }
  if (connection.source === ARCH_GRAPH_BUS_ID && CORE_MODULE_IDS.has(connection.target ?? "")) {
    const mod = connection.target as CoreModule;
    const n = nodes.find((x) => x.id === mod);
    const raw = String(n?.data?.backendKind ?? "builtin");
    if (raw === "remote" || raw === "directory") return raw;
    return "builtin";
  }
  return "directory";
}
