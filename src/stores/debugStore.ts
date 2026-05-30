import type { PresenceMode, SendMessageResponse } from '../api'
import { defineStore } from 'pinia'
import {

  queryEvents,
  queryMemories,
  reloadPolicyPlugins,

} from '../api'
import { useRoleStore } from './roleStore'

export const useDebugStore = defineStore('debug', {
  state: () => ({
    visible: false,
    events: [] as unknown[],
    memories: [] as unknown[],
    /** Knowledge chunks injected into Prompt on last `send_message` */
    lastKnowledgeChunksInPrompt: 0,
    /** Presence from the same response as above (co-present vs remote-presence) */
    lastKnowledgePresenceMode: null as PresenceMode | null,
  }),
  actions: {
    toggle() {
      this.visible = !this.visible
    },
    /** Written after main dialogue returns; dev panel shows knowledge hits for this turn */
    recordKnowledgeFromSend(res: SendMessageResponse) {
      this.lastKnowledgeChunksInPrompt = res.knowledge_chunks_in_prompt ?? 0
      this.lastKnowledgePresenceMode = res.presence_mode
    },
    async loadDebugData() {
      const roleStore = useRoleStore()
      const roleId = roleStore.currentRoleId
      const [events, memories] = await Promise.all([
        queryEvents({ role_id: roleId, limit: 10, offset: 0 }),
        queryMemories({ role_id: roleId, limit: 10, offset: 0 }),
        roleStore.refreshRoleInfo(),
      ])
      this.events = events
      this.memories = memories
    },
    async reloadPolicy() {
      return reloadPolicyPlugins()
    },
  },
})
