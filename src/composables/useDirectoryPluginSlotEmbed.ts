import { storeToRefs } from "pinia";
import { computed, ref, toValue, watch, type MaybeRefOrGetter } from "vue";
import type { PluginUiSlotInfo } from "../utils/tauri-api";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";

/**
 * 目录插件「嵌入插槽」共用逻辑：从 `pluginStore.bootstrapUiSlots` 过滤、Vue/iframe 回退、iframe 错误文案。
 */
export function useDirectoryPluginSlotEmbed(options: {
  slot: string;
  /** 与插件保存/刷新联动（如 `pluginStore.bootstrapEpoch`） */
  bootstrapEpoch: MaybeRefOrGetter<number>;
}) {
  const roleStore = useRoleStore();
  const { currentRoleId } = storeToRefs(roleStore);
  const pluginStore = usePluginStore();
  const { error: pluginError, bootstrapUiSlots } = storeToRefs(pluginStore);

  const slots = computed<PluginUiSlotInfo[]>(() =>
    (bootstrapUiSlots.value ?? []).filter((s) => s.slot === options.slot),
  );

  const frameErrors = ref<Record<string, string>>({});
  const vueFallback = ref<Record<string, boolean>>({});

  watch(
    () =>
      [toValue(options.bootstrapEpoch), currentRoleId.value, bootstrapUiSlots.value] as const,
    () => {
      vueFallback.value = {};
      frameErrors.value = {};
    },
  );

  function onFrameError(pluginId: string): void {
    frameErrors.value = { ...frameErrors.value, [pluginId]: "页面加载失败" };
  }

  function onVueFailed(pluginId: string): void {
    vueFallback.value = { ...vueFallback.value, [pluginId]: true };
  }

  function showIframe(s: PluginUiSlotInfo): boolean {
    if (pluginStore.pluginState.force_iframe_mode) return true;
    const vc = s.vueComponent?.trim();
    if (!vc) return true;
    return vueFallback.value[s.pluginId] === true;
  }

  function showVue(s: PluginUiSlotInfo): boolean {
    if (pluginStore.pluginState.force_iframe_mode) return false;
    const vc = s.vueComponent?.trim();
    if (!vc) return false;
    return vueFallback.value[s.pluginId] !== true;
  }

  return {
    pluginError,
    slots,
    frameErrors,
    onFrameError,
    onVueFailed,
    showIframe,
    showVue,
  };
}
