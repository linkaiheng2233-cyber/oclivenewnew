import type { MarketSliceStore } from './marketSlice'
import { describe, expect, it, vi } from 'vitest'
import {
  marketActions,
  marketState,
} from './marketSlice'

function createStore(): MarketSliceStore {
  return {
    ...marketState(),
    loadCachedPluginMarket: vi.fn(async () => {}),
    syncPluginMarket: vi.fn(async () => {}),
  }
}

describe('plugin market share URL routing', () => {
  it('routes a git repository to the explicit install review state', async () => {
    const store = createStore()

    await marketActions.loadFromShareUrl.call(
      store,
      'https://github.com/example/oclive-plugin-demo',
    )

    expect(store.pendingGitShareUrl).toBe(
      'https://github.com/example/oclive-plugin-demo',
    )
    expect(store.pluginMarketSnapshot).toBeNull()
    expect(store.syncPluginMarket).not.toHaveBeenCalled()
  })
})
