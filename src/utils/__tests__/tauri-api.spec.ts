import { describe, expect, it } from "vitest";
import {
  buildOclexpertPayload,
  parseOclexpertJson,
  validateExpertGraphNodes,
} from "../../lib/oclexpert";
import type { ExpertGraph, PackUiConfig, PluginBackends } from "../tauri-api";
import {
  emptyPackUiConfig,
  isInvalidParameterError,
  isPermissionDeniedError,
  isPluginNotFoundError,
  normalizePackUiConfig,
  parseApiErrorCode,
} from "../tauri-api";

describe("parseApiErrorCode", () => {
  it("extracts bracket code from invoke-style errors", () => {
    expect(parseApiErrorCode("[INVALID_PARAMETER] bad")).toBe("INVALID_PARAMETER");
    expect(parseApiErrorCode("prefix [API_PLUGIN_NOT_FOUND] suffix")).toBe(
      "API_PLUGIN_NOT_FOUND",
    );
  });

  it("returns undefined when no code token", () => {
    expect(parseApiErrorCode("plain message")).toBeUndefined();
    expect(parseApiErrorCode(null)).toBeUndefined();
  });
});

describe("error kind helpers", () => {
  it("classifies known codes", () => {
    expect(isPluginNotFoundError("[API_PLUGIN_NOT_FOUND]")).toBe(true);
    expect(isPluginNotFoundError("[INVALID_PARAMETER]")).toBe(false);
    expect(isPermissionDeniedError("[API_PERMISSION_DENIED]")).toBe(true);
    expect(isInvalidParameterError("[INVALID_PARAMETER]")).toBe(true);
  });
});

describe("normalizePackUiConfig", () => {
  it("returns empty shell when input missing", () => {
    const e = emptyPackUiConfig();
    expect(normalizePackUiConfig(null)).toEqual(e);
    expect(normalizePackUiConfig(undefined)).toEqual(e);
  });

  it("fills missing slot keys with empty order/visible", () => {
    const raw = { shell: "a", theme: {}, layout: {}, slots: {} } as unknown as PackUiConfig;
    const out = normalizePackUiConfig(raw);
    expect(out.shell).toBe("a");
    expect(out.slots.chat_toolbar.order).toEqual([]);
    expect(out.slots.sidebar.visible).toEqual([]);
  });

  it("preserves slot order/visible and normalizes theme/layout", () => {
    const raw: PackUiConfig = {
      shell: "s",
      theme: { primaryColor: "  #abc  ", backgroundColor: "", fontFamily: "  Mono  " },
      layout: { sidebar: " LEFT ", chatInput: " compact " },
      slots: {
        chat_toolbar: { order: ["a"], visible: ["b"] },
        "settings.panel": { order: [], visible: [] },
        "role.detail": { order: [], visible: [] },
        sidebar: { order: [], visible: [] },
        "chat.header": { order: [], visible: [] },
      },
    };
    const out = normalizePackUiConfig(raw);
    expect(out.theme.primaryColor).toBe("#abc");
    expect(out.theme.fontFamily).toBe("Mono");
    expect(out.layout.sidebar).toBe("left");
    expect(out.layout.chatInput).toBe("compact");
    expect(out.slots.chat_toolbar).toEqual({ order: ["a"], visible: ["b"] });
  });
});

describe("PluginBackends typing (compile-time contract smoke)", () => {
  it("accepts a full valid backend map", () => {
    const pb: PluginBackends = {
      memory: "builtin",
      emotion: "builtin_v2",
      event: "remote",
      prompt: "directory",
      llm: "ollama",
      agent: "none",
      complex_emotion: "builtin",
      directory_plugins: { llm: "com.example.llm" },
    };
    expect(pb.llm).toBe("ollama");
    expect(pb.directory_plugins?.llm).toBe("com.example.llm");
  });
});

describe("ExpertGraph JSON + .oclexpert round-trip", () => {
  const sampleGraph: ExpertGraph = {
    version: 1,
    nodes: [
      {
        type: "base_model",
        id: "base",
        ggufPath: "/models/x.gguf",
        ui: { x: 1, y: 2 },
      },
      {
        type: "lora_adapter",
        id: "l1",
        ggufPath: "/loras/a.gguf",
        strength: 0.5,
        enabled: true,
        order: 0,
        ui: null,
      },
    ],
    edges: [{ from: "base", to: "l1" }],
  };

  it("keeps structural equality through JSON clone", () => {
    const clone = JSON.parse(JSON.stringify(sampleGraph)) as ExpertGraph;
    expect(clone).toEqual(sampleGraph);
    validateExpertGraphNodes(clone);
  });

  it("buildOclexpertPayload + parseOclexpertJson round-trip", () => {
    const payload = buildOclexpertPayload(
      sampleGraph,
      { replyQualityAnchor: "x", corePersonality: null, description: null },
      { name: "n", description: "d", author: "a" },
    );
    const raw = JSON.stringify(payload);
    const parsed = parseOclexpertJson(raw);
    expect(parsed.graph).toEqual(sampleGraph);
    expect(parsed.promptStyle?.replyQualityAnchor).toBe("x");
    expect(parsed.suggestedName).toBe("n");
  });
});
