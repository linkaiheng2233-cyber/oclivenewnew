import type ChatMessageList from '@oclive/shared/components/chat/ChatMessageList.vue'
import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import type { useRoleStore } from '@oclive/shared/stores/roleStore'
import type { useUiStore } from '@oclive/shared/stores/uiStore'
import type { ComposerTranslation } from 'vue-i18n'
import { useChatSend } from '@oclive/shared/composables/useChatSend'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { effectiveChatSceneId } from '@oclive/shared/utils/pureChatScene'
import { computed, ref } from 'vue'

export function useMainShellChat(options: {
  roleStore: ReturnType<typeof useRoleStore>
  uiStore: ReturnType<typeof useUiStore>
  showToast: AppToastFn
  t: ComposerTranslation
  clearSceneBarsBeforeSend: () => void
  offerSceneBarsAfterReply: (together: boolean, destination: boolean) => void
  onTurnRecorded: (userText: string) => void
}) {
  const chatStore = useChatStore()
  const chatListRef = ref<InstanceType<typeof ChatMessageList> | null>(null)
  const chatInputRef = ref<{ focusInput?: () => void } | null>(null)

  const activeSceneId = computed(() =>
    effectiveChatSceneId(
      options.roleStore.roleInfo.interactionMode,
      options.uiStore.sceneId,
    ),
  )

  const messages = computed(() =>
    chatStore.messagesForRoleScene(options.roleStore.currentRoleId, activeSceneId.value),
  )

  const chatListLoading = computed(() =>
    chatStore.isLoading
    || chatStore.isMessagesLoadingFor(options.roleStore.currentRoleId, activeSceneId.value),
  )

  const latestRoleplayAside = computed(() => {
    const roleId = options.roleStore.currentRoleId
    return chatStore.lastAssistantAsideFor(roleId, activeSceneId.value)
  })

  const sceneHistorySplitIndex = computed(() =>
    chatStore.sceneHistorySplitForRoleScene(
      options.roleStore.currentRoleId,
      activeSceneId.value,
    ),
  )

  const { onSend, onAdultAction } = useChatSend({
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
    onAdultAction,
  }
}
