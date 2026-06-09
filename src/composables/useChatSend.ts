import type { Ref } from 'vue'
import type { ComposerTranslation } from 'vue-i18n'
import { useChatStore } from '../stores/chatStore'
import { useDebugStore } from '../stores/debugStore'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'
import { useNarrativeScene } from './useNarrativeScene'
import type { AppToastFn } from './useAppToast'

export function useChatSend(options: {
  showToast: AppToastFn
  t: ComposerTranslation
  chatInputRef: Ref<{ focusInput?: () => void } | null>
  clearSceneBarsBeforeSend: () => void
  offerSceneBarsAfterReply: (together: boolean, destination: boolean) => void
  onTurnRecorded?: (userMessage: string) => void
}) {
  const chatStore = useChatStore()
  const roleStore = useRoleStore()
  const uiStore = useUiStore()
  const debugStore = useDebugStore()
  const { applyResolvedNarrativeScene } = useNarrativeScene()

  async function onSend(payload: { content: string }) {
    options.clearSceneBarsBeforeSend()
    const userText = payload.content
    try {
      const res = await chatStore.sendMessage(userText, uiStore.sceneId)
      await roleStore.refreshRoleInfo()
      applyResolvedNarrativeScene()
      await debugStore.loadDebugData()
      if (res.reply_is_fallback) {
        const detail = res.llm_fallback_reason?.trim()
        options.showToast('info', detail || options.t('app.toast.fallbackReply'))
      }
      if (res.chat_persist_failed) {
        const detail = res.chat_persist_error?.trim()
        options.showToast(
          'warning',
          detail || options.t('app.toast.chatPersistFailed'),
        )
      }
      options.offerSceneBarsAfterReply(
        res.offer_together_travel ?? false,
        res.offer_destination_picker ?? false,
      )
      options.onTurnRecorded?.(userText)
    }
    catch (err) {
      options.showToast('error', err instanceof Error ? err.message : String(err))
    }
    finally {
      options.chatInputRef.value?.focusInput?.()
    }
  }

  return { onSend }
}
