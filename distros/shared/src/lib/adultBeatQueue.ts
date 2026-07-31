import type {
  AdultInteractionRequest,
  AdultStagedBeatDto,
  SendMessageResponse,
} from '@oclive/shared/api'
import {
  beginAdultStageGeneration,
  cancelAdultStageGeneration,
  commitAdultStagedBeat,
  generateAdultStagedBeat,
  listAdultStagedBeats,
} from '@oclive/shared/api'
import { useAdultInteractionStore } from '@oclive/shared/stores/adultInteractionStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { waitForVoicePlaybackSettled } from './voicePlaybackSettlement'

export interface AdultBeatDisplayHooks {
  display: (
    response: SendMessageResponse,
    roleId: string,
    sceneId: string,
    turnId: string,
  ) => Promise<void> | void
  reportError?: (message: string) => void
}

interface AdultBeatQueueRuntime {
  key: string
  roleId: string
  sceneId: string
  generationId: string
  nextSequence: number
  nextCommit: number
  pending: Map<number, AdultStagedBeatDto>
  generating: boolean
  generationDone: boolean
  stopped: boolean
  displaying: boolean
  commitSettled: Promise<SendMessageResponse> | null
  previousVoiceTurnId: string | null
  previousIntervalMs: number
  hooks: AdultBeatDisplayHooks
}

const queues = new Map<string, AdultBeatQueueRuntime>()
const queueIntentVersions = new Map<string, number>()
const queueSetupTails = new Map<string, Promise<void>>()
const pendingSetupTargets = new Map<string, {
  roleId: string
  sceneId: string
  version: number
}>()
let pumpRunning = false
let lastGeneratedQueueKey: string | null = null

function queueKey(roleId: string, sceneId: string): string {
  return `${roleId.trim()}:${sceneId.trim() || 'default'}`
}

function invalidateQueueIntent(key: string): number {
  const next = (queueIntentVersions.get(key) ?? 0) + 1
  queueIntentVersions.set(key, next)
  return next
}

function queueIntentIsCurrent(key: string, version: number): boolean {
  return queueIntentVersions.get(key) === version
}

function cleanupQueueIntentIfIdle(key: string): void {
  if (!queues.has(key) && !queueSetupTails.has(key)) {
    queueIntentVersions.delete(key)
    pendingSetupTargets.delete(key)
  }
}

function scheduleQueueSetup(
  roleId: string,
  sceneId: string,
  setup: (version: number) => Promise<void>,
): Promise<void> {
  const key = queueKey(roleId, sceneId)
  const version = invalidateQueueIntent(key)
  const previous = queueSetupTails.get(key) ?? Promise.resolve()
  const run: Promise<void> = previous
    .catch(() => undefined)
    .then(() => setup(version))
    .finally(() => {
      if (queueSetupTails.get(key) === run) {
        queueSetupTails.delete(key)
        pendingSetupTargets.delete(key)
        cleanupQueueIntentIfIdle(key)
      }
    })
  queueSetupTails.set(key, run)
  pendingSetupTargets.set(key, { roleId, sceneId, version })
  return run
}

function clearSessionGenerationIfMatches(
  roleId: string,
  sceneId: string,
  generationId: string,
): void {
  const adultStore = useAdultInteractionStore()
  if (adultStore.sessionFor(roleId, sceneId).generationId === generationId)
    adultStore.setSessionGeneration(roleId, sceneId)
}

function detachQueue(queue: AdultBeatQueueRuntime): void {
  queue.stopped = true
  if (queues.get(queue.key) === queue) {
    queues.delete(queue.key)
    cleanupQueueIntentIfIdle(queue.key)
  }
}

async function cancelKnownGeneration(
  roleId: string,
  sceneId: string,
  generationId: string,
): Promise<void> {
  try {
    await cancelAdultStageGeneration({
      role_id: roleId,
      scene_id: sceneId,
      generation_id: generationId,
    })
    clearSessionGenerationIfMatches(roleId, sceneId, generationId)
  }
  catch (error) {
    // Keep the persisted generation id on failure so a later lifecycle cleanup
    // can retry instead of silently orphaning the kernel-side generation.
    console.warn('[adult-stage] cancel failed', error)
  }
  finally {
    void pumpGeneration()
  }
}

function stopQueueAndCancel(queue: AdultBeatQueueRuntime): void {
  if (queue.stopped)
    return
  detachQueue(queue)
  void cancelKnownGeneration(
    queue.roleId,
    queue.sceneId,
    queue.generationId,
  )
}

function failQueue(queue: AdultBeatQueueRuntime, error: unknown): void {
  if (queue.stopped)
    return
  queue.hooks.reportError?.(
    error instanceof Error ? error.message : String(error),
  )
  stopQueueAndCancel(queue)
}

function finishQueue(queue: AdultBeatQueueRuntime): void {
  detachQueue(queue)
  clearSessionGenerationIfMatches(
    queue.roleId,
    queue.sceneId,
    queue.generationId,
  )
}

function persistedGenerationTargets(): Array<{
  roleId: string
  sceneId: string
}> {
  const targets: Array<{ roleId: string, sceneId: string }> = []
  for (const [key, session] of Object.entries(
    useAdultInteractionStore().sessions,
  )) {
    if (!session.generationId)
      continue
    if (session.roleId && session.sceneId) {
      targets.push({ roleId: session.roleId, sceneId: session.sceneId })
      continue
    }
    // Backward-compatible fallback for sessions persisted before role/scene
    // metadata was added. Role ids cannot be empty; scene defaults to default.
    const separator = key.lastIndexOf(':')
    if (separator <= 0)
      continue
    targets.push({
      roleId: key.slice(0, separator),
      sceneId: key.slice(separator + 1) || 'default',
    })
  }
  return targets
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, Math.max(1, ms)))
}

function globalBufferedCount(): number {
  let total = 0
  for (const queue of queues.values())
    total += queue.pending.size + (queue.generating ? 1 : 0)
  return total
}

function isForeground(queue: AdultBeatQueueRuntime): boolean {
  const roleStore = useRoleStore()
  const uiStore = useUiStore()
  const visible = typeof document === 'undefined' || document.visibilityState === 'visible'
  return visible
    && roleStore.currentRoleId === queue.roleId
    && (uiStore.sceneId || 'default') === queue.sceneId
}

async function waitUntilForeground(queue: AdultBeatQueueRuntime): Promise<boolean> {
  while (!queue.stopped && !isForeground(queue))
    await delay(250)
  return !queue.stopped
}

function requestForContinuation(
  queue: AdultBeatQueueRuntime,
): AdultInteractionRequest | undefined {
  return useAdultInteractionStore().requestFor(
    queue.roleId,
    queue.sceneId,
    'continue',
  )
}

function selectQueueForGeneration(): AdultBeatQueueRuntime | undefined {
  const candidates = [...queues.values()].filter(queue =>
    !queue.stopped
    && !queue.generationDone
    && !queue.generating
    && requestForContinuation(queue) !== undefined,
  )
  if (candidates.length === 0)
    return undefined
  const previousIndex = lastGeneratedQueueKey
    ? candidates.findIndex(queue => queue.key === lastGeneratedQueueKey)
    : -1
  const selected = candidates[(previousIndex + 1) % candidates.length]
  lastGeneratedQueueKey = selected.key
  return selected
}

async function pumpGeneration(): Promise<void> {
  if (pumpRunning)
    return
  pumpRunning = true
  try {
    while (true) {
      const adultStore = useAdultInteractionStore()
      if (!adultStore.backgroundQueueEnabled)
        return
      const cap = Math.max(1, Math.trunc(adultStore.backgroundQueueCap))
      if (globalBufferedCount() >= cap)
        return
      const queue = selectQueueForGeneration()
      if (!queue)
        return
      const adult = requestForContinuation(queue)
      if (!adult) {
        queue.generationDone = true
        continue
      }
      queue.generating = true
      const sequence = queue.nextSequence
      try {
        const staged = await generateAdultStagedBeat({
          role_id: queue.roleId,
          scene_id: queue.sceneId,
          generation_id: queue.generationId,
          sequence,
          adult,
        })
        if (queue.stopped)
          continue
        queue.pending.set(sequence, staged)
        queue.nextSequence = sequence + 1
        if (staged.response.adult_beat?.interaction_state !== 'active')
          queue.generationDone = true
      }
      catch (error) {
        failQueue(queue, error)
      }
      finally {
        queue.generating = false
      }
    }
  }
  finally {
    pumpRunning = false
  }
}

async function waitForSequence(
  queue: AdultBeatQueueRuntime,
  sequence: number,
): Promise<AdultStagedBeatDto | undefined> {
  while (!queue.stopped) {
    const beat = queue.pending.get(sequence)
    if (beat)
      return beat
    if (queue.generationDone && !queue.generating)
      return undefined
    void pumpGeneration()
    await delay(50)
  }
  return undefined
}

async function displayLoop(queue: AdultBeatQueueRuntime): Promise<void> {
  if (queue.displaying)
    return
  queue.displaying = true
  try {
    while (!queue.stopped) {
      const interval = delay(queue.previousIntervalMs)
      const voice = queue.previousVoiceTurnId
        ? waitForVoicePlaybackSettled(queue.previousVoiceTurnId)
        : Promise.resolve<'disabled'>('disabled')
      const [voiceStatus] = await Promise.all([voice, interval])
      if (queue.stopped)
        return
      if (voiceStatus === 'error' || voiceStatus === 'timeout') {
        useAdultInteractionStore().markVoiceTextOnly(
          queue.roleId,
          queue.sceneId,
        )
      }
      if (!await waitUntilForeground(queue))
        return
      const staged = await waitForSequence(queue, queue.nextCommit)
      if (!staged) {
        stopQueueAndCancel(queue)
        return
      }
      const committedSequence = queue.nextCommit
      queue.commitSettled = commitAdultStagedBeat({
        role_id: queue.roleId,
        scene_id: queue.sceneId,
        generation_id: queue.generationId,
        sequence: committedSequence,
      })
      let committed: SendMessageResponse
      try {
        committed = await queue.commitSettled
      }
      finally {
        queue.commitSettled = null
      }
      queue.pending.delete(committedSequence)
      queue.nextCommit = committedSequence + 1
      void pumpGeneration()
      if (queue.stopped)
        return
      // The durable commit belongs to the old chat even if the user switched
      // away while the RPC was in flight. Delay only the visible bubble/TTS
      // until that role and scene own the foreground again.
      if (!await waitUntilForeground(queue))
        return
      const displayTurnId = `adult-stage-${queue.generationId}-${committedSequence}`
      await queue.hooks.display(
        committed,
        queue.roleId,
        queue.sceneId,
        displayTurnId,
      )
      const turnId = `adult-stage-${queue.generationId}-${committedSequence}`
      queue.previousVoiceTurnId = turnId
      queue.previousIntervalMs = useAdultInteractionStore().pacingOverrideEnabled
        ? useAdultInteractionStore().pacingIntervalMs
        : (committed.adult_beat?.next_beat_interval_ms ?? 4_000)
      if (committed.adult_beat?.interaction_state !== 'active') {
        queue.generationDone = true
        finishQueue(queue)
        return
      }
    }
  }
  catch (error) {
    failQueue(queue, error)
  }
  finally {
    queue.displaying = false
  }
}

export async function startAdultBeatQueue(
  roleId: string,
  sceneId: string,
  firstResponse: SendMessageResponse,
  firstVoiceTurnId: string,
  hooks: AdultBeatDisplayHooks,
): Promise<void> {
  const adultStore = useAdultInteractionStore()
  const sid = sceneId || 'default'
  if (
    !adultStore.backgroundQueueEnabled
    || firstResponse.adult_beat?.interaction_state !== 'active'
  ) {
    return
  }
  const key = queueKey(roleId, sid)
  const pendingSetup = pendingSetupTargets.get(key)
  if (
    queues.has(key)
    || (
      pendingSetup
      && queueIntentIsCurrent(key, pendingSetup.version)
    )
  ) {
    return
  }
  const adult = adultStore.requestFor(roleId, sid, 'continue')
  if (!adult)
    return
  return scheduleQueueSetup(roleId, sid, async (version) => {
    if (
      !queueIntentIsCurrent(key, version)
      || queues.has(key)
      || !adultStore.backgroundQueueEnabled
      || firstResponse.adult_beat?.interaction_state !== 'active'
    ) {
      return
    }
    const currentAdult = adultStore.requestFor(roleId, sid, 'continue')
    if (!currentAdult)
      return
    const begun = await beginAdultStageGeneration({
      role_id: roleId,
      scene_id: sid,
      adult: currentAdult,
    })
    // Persist ownership before any stale-intent check. If the matching cancel
    // RPC fails, this tombstone remains available for lifecycle retry.
    adultStore.setSessionGeneration(roleId, sid, begun.generation_id)
    if (
      !queueIntentIsCurrent(key, version)
      || queues.has(key)
      || !adultStore.backgroundQueueEnabled
      || firstResponse.adult_beat?.interaction_state !== 'active'
      || !adultStore.requestFor(roleId, sid, 'continue')
    ) {
      await cancelKnownGeneration(roleId, sid, begun.generation_id)
      return
    }
    const queue: AdultBeatQueueRuntime = {
      key,
      roleId,
      sceneId: sid,
      generationId: begun.generation_id,
      nextSequence: begun.next_sequence,
      nextCommit: 0,
      pending: new Map(),
      generating: false,
      generationDone: false,
      stopped: false,
      displaying: false,
      commitSettled: null,
      previousVoiceTurnId: firstVoiceTurnId,
      previousIntervalMs: adultStore.pacingOverrideEnabled
        ? adultStore.pacingIntervalMs
        : (firstResponse.adult_beat.next_beat_interval_ms ?? 4_000),
      hooks,
    }
    queues.set(key, queue)
    void pumpGeneration()
    void displayLoop(queue)
  })
}

export async function resumeAdultBeatQueue(
  roleId: string,
  sceneId: string,
  hooks: AdultBeatDisplayHooks,
): Promise<void> {
  const adultStore = useAdultInteractionStore()
  const sid = sceneId || 'default'
  const key = queueKey(roleId, sid)
  const pendingSetup = pendingSetupTargets.get(key)
  if (
    queues.has(key)
    || (
      pendingSetup
      && queueIntentIsCurrent(key, pendingSetup.version)
    )
    || !adultStore.backgroundQueueEnabled
    || !adultStore.sessionFor(roleId, sid).active
  ) {
    return
  }
  const generationId = adultStore.sessionFor(roleId, sid).generationId
  if (!generationId)
    return
  return scheduleQueueSetup(roleId, sid, async (version) => {
    try {
      const restored = await listAdultStagedBeats({
        role_id: roleId,
        scene_id: sid,
        generation_id: generationId,
      })
      const currentSession = adultStore.sessionFor(roleId, sid)
      if (
        !queueIntentIsCurrent(key, version)
        || queues.has(key)
        || !adultStore.backgroundQueueEnabled
        || !currentSession.active
        || currentSession.generationId !== generationId
        || !adultStore.requestFor(roleId, sid, 'continue')
      ) {
        void cancelKnownGeneration(roleId, sid, generationId)
        return
      }
      const pending = new Map(
        restored.beats.map(beat => [beat.sequence, beat] as const),
      )
      if (!restored.active && pending.size === 0) {
        clearSessionGenerationIfMatches(roleId, sid, generationId)
        return
      }
      const nextCommit = restored.beats.length > 0
        ? Math.min(...restored.beats.map(beat => beat.sequence))
        : restored.next_sequence
      const queue: AdultBeatQueueRuntime = {
        key,
        roleId,
        sceneId: sid,
        generationId,
        nextSequence: restored.next_sequence,
        nextCommit,
        pending,
        generating: false,
        generationDone: !restored.active
          || restored.beats.some(
            beat => beat.response.adult_beat?.interaction_state !== 'active',
          ),
        stopped: false,
        displaying: false,
        commitSettled: null,
        previousVoiceTurnId: null,
        previousIntervalMs: 1,
        hooks,
      }
      queues.set(key, queue)
      void pumpGeneration()
      void displayLoop(queue)
    }
    catch (error) {
      hooks.reportError?.(
        error instanceof Error ? error.message : String(error),
      )
      void cancelKnownGeneration(roleId, sid, generationId)
    }
  })
}

export async function cancelAdultBeatQueue(
  roleId: string,
  sceneId: string,
): Promise<void> {
  const sid = sceneId || 'default'
  const key = queueKey(roleId, sid)
  invalidateQueueIntent(key)
  const setupTail = queueSetupTails.get(key)
  const runtime = queues.get(key)
  const generationIds = new Set<string>()
  if (runtime?.generationId)
    generationIds.add(runtime.generationId)
  const persistedBefore = useAdultInteractionStore()
    .sessionFor(roleId, sid)
    .generationId
  if (persistedBefore)
    generationIds.add(persistedBefore)
  if (runtime) {
    detachQueue(runtime)
    // A beat that crossed the commit boundary is already ordered for display.
    // Let it settle before cancelling only the later, still-unshown beats.
    await runtime.commitSettled?.catch(() => undefined)
  }
  // Begin/list are local DB operations and must settle before cancellation is
  // considered complete. A late generation persists its id before issuing its
  // own cancel, so a failed cancel remains retryable here and after restart.
  await setupTail?.catch(() => undefined)
  const persistedAfter = useAdultInteractionStore()
    .sessionFor(roleId, sid)
    .generationId
  if (persistedAfter)
    generationIds.add(persistedAfter)
  cleanupQueueIntentIfIdle(key)
  for (const generationId of generationIds)
    await cancelKnownGeneration(roleId, sid, generationId)
}

export async function cancelAllAdultBeatQueues(): Promise<void> {
  const targets = new Map<string, { roleId: string, sceneId: string }>()
  for (const queue of queues.values()) {
    targets.set(queue.key, {
      roleId: queue.roleId,
      sceneId: queue.sceneId,
    })
  }
  for (const [key, target] of pendingSetupTargets)
    targets.set(key, { roleId: target.roleId, sceneId: target.sceneId })
  for (const target of persistedGenerationTargets())
    targets.set(queueKey(target.roleId, target.sceneId), target)
  await Promise.all(
    [...targets.values()].map(target =>
      cancelAdultBeatQueue(target.roleId, target.sceneId),
    ),
  )
}

export async function cancelAdultBeatQueuesForRole(roleId: string): Promise<void> {
  const scenes = new Set(
    [...queues.values()]
      .filter(queue => queue.roleId === roleId)
      .map(queue => queue.sceneId),
  )
  for (const target of pendingSetupTargets.values()) {
    if (target.roleId === roleId)
      scenes.add(target.sceneId)
  }
  for (const target of persistedGenerationTargets()) {
    if (target.roleId === roleId)
      scenes.add(target.sceneId)
  }
  await Promise.all(
    [...scenes].map(sceneId => cancelAdultBeatQueue(roleId, sceneId)),
  )
}

export function notifyAdultBeatQueueCapacityChanged(): void {
  void pumpGeneration()
}
