import { describe, expect, it } from "vitest";
import { buildRelationDropdownOptions } from "../relationOptions";
import { normalizeInteractionMode, packDefaultFromApi } from "../interactionMode";
import { vec7ToRecord, PERSONALITY_TRAIT_KEYS } from "../personality-traits";
import { OCLIVE_DEFAULT_RELATION_SENTINEL } from "../tauri-api";

describe("normalizeInteractionMode", () => {
  it("maps pure_chat string", () => {
    expect(normalizeInteractionMode("pure_chat")).toBe("pure_chat");
  });

  it("defaults unknown to immersive", () => {
    expect(normalizeInteractionMode(undefined)).toBe("immersive");
    expect(normalizeInteractionMode("")).toBe("immersive");
    expect(normalizeInteractionMode("immersive")).toBe("immersive");
  });
});

describe("packDefaultFromApi", () => {
  it("keeps only valid pack defaults", () => {
    expect(packDefaultFromApi("pure_chat")).toBe("pure_chat");
    expect(packDefaultFromApi("immersive")).toBe("immersive");
    expect(packDefaultFromApi("bogus")).toBeNull();
    expect(packDefaultFromApi(null)).toBeNull();
  });
});

describe("vec7ToRecord", () => {
  it("fills missing slots with zero", () => {
    const r = vec7ToRecord([0.5]);
    expect(PERSONALITY_TRAIT_KEYS.length).toBe(7);
    expect(r.stubbornness).toBe(0.5);
    expect(r.warmth).toBe(0);
  });

  it("treats non-finite as zero", () => {
    const r = vec7ToRecord([NaN, Infinity, ...Array(5).fill(0.1)]);
    expect(r.stubbornness).toBe(0);
    expect(r.clinginess).toBe(0);
  });
});

describe("buildRelationDropdownOptions", () => {
  it("prefixes default sentinel with manifest default label", () => {
    const rows = buildRelationDropdownOptions(
      [
        { id: "friend", name: "友人" },
        { id: "rival", name: "对手" },
      ],
      "friend",
    );
    expect(rows[0]?.id).toBe(OCLIVE_DEFAULT_RELATION_SENTINEL);
    expect(rows[0]?.name).toContain("友人");
    expect(rows).toHaveLength(3);
  });
});
