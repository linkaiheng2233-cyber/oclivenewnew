import {
  applyEdgeChanges,
  ConnectionMode,
  type Connection,
  type Edge,
  type EdgeChange,
  type GraphEdge,
  type Node,
} from "@vue-flow/core";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  archConnectReasonKey,
  edgeKindForConnection,
  isSystemEdge,
  upsertArchEdge,
  validateArchConnection,
} from "../lib/archGraphConnections";
import { useAppToast } from "./useAppToast";

export function useArchitectureGraphConnections(
  nodes: { value: Node[] },
  edges: { value: Edge[] },
) {
  const { t } = useI18n();
  const { showToast } = useAppToast();
  const isConnecting = ref(false);

  function checkConnection(connection: Connection) {
    return validateArchConnection(connection, edges.value, nodes.value);
  }

  function isValidConnection(connection: Connection | GraphEdge): boolean {
    const c: Connection = {
      source: "source" in connection ? String(connection.source) : "",
      target: "target" in connection ? String(connection.target) : "",
      sourceHandle: connection.sourceHandle ?? null,
      targetHandle: connection.targetHandle ?? null,
    };
    return checkConnection(c).valid;
  }

  function onConnect(connection: Connection) {
    const result = checkConnection(connection);
    if (!result.valid) {
      showToast("error", t(archConnectReasonKey(result.reason)));
      return;
    }
    const kind = edgeKindForConnection(connection, nodes.value);
    edges.value = upsertArchEdge(connection, edges.value, kind);
  }

  function onEdgesChange(changes: EdgeChange[]) {
    const allowed: EdgeChange[] = [];
    for (const c of changes) {
      if (c.type === "remove" && "id" in c) {
        const existing = edges.value.find((e) => e.id === c.id);
        if (existing?.deletable === false || (existing && isSystemEdge(existing))) {
          showToast("error", t("pluginWorkbench.graph.connectSystemLocked"));
          continue;
        }
      }
      allowed.push(c);
    }
    edges.value = applyEdgeChanges(allowed, edges.value);
  }

  function onEdgeUpdate({ connection }: { connection: Connection }) {
    if (!connection.source || !connection.target) return;
    const result = checkConnection(connection);
    if (!result.valid) {
      showToast("error", t(archConnectReasonKey(result.reason)));
      return;
    }
    const kind = edgeKindForConnection(connection, nodes.value);
    edges.value = upsertArchEdge(connection, edges.value, kind);
  }

  function onConnectStart() {
    isConnecting.value = true;
  }

  function onConnectEnd() {
    isConnecting.value = false;
  }

  return {
    connectionMode: ConnectionMode.Strict,
    isConnecting,
    isValidConnection,
    onConnect,
    onEdgesChange,
    onEdgeUpdate,
    onConnectStart,
    onConnectEnd,
  };
}
