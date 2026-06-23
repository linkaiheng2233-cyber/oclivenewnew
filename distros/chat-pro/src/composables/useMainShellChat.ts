import type ChatMessageList from '@oclive/shared/components/chat/ChatMessageList.vue'
import type { ComposerTranslation } from 'vue-i18n'
import { computed, ref } from 'vue'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { useChatSend } from '@oclive/shared/composables/useChatSend'
import type { AppToastFn } from '@oclive/shared/composables/useAppToast'

export function useMainShellChat(options: {
  roleStore: ReturnType<typeof useRoleStore>
  uiStore: ReturnType<typeof useUiStore>
  showToast: AppToastFn
  t: ComposerTranslation
  clearSceneBarsBeforeSend: () => void
  offerSceneBarsAfterReply: (together: boolean, destination: boolean) => void
  onTurnRecorded: (userMessage: string) => void
}) {
  const chatStore = useChatStore()
  const chatListRef = ref<InstanceType<typeof ChatMessageList> | null>(null)
  const chatInputRef = ref<{ focusInput?: () => void } | null>(null)

  const messages = computed(() =>
    chatStore.messagesForRoleScene(options.roleStore.currentRoleId, options.uiStore.sceneId),
  )

  const chatListLoading = computed(() =>
    chatStore.isLoading
    || chatStore.isMessagesLoadingFor(options.roleStore.currentRoleId, options.uiStore.sceneId),
  )

  const latestRoleplayAside = computed(() => {
    const roleId = options.roleStore.currentRoleId
    const sceneId = options.uiStore.sceneId || 'default'
    return chatStore.lastAssistantAsideFor(roleId, sceneId)
  })

  const sceneHistorySplitIndex = computed(() => {
    if (!options.roleStore.interactionImmersive)
      return 0
    return chatStore.sceneHistorySplitForRoleScene(options.roleStore.currentRoleId, options.uiStore.sceneId)
  })

  const { onSend } = useChatSend({
    showToast: options.showToast,
    t: options.t,
    chatInputRef,
    clearSceneBarsBeforeSend: options.clearSceneBarsBeforeSend,
    offerSceneBarsAfterReply: options.offerSceneBarsAfterReply,
    onTurnRecorded: options.onTurnRecorded,
  })

  return {
    chatStore,
    chatListRef,
    chatInputRef,
    messages,
    chatListLoading,
    latestRoleplayAside,
    sceneHistorySplitIndex,
    onSend,
  }
}
