import { createTestingPinia } from "@pinia/testing";
import { mount } from "@vue/test-utils";
import { setActivePinia } from "pinia";
import { beforeAll, describe, expect, it, vi } from "vitest";
import { i18n, prepareI18nForLocale } from "../../i18n";
import { PERSONALITY_TRAIT_KEYS } from "../../utils/personality-traits";
import DebugPanel from "../DebugPanel.vue";

vi.mock("../../utils/tauri-api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("../../utils/tauri-api")>();
  return {
    ...mod,
    generateMonologue: vi.fn().mockResolvedValue("hello"),
  };
});

const stubBlock = { template: "<div class=\"stub-block\" />" };

function mountDebug(embedded: boolean) {
  const pinia = createTestingPinia({
    stubActions: true,
    initialState: {
      role: {
        currentRoleId: "role-a",
        roleInfo: {
          interactionMode: "immersive",
          personalitySource: "vector",
          knowledgeEnabled: true,
          knowledgeChunkCount: 3,
        },
      },
      chat: {},
      debug: {
        lastKnowledgeChunksInPrompt: 2,
        lastKnowledgePresenceMode: "co_present",
      },
      plugin: { bootstrapEpoch: 0 },
      ui: { sceneId: "home" },
    },
  });
  setActivePinia(pinia);

  return mount(DebugPanel, {
    attachTo: document.body,
    props: {
      visible: true,
      loading: false,
      favorability: 55,
      personality: [0.72, 0.41, 0.55, 0.63, 0.5, 0.48, 0.66],
      events: [{ event_type: "scene", timestamp: "2024-01-01", description: "Test event" }],
      memories: [{ content: "Memory line", timestamp: "2024-01-02", importance: 0.8 }],
      embedded,
    },
    global: {
      plugins: [pinia, i18n],
      stubs: {
        HelpHint: { template: "<span />" },
        RoleRuntimePanel: stubBlock,
        ChatExportBar: stubBlock,
        PluginSlotEmbed: stubBlock,
        RolePackBar: stubBlock,
      },
    },
  });
}

describe("DebugPanel", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  it("hides close control when embedded", () => {
    const w = mountDebug(true);
    const closeLabel = String(i18n.global.t("common.close"));
    const closeBtn = w.findAll("button").find((b) => b.attributes("aria-label") === closeLabel);
    expect(closeBtn).toBeUndefined();
    w.unmount();
  });

  it("shows close control when not embedded", () => {
    const w = mountDebug(false);
    const closeLabel = String(i18n.global.t("common.close"));
    const closeBtn = w.findAll("button").find((b) => b.attributes("aria-label") === closeLabel);
    expect(closeBtn?.exists()).toBe(true);
    w.unmount();
  });

  it("renders personality trait rows for each dimension when vector data present", () => {
    const w = mountDebug(true);
    expect(w.findAll(".trait-item").length).toBe(PERSONALITY_TRAIT_KEYS.length);
    const values = w.findAll(".trait-value").map((x) => x.text());
    expect(values.length).toBe(PERSONALITY_TRAIT_KEYS.length);
    expect(values[0]).toMatch(/0\.72/);
    w.unmount();
  });

  it("lists recent events and memories from props", () => {
    const w = mountDebug(true);
    expect(w.text()).toContain("scene");
    expect(w.text()).toContain("Memory line");
    w.unmount();
  });
});
