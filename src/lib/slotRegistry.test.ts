import { describe, expect, it } from "vitest";
import { sortedSlotRegistryEntries, uniqueSlotTypes } from "./slotRegistry";

describe("slotRegistry", () => {
  it("sorts by type order then position", () => {
    const reg = {
      llm_b: { type: "llm", label: "b", backend: "ollama", position: 2 },
      memory_a: { type: "memory", label: "a", backend: "builtin", position: 1 },
      emotion: { type: "emotion", label: "e", backend: "builtin", position: 0 },
    };
    const keys = sortedSlotRegistryEntries(reg).map(([k]) => k);
    expect(keys[0]).toBe("emotion");
    expect(keys[1]).toBe("memory_a");
    expect(keys[2]).toBe("llm_b");
  });

  it("uniqueSlotTypes dedupes types", () => {
    const reg = {
      m1: { type: "memory", label: "1", backend: "builtin", position: 0 },
      m2: { type: "memory", label: "2", backend: "builtin_v2", position: 1 },
      llm: { type: "llm", label: "l", backend: "ollama", position: 0 },
    };
    expect(uniqueSlotTypes(reg)).toEqual(["memory", "llm"]);
  });
});
