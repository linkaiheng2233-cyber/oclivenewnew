import { defineStore } from "pinia";
import {
  queryEvents,
  queryMemories,
  reloadPolicyPlugins,
  type PresenceMode,
  type SendMessageResponse,
} from "../utils/tauri-api";
import { useRoleStore } from "./roleStore";

export const useDebugStore = defineStore("debug", {
  state: () => ({
    visible: false,
    events: [] as unknown[],
    memories: [] as unknown[],
    /** 最近一次 `send_message` 注入 Prompt 的知识块条数 */
    lastKnowledgeChunksInPrompt: 0,
    /** 与上一项同一次响应的 presence（便于区分共景/异地） */
    lastKnowledgePresenceMode: null as PresenceMode | null,
  }),
  actions: {
    toggle() {
      this.visible = !this.visible;
    },
    /** 主对话返回后写入，供开发面板展示「本回合」知识命中 */
    recordKnowledgeFromSend(res: SendMessageResponse) {
      this.lastKnowledgeChunksInPrompt = res.knowledge_chunks_in_prompt ?? 0;
      this.lastKnowledgePresenceMode = res.presence_mode;
    },
    async loadDebugData() {
      const roleStore = useRoleStore();
      const roleId = roleStore.currentRoleId;
      const [events, memories] = await Promise.all([
        queryEvents({ role_id: roleId, limit: 10, offset: 0 }),
        queryMemories({ role_id: roleId, limit: 10, offset: 0 }),
        roleStore.refreshRoleInfo(),
      ]);
      this.events = events;
      this.memories = memories;
    },
    async reloadPolicy() {
      return reloadPolicyPlugins();
    },
  },
});
