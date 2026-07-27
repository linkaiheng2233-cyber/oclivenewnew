import type { ToastType } from '@oclive/shared/composables/useAppToast'
import { setUserPresenceScene, switchScene } from '@oclive/shared/api'
import { useAdultInteractionStore } from '@oclive/shared/stores/adultInteractionStore'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useDebugStore } from '@oclive/shared/stores/debugStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const SCENE_TRANSITION_MS = 520

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => window.setTimeout(resolve, ms))
}

export type ShowToast = (type: ToastType, message: string) => void

/**
 * Top-bar / travel-bar "Go": co-present uses `switchScene`; narrative-only uses `setUserPresenceScene`.
 */
export function useSceneDestination(showToast: ShowToast) {
  const { t } = useI18n()
  const roleStore = useRoleStore()
  const chatStore = useChatStore()
  const adultStore = useAdultInteractionStore()
  const uiStore = useUiStore()
  const debugStore = useDebugStore()

  const sceneTransition = ref({ visible: false, label: '' })

  function sceneLabelForId(sceneId: string): string {
    const row = roleStore.roleInfo.sceneLabels?.find(s => s.id === sceneId)
    return row?.label ?? sceneId
  }

  function characterSceneLabel(): string {
    const id = roleStore.roleInfo.currentScene
    if (!id)
      return '—'
    return sceneLabelForId(id)
  }

  async function applySceneDestination(id: string, together: boolean): Promise<void> {
    if (!roleStore.interactionImmersive)
      return
    if (!id.trim()) {
      showToast('warning', t('app.scene.selectDestinationFirst'))
      return
    }
    const label = sceneLabelForId(id)
    const previousSceneId = uiStore.sceneId || 'default'
    if (together) {
      sceneTransition.value = { visible: true, label }
    }
    try {
      if (adultStore.sessionFor(roleStore.currentRoleId, previousSceneId).active) {
        await chatStore.sendAdultAction(
          'exit',
          previousSceneId,
          `用户即将进入“${label}”场景。请自然结束旧场景中的当前互动，并按照角色人设对这次场景变化简短说一句。`,
        )
      }
      if (together) {
        const res = await switchScene(roleStore.currentRoleId, id, true)
        await sleep(SCENE_TRANSITION_MS)
        sceneTransition.value = { visible: false, label: '' }
        roleStore.applyRoleInfo(res)
        const narrative = res.user_presence_scene ?? id
        chatStore.applySceneChange(narrative)
        if (res.scene_welcome) {
          chatStore.addSystemMessage(res.scene_welcome, narrative)
        }
        showToast('success', t('app.scene.toastTogether'))
      }
      else {
        const info = await setUserPresenceScene(roleStore.currentRoleId, id)
        roleStore.applyRoleInfo(info)
        const narrative = info.user_presence_scene ?? id
        chatStore.applySceneChange(narrative)
        chatStore.addSystemMessage(
          t('app.scene.systemLine', {
            narrative: label,
            character: characterSceneLabel(),
          }),
          narrative,
        )
        showToast('success', t('app.scene.toastNarrativeOnly'))
      }
      await debugStore.loadDebugData()
    }
    catch (err) {
      sceneTransition.value = { visible: false, label: '' }
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  return {
    sceneTransition,
    applySceneDestination,
    sceneLabelForId,
    characterSceneLabel,
  }
}
