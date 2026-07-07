import type { MaybeRefOrGetter } from 'vue'
import type { PluginVueCompileError } from '@oclive/shared/utils/compilePluginVueSfc'
import type { PluginUiSlotInfo } from '@oclive/shared/api'
import { storeToRefs } from 'pinia'
import { computed, ref, toValue, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useKeyedPluginErrors } from '@oclive/shared/composables/usePluginError'

/**
 * Shared directory-plugin slot embed logic: filter from `pluginStore.bootstrapUiSlots`, Vue/iframe fallback, iframe error copy.
 * Error state is centralized via {@link useKeyedPluginErrors}.
 */
export function useDirectoryPluginSlotEmbed(options: {
  slot: MaybeRefOrGetter<string>
  /** Tied to plugin save/refresh (e.g. `pluginStore.bootstrapEpoch`). */
  bootstrapEpoch: MaybeRefOrGetter<number>
  /** When set, only embed slots from these plugin ids (e.g. pure_chat platform plugins). */
  pluginIdAllowlist?: MaybeRefOrGetter<readonly string[] | null | undefined>
}) {
  const { t } = useI18n()
  const roleStore = useRoleStore()
  const { currentRoleId } = storeToRefs(roleStore)
  const pluginStore = usePluginStore()
  const { error: pluginError, bootstrapUiSlots } = storeToRefs(pluginStore)

  const {
    messages: frameErrors,
    details: frameErrorDetails,
    clearAll: clearAllKeyedErrors,
    clearKey: clearKeyedError,
    setKey: setKeyedError,
  } = useKeyedPluginErrors()

  const slots = computed<PluginUiSlotInfo[]>(() => {
    const allowlist = toValue(options.pluginIdAllowlist)
    return (bootstrapUiSlots.value ?? []).filter((s) => {
      if (s.slot !== toValue(options.slot))
        return false
      if (!allowlist || allowlist.length === 0)
        return true
      return allowlist.includes(s.pluginId)
    })
  })

  const vueFallback = ref<Record<string, boolean>>({})
  /** Increment to force remount of iframe / Vue. */
  const reloadEpoch = ref<Record<string, number>>({})

  watch(
    () =>
      [toValue(options.bootstrapEpoch), currentRoleId?.value ?? '', bootstrapUiSlots.value] as const,
    () => {
      vueFallback.value = {}
      clearAllKeyedErrors()
      reloadEpoch.value = {}
    },
  )

  function onFrameError(pluginId: string): void {
    setKeyedError(pluginId, t('pluginWorkbench.slotEmbed.frameLoadFailed'))
  }

  function onFrameLoad(pluginId: string): void {
    if (!frameErrors.value[pluginId] && !frameErrorDetails.value[pluginId]) {
      return
    }
    clearKeyedError(pluginId)
  }

  function onVueFailed(pluginId: string): void {
    vueFallback.value = { ...vueFallback.value, [pluginId]: true }
    if (!frameErrors.value[pluginId]) {
      setKeyedError(pluginId, t('pluginWorkbench.slotEmbed.vueIframeFallback'))
    }
  }

  function onVueCompileError(pluginId: string, err: PluginVueCompileError): void {
    setKeyedError(pluginId, err.friendlyMessage, err.rawMessage)
  }

  /** Reset error state and reload this slot entry (Vue / iframe). */
  function retrySlot(s: PluginUiSlotInfo): void {
    const id = s.pluginId
    reloadEpoch.value = {
      ...reloadEpoch.value,
      [id]: (reloadEpoch.value[id] ?? 0) + 1,
    }
    clearKeyedError(id)
    vueFallback.value = { ...vueFallback.value, [id]: false }
  }

  function reloadNonceFor(pluginId: string): number {
    return reloadEpoch.value[pluginId] ?? 0
  }

  function showIframe(s: PluginUiSlotInfo): boolean {
    if (pluginStore.pluginState.force_iframe_mode)
      return true
    const vc = s.vueComponent?.trim()
    if (!vc)
      return true
    return vueFallback.value[s.pluginId] === true
  }

  function showVue(s: PluginUiSlotInfo): boolean {
    if (pluginStore.pluginState.force_iframe_mode)
      return false
    const vc = s.vueComponent?.trim()
    if (!vc)
      return false
    return vueFallback.value[s.pluginId] !== true
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
  }
}
