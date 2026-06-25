import { useI18n } from 'vue-i18n'
import { setRoleInteractionMode } from '@oclive/shared/api'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useNarrativeScene } from '@oclive/shared/composables/useNarrativeScene'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import type { InteractionMode } from '@oclive/shared/utils/interactionMode'
import { PURE_CHAT_DEFAULT_SCENE_ID } from '@oclive/shared/utils/pureChatScene'

/** Persist interaction mode + reset daily-chat scene to home. */
export function useInteractionModeSettings() {
  const roleStore = useRoleStore()
  const uiStore = useUiStore()
  const { showToast } = useAppToast()
  const { t } = useI18n()
  const { applyResolvedNarrativeScene } = useNarrativeScene()

  function applyPureChatSceneIsolation(): void {
    if (uiStore.sceneId !== PURE_CHAT_DEFAULT_SCENE_ID)
      uiStore.setScene(PURE_CHAT_DEFAULT_SCENE_ID)
  }

  async function setInteractionMode(mode: InteractionMode): Promise<void> {
    const roleId = roleStore.currentRoleId
    if (!roleId)
      return
    const info = await setRoleInteractionMode(roleId, mode)
    roleStore.applyRoleInfo(info)
    if (mode === 'pure_chat')
      applyPureChatSceneIsolation()
    else
      applyResolvedNarrativeScene()
    showToast(
      'info',
      mode === 'pure_chat'
        ? t('app.toast.interactionPureChat')
        : t('app.toast.interactionImmersive'),
    )
  }

  async function onInteractionModeSelect(ev: Event): Promise<void> {
    const value = (ev.target as HTMLSelectElement).value as InteractionMode
    if (value !== 'immersive' && value !== 'pure_chat')
      return
    if (value === roleStore.roleInfo.interactionMode)
      return
    try {
      await setInteractionMode(value)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  return {
    applyPureChatSceneIsolation,
    setInteractionMode,
    onInteractionModeSelect,
  }
}
