import type ChatMessageList from '@oclive/shared/components/chat/ChatMessageList.vue'
import type { ComposerTranslation } from 'vue-i18n'
import { computed, ref } from 'vue'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { useChatSend } from '@oclive/shared/composables/useChatSend'
import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import { effectiveChatSceneId } from '@oclive/shared/utils/pureChatScene'

export function useMainShellChat(options: {
  roleStore: ReturnType<typeof useRoleStore>
  uiStore: ReturnType<typeof useUiStore>
  showToast: AppToastFn
  t: ComposerTranslation
  clearSceneBarsBeforeSend: () => void
  offerSceneBarsAfterReply: (together: boolean, destination: boolean) => void
  onTurnRecorded: () => void
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

  const sceneHistorySplitIndex = computed(() => {
    if (!options.roleStore.interactionImmersive)
      return 0
    return chatStore.sceneHistorySplitForRoleScene(options.roleStore.currentRoleId, activeSceneId.value)
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
