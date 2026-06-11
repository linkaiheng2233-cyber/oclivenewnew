import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  injectDirectorBeat,
  isDirectorPluginAvailable,
  pingDirector,
  resetDirectorAvailabilityCache,
  THEATER_DIRECTOR_PLUGIN_ID,
} from './theaterDirectorClient'

vi.mock('../api/plugin', () => ({
  getDirectoryPluginCatalog: vi.fn(),
  directoryPluginInvoke: vi.fn(),
}))

import { directoryPluginInvoke, getDirectoryPluginCatalog } from '../api/plugin'

describe('theaterDirectorClient', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetDirectorAvailabilityCache()
  })

  it('returns false when director plugin not in catalog', async () => {
    vi.mocked(getDirectoryPluginCatalog).mockResolvedValue([
      { id: 'other.plugin', version: '1.0.0', hasRpcProcess: true, isShell: false, uiSlotNames: [], provides: [], dependencyStatus: 'ok', dependencyIssues: [] },
    ])
    await expect(isDirectorPluginAvailable()).resolves.toBe(false)
    await expect(pingDirector()).resolves.toBeNull()
  })

  it('pings director when catalog lists plugin', async () => {
    vi.mocked(getDirectoryPluginCatalog).mockResolvedValue([
      { id: THEATER_DIRECTOR_PLUGIN_ID, version: '0.1.0', hasRpcProcess: true, isShell: false, uiSlotNames: [], provides: [], dependencyStatus: 'ok', dependencyIssues: [] },
    ])
    vi.mocked(directoryPluginInvoke).mockResolvedValue({
      ok: true,
      plugin: THEATER_DIRECTOR_PLUGIN_ID,
      version: '0.1.0',
    })
    await expect(pingDirector()).resolves.toMatchObject({ ok: true })
    expect(directoryPluginInvoke).toHaveBeenCalledWith(
      THEATER_DIRECTOR_PLUGIN_ID,
      'theater.director.ping',
      {},
    )
  })

  it('injectDirectorBeat maps RPC beat to TheaterBeat', async () => {
    vi.mocked(getDirectoryPluginCatalog).mockResolvedValue([
      { id: THEATER_DIRECTOR_PLUGIN_ID, version: '0.1.0', hasRpcProcess: true, isShell: false, uiSlotNames: [], provides: [], dependencyStatus: 'ok', dependencyIssues: [] },
    ])
    vi.mocked(directoryPluginInvoke).mockResolvedValue({
      beat: { id: 'inj_1', speaker: 'b', text: '导演注入一句', delay_ms: 0 },
    })
    const beat = await injectDirectorBeat({
      scene_id: 'breakfast',
      summary: '测试',
      speaker: 'b',
    })
    expect(beat?.text).toBe('导演注入一句')
    expect(beat?.speaker).toBe('b')
  })
})
