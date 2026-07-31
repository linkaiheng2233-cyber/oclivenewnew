import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  begin: vi.fn(),
  cancel: vi.fn(),
  commit: vi.fn(),
  generate: vi.fn(),
  list: vi.fn(),
  waitForVoice: vi.fn(async () => 'disabled'),
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
  waitForVoicePlaybackSettled: mocks.waitForVoice,
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

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, reject, resolve }
}

describe('adult beat background queue', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.resetModules()
    vi.clearAllMocks()
    mocks.adultStore.backgroundQueueEnabled = true
    mocks.adultStore.backgroundQueueCap = 2
    mocks.adultStore.sessions = {}
    mocks.roleStore.currentRoleId = 'role'
    mocks.uiStore.sceneId = 'home'
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
    mocks.waitForVoice.mockResolvedValue('disabled')
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
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

  it('lets an already-committing beat finish durably without displaying after cancellation', async () => {
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

    expect(display).not.toHaveBeenCalled()
    expect(mocks.commit).toHaveBeenCalledTimes(1)
    expect(mocks.cancel).toHaveBeenCalledTimes(1)
  })

  it('waits for an in-flight begin and cancels its late generation', async () => {
    const begun = deferred<{ generation_id: string, next_sequence: number }>()
    mocks.begin.mockReturnValueOnce(begun.promise)
    const queue = await import('./adultBeatQueue')
    const start = queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      { display: vi.fn() },
    )
    await flushMicrotasks()
    expect(mocks.begin).toHaveBeenCalledTimes(1)

    const cancellation = queue.cancelAdultBeatQueue('role', 'home')
    await flushMicrotasks()
    expect(mocks.cancel).not.toHaveBeenCalled()

    begun.resolve({
      generation_id: 'late-generation',
      next_sequence: 0,
    })
    await Promise.all([start, cancellation])
    await flushMicrotasks()

    expect(mocks.generate).not.toHaveBeenCalled()
    expect(mocks.cancel).toHaveBeenCalledWith({
      role_id: 'role',
      scene_id: 'home',
      generation_id: 'late-generation',
    })
  })

  it('cancels a failed generation and permits a clean restart', async () => {
    mocks.begin
      .mockResolvedValueOnce({
        generation_id: 'failed-generation',
        next_sequence: 0,
      })
      .mockResolvedValueOnce({
        generation_id: 'replacement-generation',
        next_sequence: 0,
      })
    mocks.generate.mockRejectedValueOnce(new Error('generation failed'))
    const queue = await import('./adultBeatQueue')
    const reportError = vi.fn()

    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      { display: vi.fn(), reportError },
    )
    await flushMicrotasks()

    expect(reportError).toHaveBeenCalledWith('generation failed')
    expect(mocks.cancel).toHaveBeenCalledWith({
      role_id: 'role',
      scene_id: 'home',
      generation_id: 'failed-generation',
    })

    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'replacement-turn',
      { display: vi.fn(), reportError },
    )
    await flushMicrotasks()

    expect(mocks.begin).toHaveBeenCalledTimes(2)
    expect(mocks.generate).toHaveBeenCalledTimes(3)
    await queue.cancelAllAdultBeatQueues()
  })

  it('cancels the remaining generation when displaying a committed beat fails', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    const queue = await import('./adultBeatQueue')
    const reportError = vi.fn()
    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      {
        display: vi.fn().mockRejectedValue(new Error('display failed')),
        reportError,
      },
    )
    await flushMicrotasks()

    await vi.advanceTimersByTimeAsync(10)
    await flushMicrotasks()

    expect(reportError).toHaveBeenCalledWith('display failed')
    expect(mocks.cancel).toHaveBeenCalledWith({
      role_id: 'role',
      scene_id: 'home',
      generation_id: 'generation',
    })
  })

  it('retries a failed durable cancellation from the persisted tombstone', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    mocks.adultStore.sessions['role:home'] = {
      active: false,
      voiceTextOnly: false,
      updatedAt: 1,
      generationId: 'retry-generation',
      roleId: 'role',
      sceneId: 'home',
    }
    mocks.cancel
      .mockRejectedValueOnce(new Error('temporary cancel failure'))
      .mockResolvedValueOnce(undefined)
    const queue = await import('./adultBeatQueue')

    await queue.cancelAdultBeatQueue('role', 'home')
    expect(mocks.adultStore.sessions['role:home']?.generationId)
      .toBe('retry-generation')

    await queue.cancelAdultBeatQueue('role', 'home')
    expect(mocks.adultStore.sessions['role:home']?.generationId).toBeUndefined()
    expect(mocks.cancel).toHaveBeenCalledTimes(2)
    expect(warn).toHaveBeenCalledWith(
      '[adult-stage] cancel failed',
      expect.objectContaining({ message: 'temporary cancel failure' }),
    )
  })

  it('waits to display a committed beat until its chat owns the foreground again', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    const committed = deferred<ReturnType<typeof response>>()
    mocks.commit.mockReturnValueOnce(committed.promise)
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
    mocks.roleStore.currentRoleId = 'other-role'
    committed.resolve(response(0))
    await flushMicrotasks()

    expect(display).not.toHaveBeenCalled()

    mocks.roleStore.currentRoleId = 'role'
    await vi.advanceTimersByTimeAsync(250)
    await flushMicrotasks()
    expect(display).toHaveBeenCalledTimes(1)

    await queue.cancelAllAdultBeatQueues()
  })

  it('shares the global cap fairly across active chats', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    mocks.begin.mockImplementation(async (request: { role_id: string }) => ({
      generation_id: `generation-${request.role_id}`,
      next_sequence: 0,
    }))
    mocks.generate.mockImplementation(
      async (request: { role_id: string, generation_id: string, sequence: number }) => ({
        generation_id: request.generation_id,
        sequence: request.sequence,
        response: response(request.sequence),
      }),
    )
    const queue = await import('./adultBeatQueue')
    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-role',
      { display: vi.fn() },
    )
    await flushMicrotasks()
    await queue.startAdultBeatQueue(
      'role-b',
      'home',
      response(-1),
      'first-role-b',
      { display: vi.fn() },
    )
    await flushMicrotasks()

    expect(mocks.generate).toHaveBeenCalledTimes(1)
    expect(mocks.generate.mock.calls[0]?.[0].role_id).toBe('role')

    await vi.advanceTimersByTimeAsync(10)
    await flushMicrotasks()
    expect(mocks.generate).toHaveBeenCalledTimes(2)
    expect(mocks.generate.mock.calls[1]?.[0].role_id).toBe('role-b')

    await queue.cancelAllAdultBeatQueues()
  })

  it('marks only the current interaction text-only when prior voice fails', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    mocks.waitForVoice.mockResolvedValueOnce('error')
    const queue = await import('./adultBeatQueue')
    await queue.startAdultBeatQueue(
      'role',
      'home',
      response(-1),
      'first-turn',
      { display: vi.fn() },
    )
    await flushMicrotasks()
    await vi.advanceTimersByTimeAsync(10)
    await flushMicrotasks()

    expect(mocks.adultStore.markVoiceTextOnly)
      .toHaveBeenCalledWith('role', 'home')
    await queue.cancelAllAdultBeatQueues()
  })

  it('restores durable pending beats after restart and displays them in order', async () => {
    mocks.adultStore.backgroundQueueCap = 1
    mocks.adultStore.sessions['role:home'] = {
      active: true,
      voiceTextOnly: false,
      updatedAt: 1,
      generationId: 'restored-generation',
      roleId: 'role',
      sceneId: 'home',
    }
    mocks.list.mockResolvedValue({
      generation_id: 'restored-generation',
      active: true,
      next_sequence: 1,
      beats: [{
        generation_id: 'restored-generation',
        sequence: 0,
        response: response(0),
      }],
    })
    const queue = await import('./adultBeatQueue')
    const display = vi.fn()

    await queue.resumeAdultBeatQueue('role', 'home', { display })
    await flushMicrotasks()
    await vi.advanceTimersByTimeAsync(1)
    await flushMicrotasks()

    expect(mocks.list).toHaveBeenCalledWith({
      role_id: 'role',
      scene_id: 'home',
      generation_id: 'restored-generation',
    })
    expect(mocks.commit).toHaveBeenCalledWith(expect.objectContaining({
      generation_id: 'restored-generation',
      sequence: 0,
    }))
    expect(display).toHaveBeenCalledTimes(1)

    await queue.cancelAllAdultBeatQueues()
  })
})
