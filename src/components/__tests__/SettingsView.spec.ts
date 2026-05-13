import { createTestingPinia } from "@pinia/testing";
import { mount, flushPromises } from "@vue/test-utils";
import { beforeAll, beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { setActivePinia } from "pinia";
import { i18n, prepareI18nForLocale } from "../../i18n";
import { useUiStore } from "../../stores/uiStore";
import SettingsTierSection from "../SettingsTierSection.vue";
import SettingsView from "../../views/SettingsView.vue";

vi.mock("../../utils/tauri-api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("../../utils/tauri-api")>();
  return {
    ...mod,
    getPluginMarketSourcesConfig: vi.fn(async () => ({
      developerMode: false,
      pluginIndexSources: [] as string[],
    })),
    setPluginIndexSources: vi.fn(async () => undefined),
    setPluginMarketDeveloperMode: vi.fn(async () => ({
      developerMode: false,
      pluginIndexSources: [] as string[],
    })),
  };
});

vi.mock("../../utils/isTauriWebview", () => ({
  isTauriWebview: () => false,
}));

vi.mock("../../composables/useCloudLlmTrustModal", () => ({
  buildCloudLlmTrustPlainText: (fn: (k: string) => string) => fn("k"),
  useCloudLlmTrustModal: () => ({
    visible: { value: false },
    open: vi.fn(),
    close: vi.fn(),
  }),
}));

vi.mock("../../composables/useHostModelPick", () => ({
  notifyHostModelsInventoryChanged: vi.fn(),
}));

const confirmMock = vi.fn();

vi.mock("@tauri-apps/api/dialog", () => ({
  confirm: (...args: unknown[]) => confirmMock(...args),
}));

/** VTU root excludes `<Teleport to="body">` children; keep nav in-tree for queries. */
const teleportInlineStub = { template: "<div class=\"sv-teleport-stub\"><slot /></div>" };

const asyncStub = { template: "<div class=\"async-stub\" />" };

const settingsViewStubs = {
  Teleport: teleportInlineStub,
  HelpHint: true,
  TrustConsentModal: true,
  CloudLlmQuickSetup: true,
  ShortcutsManagerPanel: asyncStub,
  ModelSelectorSettings: asyncStub,
  ExpertModelsSettingsHub: asyncStub,
  RoleManagerSettings: asyncStub,
  SettingsDebugEmbed: asyncStub,
  PluginSettingsPanelSlots: asyncStub,
  PluginSlotEmbed: asyncStub,
  PluginManagerPanel: asyncStub,
  PluginManagerV2Panel: asyncStub,
  PluginMarketV2Panel: asyncStub,
  ExpertModelsPanel: asyncStub,
  LocalModelManagerPanel: asyncStub,
  PluginMarketPanel: asyncStub,
};

function mountSettingsView() {
  const pinia = createTestingPinia({
    stubActions: true,
    initialState: {
      ui: {
        settingsDeveloperMaster: false,
        experimentalPluginManagerV2: false,
        languagePref: "en-US",
        sceneId: "home",
        settingsPendingNavId: null,
      },
    },
  });
  setActivePinia(pinia);

  return mount(SettingsView, {
    attachTo: document.body,
    props: { visible: true },
    global: {
      plugins: [pinia, i18n],
      stubs: settingsViewStubs,
    },
  });
}

describe("SettingsView tier / developer gate", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  beforeEach(() => {
    confirmMock.mockReset();
  });

  it("hides developer-gated nav entries when immersive and master off", async () => {
    const w = mountSettingsView();
    await flushPromises();
    const expertLabel = String(i18n.global.t("settings.nav.items.dataExpertModels"));
    const labels = w.findAll(".sv-tree-btn-label").map((x) => x.text());
    expect(labels.length).toBeGreaterThan(0);
    expect(labels.some((t) => t.includes(expertLabel))).toBe(false);
    w.unmount();
  });

  it("shows developer-gated nav when master is enabled", async () => {
    const pinia = createTestingPinia({
      stubActions: true,
      initialState: {
        ui: {
          settingsDeveloperMaster: true,
          experimentalPluginManagerV2: false,
          languagePref: "en-US",
          sceneId: "home",
          settingsPendingNavId: null,
        },
      },
    });
    setActivePinia(pinia);
    const w = mount(SettingsView, {
      attachTo: document.body,
      props: { visible: true },
      global: {
        plugins: [pinia, i18n],
        stubs: settingsViewStubs,
      },
    });
    await flushPromises();
    const expertLabel = String(i18n.global.t("settings.nav.items.dataExpertModels"));
    const labels = w.findAll(".sv-tree-btn-label").map((x) => x.text());
    expect(labels.length).toBeGreaterThan(0);
    expect(labels.some((t) => t.includes(expertLabel))).toBe(true);
    w.unmount();
  });

  it("developer master checkbox reveals gated nav items", async () => {
    const pinia = createTestingPinia({
      stubActions: (action, store) => !(store.$id === "ui" && action === "setSettingsDeveloperMaster"),
      initialState: {
        ui: {
          settingsDeveloperMaster: false,
          experimentalPluginManagerV2: false,
          languagePref: "en-US",
          sceneId: "home",
          settingsPendingNavId: null,
        },
      },
    });
    setActivePinia(pinia);
    const w = mount(SettingsView, {
      attachTo: document.body,
      props: { visible: true },
      global: {
        plugins: [pinia, i18n],
        stubs: settingsViewStubs,
      },
    });
    await flushPromises();
    const expertLabel = String(i18n.global.t("settings.nav.items.dataExpertModels"));
    const cb = w.find(".sv-dev-toggle input[type=\"checkbox\"]");
    expect(cb.exists()).toBe(true);
    expect((cb.element as HTMLInputElement).checked).toBe(false);
    expect(w.findAll(".sv-tree-btn-label").map((x) => x.text()).some((t) => t.includes(expertLabel))).toBe(false);
    await cb.setValue(true);
    await flushPromises();
    expect(useUiStore().settingsDeveloperMaster).toBe(true);
    expect(w.findAll(".sv-tree-btn-label").map((x) => x.text()).some((t) => t.includes(expertLabel))).toBe(true);
    w.unmount();
  });
});

describe("SettingsTierSection L4 expand", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  let confirmSpy: ReturnType<typeof vi.spyOn> | undefined;

  afterEach(() => {
    confirmSpy?.mockRestore();
    confirmSpy = undefined;
  });

  it("keeps slot collapsed until expand is confirmed", async () => {
    confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    const pinia = createTestingPinia({ stubActions: true });
    setActivePinia(pinia);
    const w = mount(SettingsTierSection, {
      props: { tier: "L4", resetKey: 0 },
      slots: { default: "<div class=\"l4-inner\">secret</div>" },
      global: { plugins: [pinia, i18n] },
    });
    expect(w.find(".l4-inner").exists()).toBe(true);
    expect(w.find(".l4-inner").isVisible()).toBe(false);
    await w.find(".sts-btn").trigger("click");
    await flushPromises();
    expect(confirmSpy).toHaveBeenCalled();
    const bodyEl = w.find(".sts-body").element as HTMLElement;
    expect(bodyEl.style.display).not.toBe("none");
    w.unmount();
  });

  it("does not expand L4 body when confirm is cancelled", async () => {
    confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    const pinia = createTestingPinia({ stubActions: true });
    setActivePinia(pinia);
    const w = mount(SettingsTierSection, {
      props: { tier: "L4", resetKey: 0 },
      slots: { default: "<div class=\"l4-inner\">secret</div>" },
      global: { plugins: [pinia, i18n] },
    });
    await w.find(".sts-btn").trigger("click");
    await flushPromises();
    expect(w.find(".l4-inner").isVisible()).toBe(false);
    w.unmount();
  });
});
