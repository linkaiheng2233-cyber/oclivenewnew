import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import { onBeforeUnmount, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { setRemoteLifeEnabled, setRoleInteractionMode } from '@oclive/shared/api'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { resetLayoutWidths } from '@oclive/shared/composables/useLayoutWidths'
import { useOcliveAppearance } from '@oclive/shared/composables/useOcliveAppearance'

const quickActionTravelEvent = 'com.oclive.mumu.quick-actions:travel'
const settingsSetRemoteLifeEvent = 'com.oclive.mumu.settings-panel:set_remote_life'
const settingsSetInteractionModeEvent
  = 'com.oclive.mumu.settings-panel:set_interaction_mode'
const settingsCycleThemeEvent = 'com.oclive.mumu.settings-panel:cycle_theme'
const settingsResetLayoutEvent = 'com.oclive.mumu.settings-panel:request_reset_layout'
const settingsResetLayoutResultEvent = 'com.oclive.mumu.settings-panel:reset_layout_result'

export interface UsePluginEventsOptions {
  showToast: AppToastFn
  onQuickActionTravel: (payload: unknown) => void
  onPureChatMode: () => void
}

export function usePluginEvents(opts: UsePluginEventsOptions) {
  const { t } = useI18n()
  const roleStore = useRoleStore()
  const pluginStore = usePluginStore()
  const { cycleTheme } = useOcliveAppearance()

  async function onPluginSetRemoteLife(payload: unknown): Promise<void> {
    if (!roleStore.interactionImmersive) {
      opts.showToast('info', t('app.toast.interactionPureChat'))
      return
    }
    const enabledRaw = (payload as { enabled?: boolean } | null)?.enabled
    if (typeof enabledRaw !== 'boolean')
      return
    try {
      const info = await setRemoteLifeEnabled(roleStore.currentRoleId, enabledRaw)
      roleStore.applyRoleInfo(info)
      opts.showToast('success', enabledRaw ? t('app.toast.remoteLifeOn') : t('app.toast.remoteLifeOff'))
    }
    catch (err) {
      opts.showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  async function onPluginSetInteractionMode(payload: unknown): Promise<void> {
    // Plugin programming entry (`com.oclive.mumu.settings-panel:set_interaction_mode`); not a user IA surface.
    const mode = (payload as { mode?: string } | null)?.mode
    if (mode !== 'immersive' && mode !== 'pure_chat')
      return
    try {
      const info = await setRoleInteractionMode(roleStore.currentRoleId, mode)
      roleStore.applyRoleInfo(info)
      if (mode === 'pure_chat')
        opts.onPureChatMode()
      opts.showToast(
        'success',
        mode === 'immersive'
          ? t('app.toast.interactionImmersive')
          : t('app.toast.interactionPureChat'),
      )
    }
    catch (err) {
      opts.showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  function onPluginCycleTheme(): void {
    cycleTheme()
  }

  async function onPluginResetLayout(): Promise<void> {
    try {
      resetLayoutWidths()
      await pluginStore.resetToRolePackDefault()
      const message = t('app.toast.layoutResetOk')
      hostEventBus.emit(settingsResetLayoutResultEvent, { ok: true, message })
      opts.showToast('success', message)
    }
    catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      hostEventBus.emit(settingsResetLayoutResultEvent, {
        ok: false,
        message: t('app.toast.layoutResetFailPrefix') + message,
      })
      opts.showToast('error', message)
    }
  }

  onMounted(() => {
    hostEventBus.on(quickActionTravelEvent, opts.onQuickActionTravel)
    hostEventBus.on(settingsSetRemoteLifeEvent, onPluginSetRemoteLife)
    hostEventBus.on(settingsSetInteractionModeEvent, onPluginSetInteractionMode)
    hostEventBus.on(settingsCycleThemeEvent, onPluginCycleTheme)
    hostEventBus.on(settingsResetLayoutEvent, onPluginResetLayout)
  })

  onBeforeUnmount(() => {
    hostEventBus.off(quickActionTravelEvent, opts.onQuickActionTravel)
    hostEventBus.off(settingsSetRemoteLifeEvent, onPluginSetRemoteLife)
    hostEventBus.off(settingsSetInteractionModeEvent, onPluginSetInteractionMode)
    hostEventBus.off(settingsCycleThemeEvent, onPluginCycleTheme)
    hostEventBus.off(settingsResetLayoutEvent, onPluginResetLayout)
  })

  return {
    onPluginSetRemoteLife,
    onPluginSetInteractionMode,
    onPluginCycleTheme,
    onPluginResetLayout,
  }
}
