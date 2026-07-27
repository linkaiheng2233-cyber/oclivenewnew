import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  begin: vi.fn(),
  cancel: vi.fn(),
  commit: vi.fn(),
  generate: vi.fn(),
  list: vi.fn(),
  roleStore: { currentRoleId: 'role' },
  uiStore: { sceneId: 'home' },
  adultStore: {
    backgroundQueueEnabled: true,
    backgroundQueueCap: 2,
    pacingOverrideEnabled: false,
    pacingIntervalMs: 4_000,
    sessions: {} as Record<string, {
      active: boolean
      voiceTextOnly: boolean
      updatedAt: number
      generationId?: string
      roleId?: string
      sceneId?: string
    }>,
    requestFor: vi.fn(() => ({
      confirmed_adult: true,
      global_enabled: true,
      role_enabled: true,
      interaction_active: true,
      action: 'continue',
    })),
    sessionFor(roleId: string, sceneId: string) {
      return this.sessions[`${roleId}:${sceneId}`] ?? {
        active: true,
        voiceTextOnly: false,
        updatedAt: 0,
      }
    },
    setSessionGeneration(roleId: string, sceneId: string, generationId?: string) {
      const key = `${roleId}:${sceneId}`
      this.sessions[key] = {
        ...this.sessionFor(roleId, sceneId),
        roleId,
        sceneId,
        ...(generationId ? { generationId } : {}),
      }
      if (!generationId)
        delete this.sessions[key].generationId
    },
    markVoiceTextOnly: vi.fn(),
  },
}))

vi.mock('@oclive/shared/api', () => ({
  beginAdultStageGeneration: mocks.begin,
  cancelAdultStageGeneration: mocks.cancel,
  commitAdultStagedBeat: mocks.commit,
  generateAdultStagedBeat: mocks.generate,
  listAdultStagedBeats: mocks.list,
}))
vi.mock('@oclive/shared/stores/adultInteractionStore', () => ({
  useAdultInteractionStore: () => mocks.adultStore,
}))
vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))
vi.mock('@oclive/shared/stores/uiStore', () => ({
  useUiStore: () => mocks.uiStore,
}))
vi.mock('./voicePlaybackSettlement', () => ({
  waitForVoicePlaybackSettled: vi.fn(async () => 'disabled'),
}))

function response(sequence: number, state: 'active' | 'ended' = 'active') {
  return {
    reply: `dialogue ${sequence}`,
    adult_beat: {
      dialogue: `dialogue ${sequence}`,
      narration: `narration ${sequence}`,
      interaction_state: state,
      next_beat_interval_ms: 10,
    },
  }
}

async function flushMicrotasks(rounds = 12): Promise<void> {
  for (let index = 0; index < rounds; index += 1)
    await Promise.resolve()
}

describe('adult beat background queue', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.resetModules()
    vi.clearAllMocks()
    mocks.adultStore.backgroundQueueEnabled = true
    mocks.adultStore.backgroundQueueCap = 2
    mocks.adultStore.sessions = {}
    mocks.begin.mockResolvedValue({
      generation_id: 'generation',
      next_sequence: 0,
    })
    mocks.generate.mockImplementation(async (request: { sequence: number }) => ({
      generation_id: 'generation',
      sequence: request.sequence,
      response: response(request.sequence),
    }))
    mocks.commit.mockImplementation(
      async (request: { sequence: number }) => response(request.sequence),
    )
    mocks.cancel.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('pre-generates structured text up to the global cap without displaying it', async () => {
    const queue = await import('./adultBeatQueue')
    const display = vi.fn()
    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      { display },
    )
    await flushMicrotasks()

    expect(mocks.generate).toHaveBeenCalledTimes(2)
    expect(mocks.commit).not.toHaveBeenCalled()
    expect(display).not.toHaveBeenCalled()

    await queue.cancelAllAdultBeatQueues()
    expect(mocks.cancel).toHaveBeenCalledTimes(1)
  })

  it('commits and displays one ordered beat after pacing and prior voice settle', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    const queue = await import('./adultBeatQueue')
    const display = vi.fn()
    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      { display },
    )
    await flushMicrotasks()
    expect(mocks.generate).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(10)
    await flushMicrotasks()

    expect(mocks.commit).toHaveBeenCalledWith(expect.objectContaining({
      generation_id: 'generation',
      sequence: 0,
    }))
    expect(display).toHaveBeenCalledTimes(1)
    expect(display.mock.calls[0]?.[0].reply).toBe('dialogue 0')

    await queue.cancelAllAdultBeatQueues()
  })

  it('cancels a persisted generation even when no in-memory queue was restored', async () => {
    mocks.adultStore.sessions['role:home'] = {
      active: true,
      voiceTextOnly: false,
      updatedAt: 1,
      generationId: 'persisted-generation',
      roleId: 'role',
      sceneId: 'home',
    }
    const queue = await import('./adultBeatQueue')

    await queue.cancelAllAdultBeatQueues()

    expect(mocks.cancel).toHaveBeenCalledWith({
      role_id: 'role',
      scene_id: 'home',
      generation_id: 'persisted-generation',
    })
    expect(mocks.adultStore.sessions['role:home']?.generationId).toBeUndefined()
  })

  it('lets an already-committing beat display before cancellation removes later beats', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    let resolveCommit: ((value: ReturnType<typeof response>) => void) | undefined
    mocks.commit.mockImplementation(
      () => new Promise(resolve => (resolveCommit = resolve)),
    )
    const queue = await import('./adultBeatQueue')
    const display = vi.fn()
    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      { display },
    )
    await flushMicrotasks()
    await vi.advanceTimersByTimeAsync(10)
    await flushMicrotasks()
    expect(mocks.commit).toHaveBeenCalledTimes(1)

    const cancellation = queue.cancelAllAdultBeatQueues()
    await flushMicrotasks()
    expect(mocks.cancel).not.toHaveBeenCalled()

    resolveCommit?.(response(0))
    await cancellation

    expect(display).toHaveBeenCalledTimes(1)
    expect(mocks.cancel).toHaveBeenCalledTimes(1)
    expect(display.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.cancel.mock.invocationCallOrder[0]!,
    )
  })
})
