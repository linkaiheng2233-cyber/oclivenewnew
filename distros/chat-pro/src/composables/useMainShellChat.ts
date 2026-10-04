import type ChatMessageList from '@oclive/shared/components/chat/ChatMessageList.vue'
import type { AppToastFn } from '@oclive/shared/composables/useAppToast'
import type { useRoleStore } from '@oclive/shared/stores/roleStore'
import type { useUiStore } from '@oclive/shared/stores/uiStore'
import type { ComposerTranslation } from 'vue-i18n'
import { useChatSend } from '@oclive/shared/composables/useChatSend'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useMinimalRoleChatStore } from '@oclive/shared/stores/minimalRoleChatStore'
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
  isContextChanging?: () => boolean
}) {
  const chatStore = useChatStore()
  const minimalRoleChatStore = useMinimalRoleChatStore()
  const activeChatKey = computed(() => minimalRoleChatStore.source?.role_id
    ?? `${options.roleStore.currentRoleId}-${options.uiStore.sceneId}`)
  const chatListRef = ref<InstanceType<typeof ChatMessageList> | null>(null)
  const chatInputRef = ref<{ focusInput?: () => void } | null>(null)

  const activeSceneId = computed(() =>
    effectiveChatSceneId(
      options.roleStore.roleInfo.interactionMode,
      options.uiStore.sceneId,
    ),
  )

  const messages = computed(() =>
    options.roleStore.minimalRoleActive
      ? minimalRoleChatStore.messages
      : chatStore.messagesForRoleScene(options.roleStore.currentRoleId, activeSceneId.value),
  )

  const chatListLoading = computed(() =>
    options.roleStore.minimalRoleActive
      ? minimalRoleChatStore.isLoading
      : chatStore.isLoading
        || chatStore.isMessagesLoadingFor(options.roleStore.currentRoleId, activeSceneId.value),
  )

  const latestRoleplayAside = computed(() => {
    if (options.roleStore.minimalRoleActive)
      return ''
    const roleId = options.roleStore.currentRoleId
    return chatStore.lastAssistantAsideFor(roleId, activeSceneId.value)
  })

  const sceneHistorySplitIndex = computed(() =>
    options.roleStore.minimalRoleActive
      ? 0
      : chatStore.sceneHistorySplitForRoleScene(
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
    isContextChanging: options.isContextChanging,
  })

  return {
    chatStore,
    minimalRoleChatStore,
    activeChatKey,
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
