import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "./useAppToast";
import { hostEventBus } from "../lib/hostEventBus";
import { CLOUD_LLM_PRESET_DEFAULTS, CLOUD_LLM_PRESET_ORDER } from "../lib/cloudLlmPresets";
import type { HostCloudLlmPublicDto } from "../utils/tauri-api";
import {
  getHostChatModel,
  getHostCloudLlmPublic,
  ollamaModelsHealth,
  ollamaModelsListNames,
  setHostChatModel,
} from "../utils/tauri-api";

export const HOST_CHAT_MODEL_CUSTOM_SENTINEL = "__oclive_custom_model__";

type PickSingleton = ReturnType<typeof createPickState>;

let singleton: PickSingleton | null = null;
let watchStarted = false;

function createPickState() {
  const { t } = useI18n();
  const { showToast } = useAppToast();

  const modelId = ref("");
  const lastSaved = ref("");
  const ollamaNames = ref<string[]>([]);
  const cloudPub = ref(null as HostCloudLlmPublicDto | null);
  const useCustomModel = ref(false);
  const selectModel = ref("");
  const customInputEl = ref<HTMLInputElement | null>(null);

  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  const cloudSelectOptions = computed(() => {
    const s = new Set<string>();
    for (const pid of CLOUD_LLM_PRESET_ORDER) {
      if (pid === "custom") continue;
      s.add(CLOUD_LLM_PRESET_DEFAULTS[pid].model);
    }
    const saved = cloudPub.value?.model?.trim();
    if (saved) s.add(saved);
    const local = new Set(ollamaNames.value);
    return [...s].filter((m) => !local.has(m)).sort((a, b) => a.localeCompare(b));
  });

  function syncSelectFromModel(): void {
    const m = modelId.value.trim();
    if (!m) {
      selectModel.value = "";
      useCustomModel.value = false;
      return;
    }
    if (ollamaNames.value.includes(m)) {
      useCustomModel.value = false;
      selectModel.value = m;
      return;
    }
    if (cloudSelectOptions.value.includes(m)) {
      useCustomModel.value = false;
      selectModel.value = m;
      return;
    }
    useCustomModel.value = true;
    selectModel.value = HOST_CHAT_MODEL_CUSTOM_SENTINEL;
  }

  function schedulePersistCustom(): void {
    if (saveTimer != null) {
      window.clearTimeout(saveTimer);
      saveTimer = null;
    }
    saveTimer = window.setTimeout(() => {
      saveTimer = null;
      void persistModel();
    }, 400);
  }

  async function persistModel(): Promise<void> {
    const m = modelId.value.trim();
    if (!m) {
      showToast("error", String(t("chatComposer.errEmpty")));
      modelId.value = lastSaved.value;
      syncSelectFromModel();
      return;
    }
    if (m === lastSaved.value) return;
    try {
      await setHostChatModel(m);
      lastSaved.value = m;
      syncSelectFromModel();
      hostEventBus.emitBuiltin("host:host_chat_model_updated");
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
      modelId.value = lastSaved.value;
      syncSelectFromModel();
    }
  }

  async function loadOllama(): Promise<void> {
    try {
      const ok = await ollamaModelsHealth();
      if (!ok) {
        ollamaNames.value = [];
        return;
      }
      ollamaNames.value = await ollamaModelsListNames();
    } catch {
      ollamaNames.value = [];
    }
  }

  async function loadCloudPublic(): Promise<void> {
    try {
      cloudPub.value = await getHostCloudLlmPublic();
    } catch {
      cloudPub.value = null;
    }
  }

  async function bootstrap(): Promise<void> {
    try {
      const cur = await getHostChatModel();
      modelId.value = cur.trim();
      lastSaved.value = modelId.value;
    } catch {
      modelId.value = "";
    }
    await Promise.all([loadOllama(), loadCloudPublic()]);
    syncSelectFromModel();
  }

  function onWindowFocus(): void {
    void Promise.all([loadOllama(), loadCloudPublic()]).then(() => {
      syncSelectFromModel();
    });
  }

  function onSelectModel(e: Event): void {
    const el = e.target as HTMLSelectElement;
    const v = el.value;
    if (v === HOST_CHAT_MODEL_CUSTOM_SENTINEL) {
      useCustomModel.value = true;
      selectModel.value = HOST_CHAT_MODEL_CUSTOM_SENTINEL;
      void nextTick(() => {
        customInputEl.value?.focus();
        customInputEl.value?.select();
      });
      return;
    }
    useCustomModel.value = false;
    selectModel.value = v;
    modelId.value = v;
    void persistModel();
  }

  function onCustomModelInput(): void {
    schedulePersistCustom();
  }

  function onCustomModelBlur(): void {
    void persistModel();
  }

  function startWatchAndFocus(): void {
    if (watchStarted) return;
    watchStarted = true;
    watch([ollamaNames, cloudPub], () => {
      syncSelectFromModel();
    });
    if (typeof window !== "undefined") {
      window.addEventListener("focus", onWindowFocus);
    }
  }

  return {
    modelId,
    lastSaved,
    ollamaNames,
    cloudPub,
    useCustomModel,
    selectModel,
    customInputEl,
    cloudSelectOptions,
    CUSTOM_SENTINEL: HOST_CHAT_MODEL_CUSTOM_SENTINEL,
    bootstrap,
    loadOllama,
    loadCloudPublic,
    syncSelectFromModel,
    persistModel,
    onSelectModel,
    onCustomModelInput,
    onCustomModelBlur,
    startWatchAndFocus,
  };
}

/** 全局对话模型 id 的列表与持久化（单例，撰写区与纯聊浮层共用）。 */
export function useHostModelPick(): PickSingleton {
  if (!singleton) {
    singleton = createPickState();
  }
  singleton.startWatchAndFocus();
  return singleton;
}
