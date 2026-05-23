import { defineStore } from 'pinia'
import { mcpActions, mcpState } from './plugin/mcpSlice'

/** Reserved for future Agent / MCP panel wiring. */
export const usePluginMcpStore = defineStore('pluginMcp', {
  state: () => mcpState(),
  actions: mcpActions,
})
