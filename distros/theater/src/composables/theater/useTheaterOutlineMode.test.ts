import type { TheaterScenePreset } from './theaterSceneCatalog'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { DEFAULT_THEATER_CAST_CONFIG } from './theaterCastConfig'
import { requestOutlineScene } from './useTheaterOutlineMode'

const { generateTheaterScene } = vi.hoisted(() => ({
  generateTheaterScene: vi.fn(),
}))

vi.mock('@oclive/shared/api/theater', () => ({
  generateTheaterScene,
}))

const preset: TheaterScenePreset = {
  id: 'breakfast',
  labelKey: 'theater.scene.breakfast',
  skeletonPath: 'theater/breakfast.json',
  sceneBrief: 'breakfast',
  sceneSettingHint: 'home',
  pokeEnabled: true,
  pokeChips: [],
  runtimeSceneId: 'theater:home',
  order: 1,
}

describe('outline scene request', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('maps the current castA/castB contract into the kernel request', async () => {
    generateTheaterScene.mockResolvedValue({ source: 'model', beats: [] })

    await requestOutlineScene('first meet, then reconcile', DEFAULT_THEATER_CAST_CONFIG, preset)

    expect(generateTheaterScene).toHaveBeenCalledWith(
      expect.objectContaining({
        cast_a: {
          role_id: DEFAULT_THEATER_CAST_CONFIG.castA.roleId,
          name: DEFAULT_THEATER_CAST_CONFIG.castA.displayName,
        },
        cast_b: {
          role_id: DEFAULT_THEATER_CAST_CONFIG.castB.roleId,
          name: DEFAULT_THEATER_CAST_CONFIG.castB.displayName,
        },
      }),
    )
  })
})
