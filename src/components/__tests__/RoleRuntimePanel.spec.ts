import { createTestingPinia } from "@pinia/testing";
import { mount, flushPromises } from "@vue/test-utils";
import { setActivePinia } from "pinia";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { i18n, prepareI18nForLocale } from "../../i18n";
import { useExpertModelsStore } from "../../stores/expertModelsStore";
import { usePluginStore } from "../../stores/pluginStore";
import type { ExpertGraph } from "../../utils/tauri-api";
import RoleRuntimePanel from "../RoleRuntimePanel.vue";

const showToast = vi.fn();
vi.mock("../../composables/useAppToast", () => ({
  useAppToast: () => ({ showToast }),
}));

const openExpertWorkbenchEdit = vi.fn();
vi.mock("../../lib/expertWorkbenchOpen", () => ({
  openExpertWorkbenchEdit: (...args: unknown[]) => openExpertWorkbenchEdit(...args),
}));

const teleportStub = { template: "<div class=\"tp-stub\"><slot /></div>" };

const emptyGraph = (): ExpertGraph => ({ version: 1, nodes: [], edges: [] });
const graphWithNode = (): ExpertGraph => ({
  version: 1,
  nodes: [{ type: "base_model", id: "b", ggufPath: "/m.gguf", ui: null }],
  edges: [],
});

function mountRoleRuntime(expertPartial: Record<string, unknown>) {
  const pinia = createTestingPinia({
    stubActions: true,
    initialState: {
      role: {
        currentRoleId: "role-a",
        roleInfo: {
          userRelations: [],
          eventImpactFactor: 1,
          personalitySource: "vector",
          description: "",
          version: "",
          author: "",
        },
      },
      expertModels: {
        loading: false,
        error: null,
        graphSource: "pack_default",
        effectiveGraph: emptyGraph(),
        effectivePromptStyle: null,
        ...expertPartial,
      },
    },
  });
  setActivePinia(pinia);
  const expert = useExpertModelsStore();
  vi.mocked(expert.refresh).mockImplementation(async () => {});
  vi.mocked(expert.clearSessionOverrideAndApply).mockResolvedValue({ ok: true } as never);
  const plugin = usePluginStore();
  vi.mocked(plugin.openPanel).mockImplementation(() => {});

  return mount(RoleRuntimePanel, {
    attachTo: document.body,
    global: {
      plugins: [pinia, i18n],
      stubs: {
        HelpHint: true,
        Teleport: teleportStub,
      },
    },
  });
}

describe("RoleRuntimePanel + ExpertModelsRuntimeCard", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  beforeEach(() => {
    showToast.mockClear();
    openExpertWorkbenchEdit.mockClear();
  });

  it("shows pure expert pill when graph is empty pack default", async () => {
    const w = mountRoleRuntime({
      graphSource: "pack_default",
      effectiveGraph: emptyGraph(),
    });
    await flushPromises();
    expect(w.find(".expert-runtime__pill--pure").exists()).toBe(true);
    expect(w.find(".expert-runtime__btn--danger").exists()).toBe(false);
    w.unmount();
  });

  it("shows role-default pill when role has non-empty default graph", async () => {
    const w = mountRoleRuntime({
      graphSource: "role_default",
      effectiveGraph: graphWithNode(),
    });
    await flushPromises();
    expect(w.find(".expert-runtime__pill--role").exists()).toBe(true);
    expect(w.find(".expert-runtime__btn--danger").exists()).toBe(false);
    w.unmount();
  });

  it("shows session pill and reset when session override active", async () => {
    const w = mountRoleRuntime({
      graphSource: "session_override",
      effectiveGraph: graphWithNode(),
    });
    await flushPromises();
    expect(w.find(".expert-runtime__pill--sess").exists()).toBe(true);
    expect(w.find(".expert-runtime__btn--danger").exists()).toBe(true);
    w.unmount();
  });

  it("opens detail modal on 查看详情 and calls workbench helper on edit", async () => {
    const w = mountRoleRuntime({
      graphSource: "pack_default",
      effectiveGraph: emptyGraph(),
    });
    await flushPromises();
    await w.find(".expert-runtime__actions .expert-runtime__btn").trigger("click");
    await flushPromises();
    expect(w.find(".expert-runtime__modal-title").exists()).toBe(true);
    await w.find(".expert-runtime__modal-actions .expert-runtime__btn").trigger("click");
    await flushPromises();

    const primary = w.findAll(".expert-runtime__actions .expert-runtime__btn--primary");
    expect(primary.length).toBe(1);
    await primary[0]!.trigger("click");
    expect(openExpertWorkbenchEdit).toHaveBeenCalledWith({ draftMode: "effective" });
    w.unmount();
  });

  it("opens backends panel when link clicked", async () => {
    const w = mountRoleRuntime({
      graphSource: "pack_default",
      effectiveGraph: emptyGraph(),
    });
    await flushPromises();
    await w.find(".link-open-backends").trigger("click");
    expect(usePluginStore().openPanel).toHaveBeenCalledWith("backends");
    w.unmount();
  });

  it("runs reset flow when session override and user confirms", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    const w = mountRoleRuntime({
      graphSource: "session_override",
      effectiveGraph: graphWithNode(),
    });
    await flushPromises();
    await w.find(".expert-runtime__btn--danger").trigger("click");
    await flushPromises();
    expect(confirmSpy).toHaveBeenCalled();
    expect(useExpertModelsStore().clearSessionOverrideAndApply).toHaveBeenCalled();
    confirmSpy.mockRestore();
    w.unmount();
  });
});
