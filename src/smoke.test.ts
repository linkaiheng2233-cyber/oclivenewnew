import { describe, it, expect } from "vitest";

describe("main repo smoke", () => {
  it("vitest pipeline is wired", () => {
    expect(1 + 1).toBe(2);
  });
});
