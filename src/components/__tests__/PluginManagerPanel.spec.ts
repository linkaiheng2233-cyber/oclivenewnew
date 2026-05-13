import { createTestingPinia } from "@pinia/testing";
import { mount, flushPromises } from "@vue/test-utils";
import { setActivePinia } from "pinia";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { i18n, prepareI18nForLocale } from "../../i18n";
import { usePluginStore } from "../../stores/pluginStore";
import type { DirectoryPluginCatalogEntry } from "../../utils/tauri-api";
import PluginManagerPanel from "../../views/PluginManagerPanel.vue";

const showToast = vi.fn();
vi.mock("../../composables/useAppToast", () => ({
  useAppToast: () => ({ showToast }),
}));

vi.mock("../../utils/tauri-api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("../../utils/tauri-api")>();
  return {
    ...mod,
    getPluginMarketSourcesConfig: vi.fn().mockResolvedValue({
      developerMode: false,
      pluginIndexSources: [] as string[],
    }),
  };
});

const teleportStub = { template: "<div class=\"tp-stub\"><slot /></div>" };

function catalogEntry(id: string): DirectoryPluginCatalogEntry {
  return {
    id,
    version: "1.0.0",
    pluginType: "directory",
    installMeta: null,
    hasUiSettings: false,
    hasRpcProcess: false,
    declaresRpcMethods: false,
    isShell: false,
    uiSlotNames: [],
    provides: [],
    dependencyStatus: "ok",
    dependencyIssues: [],
  };
}

function mountPm() {
  const pinia = createTestingPinia({
    stubActions: true,
    initialState: {
      plugin: {
        panelVisible: true,
        loading: false,
        error: null as string | null,
        panelMainTab: "plugins",
        catalog: [catalogEntry("com.test.alpha"), catalogEntry("com.test.beta")],
        supportedUiSlots: [] as string[],
        pluginUpdateById: {} as Record<string, { hasUpdate: boolean }>,
        persistScope: "role",
        pluginMarketSnapshot: null,
        pluginMarketSyncing: false,
        pluginMarketError: null,
        pluginUpdatesCheckLoading: false,
        bootstrapEpoch: 0,
      },
      role: {
        currentRoleId: "role-a",
        roleInfo: {
          authorPack: null,
        },
      },
    },
  });
  setActivePinia(pinia);
  const ps = usePluginStore();
  vi.mocked(ps.pluginsOrderedForSlot).mockReturnValue([]);
  vi.mocked(ps.toolbarPluginsOrdered).mockReturnValue([]);
  vi.mocked(ps.batchUpdatePluginsFromGitIndex).mockResolvedValue(undefined);
  vi.mocked(ps.batchEnablePluginIds).mockImplementation(() => {});
  vi.mocked(ps.batchDisablePluginIds).mockImplementation(() => {});

  return mount(PluginManagerPanel, {
    props: { embedded: true },
    attachTo: document.body,
    global: {
      plugins: [pinia, i18n],
      stubs: {
        Teleport: teleportStub,
        PluginBackendSessionPanel: { template: "<div data-stub-backend-session />" },
        ExpertModelsRuntimeCard: { template: "<div data-stub-expert-runtime />" },
        InstalledPluginWorkspaceDetail: { template: "<div data-stub-workspace-detail />" },
        PluginScaffoldWizard: { template: "<div />" },
        PmSlotRow: { template: "<div />" },
        PluginSlotEmbed: { template: "<div />" },
        HelpCircle: { template: "<span><slot /></span>" },
      },
    },
  });
}

describe("PluginManagerPanel", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  beforeEach(() => {
    showToast.mockClear();
  });

  it("renders installed plugin ids from catalog", async () => {
    const w = mountPm();
    await flushPromises();
    const ids = w.findAll(".pm-wb-item-id").map((x) => x.text());
    expect(ids).toContain("com.test.alpha");
    expect(ids).toContain("com.test.beta");
    w.unmount();
  });

  it("switches between plugins tab (market + installed) and backends tab", async () => {
    const w = mountPm();
    await flushPromises();
    const marketTitle = String(i18n.global.t("pluginManagerV1.ui.market.title"));
    const installedTitle = String(i18n.global.t("pluginManagerV1.ui.installed.title"));
    expect(w.text()).toContain(marketTitle);
    expect(w.text()).toContain(installedTitle);

    const tabs = w.findAll(".pm-tabs .pm-tab");
    const backendsTab = tabs.find((t) => t.text().includes(String(i18n.global.t("pluginManagerV1.ui.tabs.backends"))));
    expect(backendsTab?.exists()).toBe(true);
    await backendsTab!.trigger("click");
    await flushPromises();
    const llamaTitle = String(i18n.global.t("pluginManagerV1.ui.localLlama.title"));
    expect(w.text()).toContain(llamaTitle);

    const pluginsTab = tabs.find((t) => t.text().includes(String(i18n.global.t("pluginManagerV1.ui.tabs.plugins"))));
    await pluginsTab!.trigger("click");
    await flushPromises();
    expect(w.text()).toContain(installedTitle);
    w.unmount();
  });

  it("does not call batch git update when nothing selected; calls when one plugin checked", async () => {
    const w = mountPm();
    await flushPromises();
    const ps = usePluginStore();
    vi.mocked(ps.batchUpdatePluginsFromGitIndex).mockClear();

    const toolbar = w.find(".pm-primary-actions");
    await toolbar.findAll("button").find((b) => b.text().includes("Git"))!.trigger("click");
    await flushPromises();
    expect(ps.batchUpdatePluginsFromGitIndex).not.toHaveBeenCalled();

    const batchToggle = w.find(".pm-batch-toggle input[type=\"checkbox\"]");
    await batchToggle.setValue(true);
    await flushPromises();
    expect(w.find(".pm-batch-bar").exists()).toBe(false);

    const firstBatchCb = w.find(".pm-wb-batch input[type=\"checkbox\"]");
    await firstBatchCb.setValue(true);
    await flushPromises();
    expect(w.find(".pm-batch-bar").exists()).toBe(true);

    await w.find(".pm-batch-bar").findAll("button").find((b) => b.text().includes("Git"))!.trigger("click");
    await flushPromises();
    expect(ps.batchUpdatePluginsFromGitIndex).toHaveBeenCalledWith(["com.test.alpha"]);
    w.unmount();
  });
});
