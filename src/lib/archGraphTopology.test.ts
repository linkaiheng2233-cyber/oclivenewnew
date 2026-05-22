import { describe, expect, it } from "vitest";
import {
  ARCH_GRAPH_BUS_ID,
  ARCH_GRAPH_KERNEL_ID,
} from "../composables/useArchitectureGraphModel";
import {
  buildBlueprintArchitectureEdges,
  busFacHandleForType,
  orderedBusFacTypes,
  sortSlotsForArchitectureRing,
} from "./archGraphTopology";

const registry = {
  llm: { type: "llm", label: "L", backend: "ollama", position: 0 },
  memory: { type: "memory", label: "M", backend: "builtin", position: 0 },
  memory_2: { type: "memory", label: "M2", backend: "directory", position: 1, plugin: "p1" },
};

describe("archGraphTopology", () => {
  it("sorts slots in architecture ring type order", () => {
    const keys = sortSlotsForArchitectureRing(registry).map(([k]) => k);
    expect(keys.indexOf("memory")).toBeLessThan(keys.indexOf("llm"));
    expect(keys.indexOf("memory_2")).toBeGreaterThan(keys.indexOf("memory"));
  });

  it("maps complex_emotion to fac-complex handle", () => {
    expect(busFacHandleForType("complex_emotion")).toBe("fac-complex");
    expect(busFacHandleForType("memory")).toBe("fac-memory");
  });

  it("builds three wire layers including directory plugins", () => {
    const edges = buildBlueprintArchitectureEdges(registry, (key, entry) => {
      if (key === "memory_2" && entry.backend === "directory") return ["p1"];
      return [];
    });
    expect(edges.find((e) => e.id === "kernel-bus")?.source).toBe(ARCH_GRAPH_KERNEL_ID);
    expect(edges.find((e) => e.id === "kernel-bus")?.target).toBe(ARCH_GRAPH_BUS_ID);
    expect(edges.some((e) => e.id === "bus-memory_2")).toBe(true);
    expect(edges.some((e) => e.id === "slot-memory_2-p1")).toBe(true);
  });

  it("orders bus fac types for architecture diagram", () => {
    expect(orderedBusFacTypes(registry)).toEqual(["memory", "llm"]);
  });
});
