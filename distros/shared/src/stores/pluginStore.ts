import { defineStore } from 'pinia'
import { directoryActions, directoryState } from './plugin/directorySlice'
import { installedActions, installedState } from './plugin/installedSlice'

export * from './plugin/constants'
export { usePluginMarketStore } from './pluginMarketStore'
export { usePluginMcpStore } from './pluginMcpStore'

export const usePluginStore = defineStore('plugin', {
  state: () => ({
    ...directoryState(),
    ...installedState(),
  }),
  actions: {
    ...directoryActions,
    ...installedActions,
  },
})
