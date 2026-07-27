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
let pumpRunning = false
let fairCursor = 0

function queueKey(roleId: string, sceneId: string): string {
  return `${roleId.trim()}:${sceneId.trim() || 'default'}`
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
  const selected = candidates[fairCursor % candidates.length]
  fairCursor = (fairCursor + 1) % Math.max(1, candidates.length)
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
        if (!queue.stopped) {
          queue.generationDone = true
          queue.hooks.reportError?.(
            error instanceof Error ? error.message : String(error),
          )
        }
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
        queue.stopped = true
        queues.delete(queue.key)
        useAdultInteractionStore().setSessionGeneration(
          queue.roleId,
          queue.sceneId,
        )
        return
      }
      const committedSequence = queue.nextCommit
      const commitAndDisplay = async (): Promise<SendMessageResponse> => {
        const response = await commitAdultStagedBeat({
          role_id: queue.roleId,
          scene_id: queue.sceneId,
          generation_id: queue.generationId,
          sequence: committedSequence,
        })
        queue.pending.delete(committedSequence)
        queue.nextCommit = committedSequence + 1
        void pumpGeneration()
        const displayTurnId = `adult-stage-${queue.generationId}-${committedSequence}`
        await queue.hooks.display(
          response,
          queue.roleId,
          queue.sceneId,
          displayTurnId,
        )
        return response
      }
      queue.commitSettled = commitAndDisplay()
      let committed: SendMessageResponse
      try {
        committed = await queue.commitSettled
      }
      finally {
        queue.commitSettled = null
      }
      const turnId = `adult-stage-${queue.generationId}-${committedSequence}`
      queue.previousVoiceTurnId = turnId
      queue.previousIntervalMs = useAdultInteractionStore().pacingOverrideEnabled
        ? useAdultInteractionStore().pacingIntervalMs
        : (committed.adult_beat?.next_beat_interval_ms ?? 4_000)
      if (committed.adult_beat?.interaction_state !== 'active') {
        queue.generationDone = true
        queue.stopped = true
        queues.delete(queue.key)
        useAdultInteractionStore().setSessionGeneration(
          queue.roleId,
          queue.sceneId,
        )
        return
      }
    }
  }
  catch (error) {
    if (!queue.stopped) {
      queue.hooks.reportError?.(
        error instanceof Error ? error.message : String(error),
      )
    }
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
  if (queues.has(key))
    return
  const adult = adultStore.requestFor(roleId, sid, 'continue')
  if (!adult)
    return
  const begun = await beginAdultStageGeneration({
    role_id: roleId,
    scene_id: sid,
    adult,
  })
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
  adultStore.setSessionGeneration(roleId, sid, queue.generationId)
  void pumpGeneration()
  void displayLoop(queue)
}

export async function resumeAdultBeatQueue(
  roleId: string,
  sceneId: string,
  hooks: AdultBeatDisplayHooks,
): Promise<void> {
  const adultStore = useAdultInteractionStore()
  const sid = sceneId || 'default'
  const key = queueKey(roleId, sid)
  if (
    queues.has(key)
    || !adultStore.backgroundQueueEnabled
    || !adultStore.sessionFor(roleId, sid).active
  ) {
    return
  }
  const generationId = adultStore.sessionFor(roleId, sid).generationId
  if (!generationId)
    return
  try {
    const restored = await listAdultStagedBeats({
      role_id: roleId,
      scene_id: sid,
      generation_id: generationId,
    })
    const pending = new Map(
      restored.beats.map(beat => [beat.sequence, beat] as const),
    )
    if (!restored.active && pending.size === 0) {
      adultStore.setSessionGeneration(roleId, sid)
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
    adultStore.setSessionGeneration(roleId, sid)
    hooks.reportError?.(error instanceof Error ? error.message : String(error))
  }
}

export async function cancelAdultBeatQueue(
  roleId: string,
  sceneId: string,
): Promise<void> {
  const sid = sceneId || 'default'
  const key = queueKey(roleId, sid)
  const runtime = queues.get(key)
  const generationId = runtime?.generationId
    ?? useAdultInteractionStore().sessionFor(roleId, sid).generationId
  if (runtime) {
    runtime.stopped = true
    queues.delete(key)
    // A beat that crossed the commit boundary is already ordered for display.
    // Let it settle before cancelling only the later, still-unshown beats.
    await runtime.commitSettled?.catch(() => undefined)
  }
  useAdultInteractionStore().setSessionGeneration(roleId, sid)
  if (!generationId)
    return
  try {
    await cancelAdultStageGeneration({
      role_id: roleId,
      scene_id: sid,
      generation_id: generationId,
    })
  }
  catch (error) {
    console.warn('[adult-stage] cancel failed', error)
  }
  finally {
    void pumpGeneration()
  }
}

export async function cancelAllAdultBeatQueues(): Promise<void> {
  const targets = new Map<string, { roleId: string, sceneId: string }>()
  for (const queue of queues.values()) {
    targets.set(queue.key, {
      roleId: queue.roleId,
      sceneId: queue.sceneId,
    })
  }
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
