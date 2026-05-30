import { useChatStore } from '../stores/chatStore'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'
import { resolveUserNarrativeSceneId } from './narrativeScene'

/** After `refreshRoleInfo` etc., align `uiStore.sceneId` with `user_presence_scene`. */
export function useNarrativeScene() {
  const chatStore = useChatStore()
  const roleStore = useRoleStore()
  const uiStore = useUiStore()

  function applyResolvedNarrativeScene(): void {
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
