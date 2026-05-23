import { defineStore } from 'pinia'
import { directoryActions, directoryState } from './plugin/directorySlice'
import { installedActions, installedState } from './plugin/installedSlice'
import { marketActions, marketState } from './plugin/marketSlice'
import { mcpActions, mcpState } from './plugin/mcpSlice'
import { traceActions, traceState } from './plugin/traceSlice'

export * from './plugin/constants'

export const usePluginStore = defineStore('plugin', {
  state: () => ({
    ...traceState(),
    ...directoryState(),
    ...installedState(),
    ...marketState(),
    ...mcpState(),
  }),
  actions: {
    ...traceActions,
    ...directoryActions,
    ...installedActions,
    ...marketActions,
    ...mcpActions,
  },
})
