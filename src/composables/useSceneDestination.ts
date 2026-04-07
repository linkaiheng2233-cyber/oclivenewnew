import { ref } from "vue";
import { useChatStore } from "../stores/chatStore";
import { useDebugStore } from "../stores/debugStore";
import { useRoleStore } from "../stores/roleStore";
import { setUserPresenceScene, switchScene } from "../utils/tauri-api";
import type { ToastType } from "./useAppToast";

const SCENE_TRANSITION_MS = 520;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

export type ShowToast = (type: ToastType, message: string) => void;

/**
 * 顶栏/位移条「前往」：同行则 `switchScene`；仅叙事则 `setUserPresenceScene`。
 */
export function useSceneDestination(showToast: ShowToast) {
  const roleStore = useRoleStore();
  const chatStore = useChatStore();
  const debugStore = useDebugStore();

  const sceneTransition = ref({ visible: false, label: "" });

  function sceneLabelForId(sceneId: string): string {
    const row = roleStore.roleInfo.sceneLabels?.find((s) => s.id === sceneId);
    return row?.label ?? sceneId;
  }

  function characterSceneLabel(): string {
    const id = roleStore.roleInfo.currentScene;
    if (!id) return "—";
    return sceneLabelForId(id);
  }

  async function applySceneDestination(id: string, together: boolean): Promise<void> {
    if (!id.trim()) {
      showToast("warning", "请先选择目的地");
      return;
    }
    const label = sceneLabelForId(id);
    if (together) {
      sceneTransition.value = { visible: true, label };
    }
    try {
      if (together) {
        const res = await switchScene(roleStore.currentRoleId, id, true);
        await sleep(SCENE_TRANSITION_MS);
        sceneTransition.value = { visible: false, label: "" };
        roleStore.applyRoleInfo(res);
        const narrative = res.user_presence_scene ?? id;
        chatStore.applySceneChange(narrative);
        if (res.scene_welcome) {
          chatStore.addSystemMessage(res.scene_welcome, narrative);
        }
        showToast("success", "已前往（同行）");
      } else {
        const info = await setUserPresenceScene(roleStore.currentRoleId, id);
        roleStore.applyRoleInfo(info);
        const narrative = info.user_presence_scene ?? id;
        chatStore.applySceneChange(narrative);
        chatStore.addSystemMessage(
          `叙事场景已切换为「${label}」；角色仍位于「${characterSceneLabel()}」。`,
          narrative,
        );
        showToast("success", "已切换叙事场景（角色未移动）");
      }
      await debugStore.loadDebugData();
    } catch (err) {
      sceneTransition.value = { visible: false, label: "" };
      showToast("error", err instanceof Error ? err.message : String(err));
    }
  }

  return {
    sceneTransition,
    applySceneDestination,
    sceneLabelForId,
    characterSceneLabel,
  };
}
