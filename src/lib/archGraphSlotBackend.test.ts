import { describe, expect, it } from "vitest";
import { patchSlotRegistryBackend } from "./archGraphSlotBackend";

const pack = {
  llm: { type: "llm", label: "L", backend: "ollama", position: 0 },
  memory: { type: "memory", label: "M", backend: "builtin", position: 0 },
};

describe("archGraphSlotBackend", () => {
  it("patches backend for an existing slot key", () => {
    const next = patchSlotRegistryBackend(pack, "memory", "remote");
    expect(next.memory.backend).toBe("remote");
    expect(next.llm.backend).toBe("ollama");
  });

  it("throws for unknown slot key", () => {
    expect(() => patchSlotRegistryBackend(pack, "missing", "builtin")).toThrow(
      /unknown slot key/,
    );
  });
});
