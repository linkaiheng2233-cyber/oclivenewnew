import { storeToRefs } from "pinia";
import { computed, ref, toValue, watch, type MaybeRefOrGetter } from "vue";
import type { PluginUiSlotInfo } from "../utils/tauri-api";
import { PluginVueCompileError } from "../utils/compilePluginVueSfc";
import { useKeyedPluginErrors } from "./usePluginError";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";

/**
 * 目录插件「嵌入插槽」共用逻辑：从 `pluginStore.bootstrapUiSlots` 过滤、Vue/iframe 回退、iframe 错误文案。
 * 错误状态由 {@link useKeyedPluginErrors} 统一管理。
 */
export function useDirectoryPluginSlotEmbed(options: {
  slot: MaybeRefOrGetter<string>;
  /** 与插件保存/刷新联动（如 `pluginStore.bootstrapEpoch`） */
  bootstrapEpoch: MaybeRefOrGetter<number>;
}) {
  const roleStore = useRoleStore();
  const { currentRoleId } = storeToRefs(roleStore);
  const pluginStore = usePluginStore();
  const { error: pluginError, bootstrapUiSlots } = storeToRefs(pluginStore);

  const {
    messages: frameErrors,
    details: frameErrorDetails,
    clearAll: clearAllKeyedErrors,
    clearKey: clearKeyedError,
    setKey: setKeyedError,
  } = useKeyedPluginErrors();

  const slots = computed<PluginUiSlotInfo[]>(() =>
    (bootstrapUiSlots.value ?? []).filter((s) => s.slot === toValue(options.slot)),
  );

  const vueFallback = ref<Record<string, boolean>>({});
  /** 递增以强制重挂 iframe / Vue */
  const reloadEpoch = ref<Record<string, number>>({});

  watch(
    () =>
      [toValue(options.bootstrapEpoch), currentRoleId.value, bootstrapUiSlots.value] as const,
    () => {
      vueFallback.value = {};
      clearAllKeyedErrors();
      reloadEpoch.value = {};
    },
  );

  function onFrameError(pluginId: string): void {
    setKeyedError(pluginId, "页面加载失败");
  }

  function onFrameLoad(pluginId: string): void {
    if (!frameErrors.value[pluginId] && !frameErrorDetails.value[pluginId]) {
      return;
    }
    clearKeyedError(pluginId);
  }

  function onVueFailed(pluginId: string): void {
    vueFallback.value = { ...vueFallback.value, [pluginId]: true };
    if (!frameErrors.value[pluginId]) {
      setKeyedError(pluginId, "Vue 组件加载失败，已尝试 iframe 回退");
    }
  }

  function onVueCompileError(pluginId: string, err: PluginVueCompileError): void {
    setKeyedError(pluginId, err.friendlyMessage, err.rawMessage);
  }

  /** 重置错误状态并重新加载该插槽条目（Vue / iframe）。 */
  function retrySlot(s: PluginUiSlotInfo): void {
    const id = s.pluginId;
    reloadEpoch.value = {
      ...reloadEpoch.value,
      [id]: (reloadEpoch.value[id] ?? 0) + 1,
    };
    clearKeyedError(id);
    vueFallback.value = { ...vueFallback.value, [id]: false };
  }

  function reloadNonceFor(pluginId: string): number {
    return reloadEpoch.value[pluginId] ?? 0;
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
    frameErrorDetails,
    reloadNonceFor,
    onFrameError,
    onFrameLoad,
    onVueFailed,
    onVueCompileError,
    retrySlot,
    showIframe,
    showVue,
  };
}
