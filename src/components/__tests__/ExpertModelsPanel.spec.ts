import { createTestingPinia } from "@pinia/testing";
import { mount, flushPromises } from "@vue/test-utils";
import { setActivePinia } from "pinia";
import { beforeAll, describe, expect, it, vi } from "vitest";
import { i18n, prepareI18nForLocale } from "../../i18n";
import ExpertModelsPanel from "../ExpertModels/ExpertModelsPanel.vue";
import type { ExpertGraph } from "../../utils/tauri-api";

const showToast = vi.fn();

vi.mock("../../composables/useAppToast", () => ({
  useAppToast: () => ({ showToast }),
}));

vi.mock("@tauri-apps/api/shell", () => ({
  open: vi.fn().mockResolvedValue(undefined),
}));

const saveMock = vi.fn();
const openMock = vi.fn();
const writeTextFileMock = vi.fn().mockResolvedValue(undefined);
const readTextFileMock = vi.fn();

vi.mock("@tauri-apps/api/dialog", () => ({
  save: (...args: unknown[]) => saveMock(...args),
  open: (...args: unknown[]) => openMock(...args),
}));

vi.mock("@tauri-apps/api/fs", () => ({
  writeTextFile: (...args: unknown[]) => writeTextFileMock(...args),
  readTextFile: (...args: unknown[]) => readTextFileMock(...args),
}));

function mountExpertPanel() {
  const pinia = createTestingPinia({
    stubActions: true,
    initialState: {
      expertModels: {
        baseModels: [{ name: "stub.gguf", path: "/stub/stub.gguf" }],
        loading: false,
        loras: [{ name: "lora.gguf", path: "/stub/lora.gguf" }],
        draftGraph: {
          version: 1,
          nodes: [
            {
              type: "base_model",
              id: "base",
              ggufPath: "/models/base.gguf",
              ui: null,
            },
          ],
          edges: [],
        } as ExpertGraph,
        effectiveGraph: {
          version: 1,
          nodes: [
            {
              type: "base_model",
              id: "base",
              ggufPath: "/models/base.gguf",
              ui: null,
            },
          ],
          edges: [],
        } as ExpertGraph,
        graphSource: "session_override",
        promptStyleSource: "role_default",
      },
    },
  });

  setActivePinia(pinia);

  return mount(ExpertModelsPanel, {
    attachTo: document.body,
    global: {
      plugins: [pinia, i18n],
      stubs: {
        ExpertModelsCanvas: { template: "<div data-stub-canvas />" },
        ExpertCloudEventSection: { template: "<div />" },
        OclexpertPublishWizard: { template: "<div />" },
      },
    },
    props: { embedded: true },
  });
}

describe("ExpertModelsPanel", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  beforeEach(() => {
    vi.clearAllMocks();
    saveMock.mockReset();
    openMock.mockReset();
    writeTextFileMock.mockReset();
    readTextFileMock.mockReset();
  });

  it("renders graph source labels for store graphSource / promptStyleSource", async () => {
    const w = mountExpertPanel();
    await flushPromises();
    const html = w.html();
    expect(html).toContain(String(i18n.global.t("expertModels.meta.graphSource")));
    expect(html).toContain(String(i18n.global.t("expertModels.source.sessionOverride")));
    expect(html).toContain(String(i18n.global.t("expertModels.source.roleDefault")));
    w.unmount();
  });

  it("exports workflow JSON matching draft graph", async () => {
    saveMock.mockResolvedValue("/tmp/out.json");
    const w = mountExpertPanel();
    await flushPromises();
    await w.find("#em-workflow-name").setValue("mywf");
    const btns = w.findAll("button");
    const exportBtn = btns.find((b) => b.text().includes(String(i18n.global.t("expertModels.workflows.exportFile"))));
    expect(exportBtn).toBeTruthy();
    await exportBtn!.trigger("click");
    await flushPromises();
    expect(saveMock).toHaveBeenCalled();
    expect(writeTextFileMock).toHaveBeenCalled();
    const [, content] = writeTextFileMock.mock.calls[0]!;
    const parsed = JSON.parse(String(content)) as {
      graph: ExpertGraph;
      name: string;
    };
    expect(parsed.name).toBe("mywf");
    expect(parsed.graph.nodes[0]).toMatchObject({ type: "base_model", ggufPath: "/models/base.gguf" });
    w.unmount();
  });

  it("shows import preview for valid .oclexpert JSON then can cancel", async () => {
    const wrapped = {
      format: "oclexpert",
      fileVersion: 1,
      name: "imp",
      graph: {
        version: 1,
        nodes: [{ type: "base_model", id: "b", ggufPath: "/g.gguf", ui: null }],
        edges: [],
      },
      promptStyle: null,
    };
    openMock.mockResolvedValue("/tmp/in.json");
    readTextFileMock.mockResolvedValue(JSON.stringify(wrapped));

    const w = mountExpertPanel();
    await flushPromises();
    const importBtn = w
      .findAll("button")
      .find((b) => b.text().includes(String(i18n.global.t("expertModels.oclexpert.import"))));
    expect(importBtn).toBeTruthy();
    await importBtn!.trigger("click");
    await flushPromises();
    expect(w.html()).toContain(String(i18n.global.t("expertModels.oclexpert.previewTitle")));

    const cancelBtn = w
      .findAll("button")
      .find((b) => b.text().includes(String(i18n.global.t("expertModels.oclexpert.previewCancel"))));
    expect(cancelBtn).toBeTruthy();
    await cancelBtn!.trigger("click");
    await flushPromises();
    w.unmount();
  });

  it("adds and removes a LoRA via form select and remove button", async () => {
    const w = mountExpertPanel();
    await flushPromises();
    const selects = w.findAll(".em-select");
    const loraSelect = selects.find((s) => {
      const opts = s.findAll("option");
      return opts.some((o) => o.attributes("value") === "/stub/lora.gguf");
    });
    expect(loraSelect).toBeTruthy();
    await loraSelect!.setValue("/stub/lora.gguf");
    await flushPromises();
    const removeBtns = w.findAll("button.em-mini.danger");
    expect(removeBtns.length).toBeGreaterThan(0);
    await removeBtns[0]!.trigger("click");
    await flushPromises();
    w.unmount();
  });
});
