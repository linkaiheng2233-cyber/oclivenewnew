import { useChatStore } from "../stores/chatStore";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import { resolveUserNarrativeSceneId } from "./narrativeScene";

/** 在 `refreshRoleInfo` 等拿到 DB 快照后，将 `uiStore.sceneId` 与 `user_presence_scene` 对齐 */
export function useNarrativeScene() {
  const chatStore = useChatStore();
  const roleStore = useRoleStore();
  const uiStore = useUiStore();

  function applyResolvedNarrativeScene(): void {
    chatStore.applySceneChange(
      resolveUserNarrativeSceneId(
        roleStore.roleInfo.userPresenceScene,
        roleStore.roleInfo.currentScene,
        roleStore.roleInfo.scenes,
        uiStore.sceneId,
      ),
      { skipHistorySplit: true },
    );
  }

  return { applyResolvedNarrativeScene, resolveUserNarrativeSceneId };
}
