import { describe, expect, it } from "vitest";
import {
  ARCH_GRAPH_BUS_ID,
  ARCH_GRAPH_KERNEL_ID,
} from "../composables/useArchitectureGraphModel";
import { validateArchConnection } from "./archGraphConnections";

const nodes = [
  { id: ARCH_GRAPH_KERNEL_ID, type: "archKernel", data: {} },
  { id: ARCH_GRAPH_BUS_ID, type: "archBus", data: { moduleKeys: ["memory"] } },
  { id: "memory", type: "archModule", data: { moduleKey: "memory", backendKind: "directory" } },
  { id: "plugin:foo", type: "archPlugin", data: { moduleKey: "memory", pluginId: "foo" } },
];

describe("validateArchConnection", () => {
  it("allows kernel pipeline to bus", () => {
    expect(
      validateArchConnection(
        {
          source: ARCH_GRAPH_KERNEL_ID,
          target: ARCH_GRAPH_BUS_ID,
          sourceHandle: "pipeline",
          targetHandle: "pipeline-in",
        },
        [],
        nodes,
      ).valid,
    ).toBe(true);
  });

  it("rejects plugin wire when module is not directory", () => {
    const builtinNodes = [
      ...nodes.slice(0, 3),
      { ...nodes[2]!, data: { moduleKey: "memory", backendKind: "builtin" } },
      nodes[3]!,
    ];
    expect(
      validateArchConnection(
        {
          source: "memory",
          target: "plugin:foo",
          sourceHandle: "plugin-out",
          targetHandle: "plugin-in",
        },
        [],
        builtinNodes,
      ),
    ).toEqual({ valid: false, reason: "pluginBackendRequired" });
  });
});
