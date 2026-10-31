import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import type { Ref } from 'vue'
import type { ComposerTranslation } from 'vue-i18n'
import { toErrorPresentation } from '@oclive/shared/api/helpers'
import { useNarrativeScene } from '@oclive/shared/composables/useNarrativeScene'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useDebugStore } from '@oclive/shared/stores/debugStore'
import { useMinimalRoleChatStore } from '@oclive/shared/stores/minimalRoleChatStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { effectiveChatSceneId } from '@oclive/shared/utils/pureChatScene'

export function useChatSend(options: {
  showToast: AppToastFn
  t: ComposerTranslation
  chatInputRef: Ref<{ focusInput?: () => void } | null>
  clearSceneBarsBeforeSend: () => void
  offerSceneBarsAfterReply: (together: boolean, destination: boolean) => void
  onTurnRecorded?: (userText: string) => void
  isContextChanging?: () => boolean
  debugDataEnabled?: () => boolean
}) {
  const chatStore = useChatStore()
  const minimal = useMinimalRoleChatStore()
  const roleStore = useRoleStore()
  const uiStore = useUiStore()
  const debugStore = useDebugStore()
  const { applyResolvedNarrativeScene } = useNarrativeScene()

  async function onSend(payload: { content: string }) {
    if (options.isContextChanging?.())
      return
    options.clearSceneBarsBeforeSend()
    const userText = payload.content
    try {
      if (roleStore.minimalRoleActive) {
        await minimal.sendMessage(userText)
        return { status: 'sent' as const }
      }
      const sceneId = effectiveChatSceneId(
        roleStore.roleInfo.interactionMode,
        uiStore.sceneId,
      )
      const res = await chatStore.sendMessage(userText, sceneId)
      if (!res)
        return
      try {
        await roleStore.refreshRoleInfo()
        applyResolvedNarrativeScene()
        if (options.debugDataEnabled?.() !== false)
          await debugStore.loadDebugData()
      }
      catch (err) {
        const failure = toErrorPresentation(err, options.t('errorUi.refreshAfterReply'))
        options.showToast('warning', failure.summary, failure.details)
      }
      if (res.reply_is_fallback) {
        const detail = res.llm_fallback_reason?.trim()
        const failure = toErrorPresentation(detail, options.t('app.toast.fallbackReply'))
        options.showToast('info', failure.summary, failure.details)
      }
      if (res.chat_persist_failed) {
        const detail = res.chat_persist_error?.trim()
        const failure = toErrorPresentation(detail, options.t('app.toast.chatPersistFailed'))
        options.showToast('warning', failure.summary, failure.details)
      }
      options.offerSceneBarsAfterReply(
        res.offer_together_travel ?? false,
        res.offer_destination_picker ?? false,
      )
      options.onTurnRecorded?.(userText)
      return { status: 'sent' as const }
    }
    catch (err) {
      const failure = toErrorPresentation(err)
      options.showToast('error', failure.summary, failure.details)
      return { status: 'failed' as const, error: err }
    }
    finally {
      options.chatInputRef.value?.focusInput?.()
    }
  }

  async function onAdultAction(payload: { action: 'exit' }) {
    if (options.isContextChanging?.() || roleStore.minimalRoleActive)
      return
    options.clearSceneBarsBeforeSend()
    try {
      const sceneId = effectiveChatSceneId(
        roleStore.roleInfo.interactionMode,
        uiStore.sceneId,
      )
      const res = await chatStore.sendAdultAction(payload.action, sceneId)
      if (!res)
        return
      await roleStore.refreshRoleInfo()
      if (options.debugDataEnabled?.() !== false)
        await debugStore.loadDebugData()
    }
    catch (err) {
      const failure = toErrorPresentation(err)
      options.showToast('error', failure.summary, failure.details)
    }
    finally {
      options.chatInputRef.value?.focusInput?.()
    }
  }

  return { onSend, onAdultAction }
}
