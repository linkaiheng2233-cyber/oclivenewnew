import { describe, expect, it } from "vitest";
import {
  addSlotToRegistry,
  canRemoveSlotKey,
  removeSlotFromRegistry,
  SLOT_REGISTRY_LAST_LLM,
  sortedSlotRegistryEntries,
} from "../lib/slotRegistry";

const basePack = {
  llm: { type: "llm", label: "L", backend: "ollama", position: 0 },
  memory: { type: "memory", label: "M", backend: "builtin", position: 0 },
};

describe("slotRegistry toolbar helpers", () => {
  it("adds a memory slot instance with auto key", () => {
    const { registry, key } = addSlotToRegistry(basePack, "memory", "短期记忆");
    expect(key).toBe("memory_2");
    expect(registry[key].label).toBe("短期记忆");
    expect(registry[key].type).toBe("memory");
    const keys = sortedSlotRegistryEntries(registry).map(([k]) => k);
    expect(keys).toContain("memory_2");
  });

  it("removes a non-llm slot instance", () => {
    const { registry } = addSlotToRegistry(basePack, "memory", "extra");
    const next = removeSlotFromRegistry(registry, "memory_2");
    expect(next.memory_2).toBeUndefined();
    expect(next.llm).toBeDefined();
  });

  it("cannot remove the last llm instance (button guard)", () => {
    expect(canRemoveSlotKey(basePack, "llm")).toBe(false);
    expect(() => removeSlotFromRegistry(basePack, "llm")).toThrow(SLOT_REGISTRY_LAST_LLM);
  });

  it("allows removing llm when another llm exists", () => {
    const withTwo = addSlotToRegistry(basePack, "llm", "B");
    expect(canRemoveSlotKey(withTwo.registry, "llm")).toBe(true);
    const next = removeSlotFromRegistry(withTwo.registry, "llm");
    expect(next.llm).toBeUndefined();
    expect(next.llm_2).toBeDefined();
  });
});
