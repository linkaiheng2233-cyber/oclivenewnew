import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { resolveUserNarrativeSceneId } from '@oclive/shared/composables/narrativeScene'

/** After `refreshRoleInfo` etc., align `uiStore.sceneId` with `user_presence_scene`. */
export function useNarrativeScene() {
  const chatStore = useChatStore()
  const roleStore = useRoleStore()
  const uiStore = useUiStore()

  function applyResolvedNarrativeScene(): void {
    if (!roleStore.interactionImmersive)
      return
    chatStore.applySceneChange(
      resolveUserNarrativeSceneId(
        roleStore.roleInfo.userPresenceScene,
        roleStore.roleInfo.currentScene,
        roleStore.roleInfo.scenes,
        uiStore.sceneId,
      ),
      { skipHistorySplit: true },
    )
  }

  return { applyResolvedNarrativeScene, resolveUserNarrativeSceneId }
}
