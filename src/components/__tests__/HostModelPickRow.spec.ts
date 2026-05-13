import { mount, flushPromises } from "@vue/test-utils";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { i18n, prepareI18nForLocale } from "../../i18n";
import HostModelPickRow from "../HostModelPickRow.vue";

const setHostChatModel = vi.fn().mockResolvedValue(undefined);

vi.mock("../../utils/tauri-api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("../../utils/tauri-api")>();
  return {
    ...mod,
    getHostChatModel: vi.fn(),
    ollamaModelsHealth: vi.fn(),
    ollamaModelsListNames: vi.fn(),
    getHostCloudLlmPublic: vi.fn(),
    setHostChatModel: (...args: unknown[]) => setHostChatModel(...args),
    probeLocalLlmRuntime: vi.fn().mockResolvedValue({ ollamaProcess: false, llamaLikeProcess: false }),
  };
});

import {
  getHostChatModel,
  getHostCloudLlmPublic,
  ollamaModelsHealth,
  ollamaModelsListNames,
} from "../../utils/tauri-api";

describe("HostModelPickRow", () => {
  beforeAll(async () => {
    await prepareI18nForLocale("en-US");
  });

  beforeEach(() => {
    setHostChatModel.mockClear();
    vi.mocked(getHostChatModel).mockReset();
    vi.mocked(ollamaModelsHealth).mockReset();
    vi.mocked(ollamaModelsListNames).mockReset();
    vi.mocked(getHostCloudLlmPublic).mockReset();
  });

  it("shows offline placeholder when local model list is empty", async () => {
    vi.mocked(getHostChatModel).mockResolvedValue("");
    vi.mocked(ollamaModelsHealth).mockResolvedValue(false);
    vi.mocked(getHostCloudLlmPublic).mockResolvedValue(null);

    const w = mount(HostModelPickRow, {
      global: { plugins: [i18n] },
    });
    await flushPromises();
    const opt = w.find("select option[disabled]");
    expect(opt.exists()).toBe(true);
    expect(opt.attributes("value")).toBe("__none__");
    w.unmount();
  });

  it("renders cloud optgroup only when cloud is configured with url and key", async () => {
    vi.mocked(getHostChatModel).mockResolvedValue("llama3");
    vi.mocked(ollamaModelsHealth).mockResolvedValue(true);
    vi.mocked(ollamaModelsListNames).mockResolvedValue(["llama3"]);
    vi.mocked(getHostCloudLlmPublic).mockResolvedValue({
      baseUrl: "https://api.example/v1",
      hasApiKey: true,
      model: "cloud-model-a",
    } as never);

    const w = mount(HostModelPickRow, {
      global: { plugins: [i18n] },
    });
    await flushPromises();
    const groups = w.findAll("select optgroup");
    const cloudLabels = groups.map((g) => g.attributes("label"));
    expect(cloudLabels.some((l) => l?.includes("Cloud") || l?.includes("云"))).toBe(true);
    w.unmount();
  });

  it("omits cloud optgroup when baseUrl or api key missing", async () => {
    vi.mocked(getHostChatModel).mockResolvedValue("m1");
    vi.mocked(ollamaModelsHealth).mockResolvedValue(true);
    vi.mocked(ollamaModelsListNames).mockResolvedValue(["m1"]);
    vi.mocked(getHostCloudLlmPublic).mockResolvedValue({
      baseUrl: "",
      hasApiKey: true,
      model: "x",
    } as never);

    const w = mount(HostModelPickRow, {
      global: { plugins: [i18n] },
    });
    await flushPromises();
    const groups = w.findAll("select optgroup");
    expect(groups.length).toBe(2);
    w.unmount();
  });

  it("always lists custom sentinel option in select", async () => {
    vi.mocked(getHostChatModel).mockResolvedValue("m1");
    vi.mocked(ollamaModelsHealth).mockResolvedValue(true);
    vi.mocked(ollamaModelsListNames).mockResolvedValue(["m1"]);
    vi.mocked(getHostCloudLlmPublic).mockResolvedValue(null);

    const w = mount(HostModelPickRow, {
      global: { plugins: [i18n] },
    });
    await flushPromises();
    const customOpt = w.findAll("select option").find((o) => o.attributes("value") === "__oclive_custom_model__");
    expect(customOpt?.exists()).toBe(true);
    w.unmount();
  });

  it("persists when switching to another local model", async () => {
    vi.mocked(getHostChatModel).mockResolvedValue("m1");
    vi.mocked(ollamaModelsHealth).mockResolvedValue(true);
    vi.mocked(ollamaModelsListNames).mockResolvedValue(["m1", "m2"]);
    vi.mocked(getHostCloudLlmPublic).mockResolvedValue(null);

    const w = mount(HostModelPickRow, {
      global: { plugins: [i18n] },
    });
    await flushPromises();
    const sel = w.find("select");
    await sel.setValue("m2");
    await flushPromises();
    expect(setHostChatModel).toHaveBeenCalledWith("m2");
    w.unmount();
  });
});
