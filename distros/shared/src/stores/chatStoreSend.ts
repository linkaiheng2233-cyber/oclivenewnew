import type { AdultInteractionAction, SendMessageResponse } from '@oclive/shared/api'
import type { ChatMessage, SceneHistorySplitIndex } from './chatStore'
import { sendMessage, sendMessageStream, toastAsyncError } from '@oclive/shared/api'
import {
  cancelAdultBeatQueue,
  resumeAdultBeatQueue,
  startAdultBeatQueue,
} from '@oclive/shared/lib/adultBeatQueue'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { VOICE_STREAM_SENTENCE_EVENT } from '@oclive/shared/lib/voiceAsrEvents'
import { waitForVoicePlaybackSettled } from '@oclive/shared/lib/voicePlaybackSettlement'
import { isChatStreamEnabled } from '@oclive/shared/utils/chatStreamSettings'
import { getRelationUpgradeMessage } from '@oclive/shared/utils/relation'
import { presentationFromSendResponse } from '@oclive/shared/utils/replyPresentation'
import { draftsFromPresentation, segmentMessageIds, splitReplyBySeparatorLine } from '@oclive/shared/utils/replySegments'
import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '@oclive/shared/utils/roleplayReplySplit'
import { StreamingVoiceChunker } from '@oclive/shared/utils/streamingVoiceChunker'
import { useAdultInteractionStore } from './adultInteractionStore'
import { parseMessageTimestamp } from './chatStoreLoad'
import { useDebugStore } from './debugStore'
import { useRoleStore } from './roleStore'
import { useUiStore } from './uiStore'

export interface ChatStoreSendContext {
  sceneHistorySplitIndex: SceneHistorySplitIndex
  setLoading: (loading: boolean) => void
  getMessageCountForRoleScene: (roleId: string, sceneId: string) => number
  addMessage: (
    roleId: string,
    sceneId: string,
    msg: ChatMessage,
    options?: { persistIdbCache?: boolean },
  ) => void
  patchMessageById: (
    roleId: string,
    sceneId: string,
    localId: string,
    patch: Partial<Pick<ChatMessage, 'id' | 'timestamp' | 'content' | 'streaming' | 'emotion' | 'aside'>>,
  ) => void
  deleteMessage: (roleId: string, sceneId: string, messageId: string) => void
  addSystemMessage: (content: string, sceneId?: string) => void
  clampSceneHistorySplitForBucket: (
    splitIndex: SceneHistorySplitIndex,
    roleId: string,
    sceneId: string,
    messageCount: number,
    sessionFloor?: number,
  ) => void
}

let activeSendSeq = 0
let inFlightStreamAbort: AbortController | null = null
let adultScheduleSeq = 0

export interface ChatStoreSendOptions {
  adultAction?: AdultInteractionAction
  hideUserMessage?: boolean
}

function cancelAdultBeatSchedule(): void {
  adultScheduleSeq += 1
}

export function cancelActiveChatSend(): void {
  activeSendSeq += 1
  cancelAdultBeatSchedule()
  inFlightStreamAbort?.abort()
  inFlightStreamAbort = null
}

function isAbortError(err: unknown, signal?: AbortSignal): boolean {
  if (signal?.aborted)
    return true
  return err instanceof DOMException && err.name === 'AbortError'
}

export async function sendChatStoreMessage(
  context: ChatStoreSendContext,
  content: string,
  sceneId: string,
  options: ChatStoreSendOptions = {},
): Promise<SendMessageResponse | undefined> {
  const roleStore = useRoleStore()
  const adultStore = useAdultInteractionStore()
  const uiStore = useUiStore()
  const roleId = roleStore.currentRoleId
  const sid = sceneId || 'default'
  if (options.adultAction !== 'continue')
    await cancelAdultBeatQueue(roleId, sid)
  const adultRequest = roleStore.roleInfo.adultExtensionAvailable
    ? adultStore.requestFor(roleId, sid, options.adultAction ?? 'message')
    : undefined
  if (options.adultAction && !adultRequest)
    return
  cancelAdultBeatSchedule()
  const countBeforeTurn = context.getMessageCountForRoleScene(roleId, sid)
  const userLocalId = `u-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  const sendSeq = ++activeSendSeq
  const isStale = () =>
    sendSeq !== activeSendSeq
    || roleStore.currentRoleId !== roleId
    || (uiStore.sceneId || 'default') !== sid

  inFlightStreamAbort?.abort()
  const streamAbort = new AbortController()
  inFlightStreamAbort = streamAbort

  if (!options.hideUserMessage) {
    context.addMessage(roleId, sid, {
      id: userLocalId,
      role: 'user',
      content,
      timestamp: Date.now(),
    }, { persistIdbCache: false })
  }
  context.setLoading(true)
  const assistantLocalId = `a-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  const streamId = assistantLocalId
  hostEventBus.emitBuiltin('message:submit', {
    role_id: roleId,
    scene_id: sid,
    stream_id: streamId,
    submitted_at_ms: Date.now(),
  })
  const relationBefore = roleStore.roleInfo.relationState
  const replyMode = roleStore.roleInfo.replyMode
  const replySeparator = replyMode?.separator ?? ''
  const replySegmentsCap = replyMode?.segments ?? 2
  let streamBubbleActive = false
  let secondStreamBubbleActive = false
  let streamSpokenPrefix = ''
  let lastStreamAccumulated = ''
  const voiceChunker = new StreamingVoiceChunker()

  function emitStreamVoiceChunks(chunks: string[]): void {
    for (const chunk of chunks) {
      streamSpokenPrefix += chunk
      hostEventBus.emitBuiltin(VOICE_STREAM_SENTENCE_EVENT, {
        sentence: chunk,
        stream_id: streamId,
        role_id: roleId,
        bot_emotion: 'neutral',
      })
    }
  }
  try {
    let res: SendMessageResponse
    // Structured adult output must be parsed as one complete envelope. Ordinary
    // role packs retain the low-TTFT SSE path.
    const streamEnabled = isChatStreamEnabled() && !adultRequest
    if (streamEnabled) {
      try {
        context.addMessage(roleId, sid, {
          id: assistantLocalId,
          role: 'assistant',
          content: '',
          timestamp: Date.now(),
          streaming: true,
        }, { persistIdbCache: false })
        streamBubbleActive = true
        res = await sendMessageStream(
          {
            role_id: roleId,
            user_message: content,
            scene_id: sid || null,
            adult: adultRequest,
          },
          {
            signal: streamAbort.signal,
            onToken: (_token, accumulated) => {
              if (isStale())
                return
              lastStreamAccumulated = accumulated
              if (!replyMode || !replySeparator) {
                context.patchMessageById(roleId, sid, assistantLocalId, {
                  content: accumulated,
                })
              }
              else {
                const segments = splitReplyBySeparatorLine(
                  accumulated,
                  replySeparator,
                  replySegmentsCap,
                )
                context.patchMessageById(roleId, sid, assistantLocalId, {
                  content: segments[0] ?? '',
                })
                if (segments.length > 1 && !secondStreamBubbleActive) {
                  context.addMessage(roleId, sid, {
                    id: `${assistantLocalId}#s1`,
                    role: 'assistant',
                    content: '',
                    timestamp: Date.now(),
                    streaming: true,
                  }, { persistIdbCache: false })
                  secondStreamBubbleActive = true
                }
                if (secondStreamBubbleActive) {
                  context.patchMessageById(roleId, sid, `${assistantLocalId}#s1`, {
                    content: segments[1] ?? '',
                  })
                }
              }
              emitStreamVoiceChunks(voiceChunker.push(accumulated))
            },
          },
        )
        emitStreamVoiceChunks(voiceChunker.flush(lastStreamAccumulated))
      }
      catch (streamErr) {
        if (isAbortError(streamErr, streamAbort.signal) || isStale()) {
          if (streamBubbleActive)
            context.deleteMessage(roleId, sid, assistantLocalId)
          if (secondStreamBubbleActive)
            context.deleteMessage(roleId, sid, `${assistantLocalId}#s1`)
          return
        }
        console.warn('[chat] stream failed, fallback to /chat', streamErr)
        if (streamBubbleActive) {
          context.deleteMessage(roleId, sid, assistantLocalId)
          streamBubbleActive = false
        }
        if (secondStreamBubbleActive) {
          context.deleteMessage(roleId, sid, `${assistantLocalId}#s1`)
          secondStreamBubbleActive = false
        }
        res = await sendMessage({
          role_id: roleId,
          user_message: content,
          scene_id: sid || null,
          adult: adultRequest,
        })
      }
    }
    else {
      res = await sendMessage({
        role_id: roleId,
        user_message: content,
        scene_id: sid || null,
        adult: adultRequest,
      })
    }

    if (isStale())
      return

    if (res.user_message_id && !options.hideUserMessage) {
      context.patchMessageById(roleId, sid, userLocalId, {
        id: res.user_message_id,
        timestamp: parseMessageTimestamp(res.user_message_timestamp),
      })
    }

    const pres = presentationFromSendResponse(res)
    const preSplit = res.adult_beat
      ? {
          dialogue: res.adult_beat.dialogue,
          aside: res.adult_beat.narration,
        }
      : splitRoleplayReply(pres.replyText)
    const aside = preSplit.aside.trim()
    const drafts = res.adult_beat
      ? [{ text: preSplit.dialogue, delayMs: 0 }]
      : draftsFromPresentation(pres.replyText, res.reply_presentation)

    if (streamBubbleActive) {
      context.deleteMessage(roleId, sid, assistantLocalId)
    }
    if (secondStreamBubbleActive)
      context.deleteMessage(roleId, sid, `${assistantLocalId}#s1`)
    const baseId = res.assistant_message_id ?? assistantLocalId
    const baseTimestamp = parseMessageTimestamp(res.assistant_message_timestamp)
    const segmentIds = segmentMessageIds(baseId, drafts.length)
    for (let i = 0; i < drafts.length; i++) {
      const delayMs = i === 0 ? 0 : drafts[i].delayMs
      if (delayMs > 0)
        await new Promise<void>(resolve => window.setTimeout(resolve, delayMs))
      if (isStale())
        return
      const segmentSplit = splitRoleplayReply(drafts[i].text)
      const segmentDialogue = assistantDialogueFromSplit(drafts[i].text, segmentSplit)
      context.addMessage(roleId, sid, {
        id: segmentIds[i],
        role: 'assistant',
        content: segmentDialogue,
        timestamp: baseTimestamp + i,
        emotion: pres.assistantEmotionLabel,
        presenceVariant: pres.presenceVariant,
        replyIsFallback: pres.replyIsFallback,
        streaming: false,
        ...(i === drafts.length - 1 && aside.length > 0 ? { aside } : {}),
      }, { persistIdbCache: false })
    }
    const voiceTextOnlyForTurn = adultStore.sessionFor(roleId, sid).voiceTextOnly
    if (res.adult_beat) {
      adultStore.updateSession(
        roleId,
        sid,
        res.adult_beat.interaction_state,
      )
    }

    useDebugStore().recordKnowledgeFromSend(res)
    roleStore.updateLocalAfterMessage(
      pres.assistantEmotionLabel,
      res.favorability_current,
      {
        visualStateId: res.visual_state_id,
        portraitAssetPath:
          res.performance_directive?.path
          ?? res.performance_directive?.fallback_image,
      },
    )
    if (res.relation_state) {
      const tip = getRelationUpgradeMessage(
        res.relation_state,
        relationBefore,
      )
      if (tip)
        context.addSystemMessage(tip, sid)
      roleStore.updateRelationState(res.relation_state)
    }
    hostEventBus.emitBuiltin('message:sent', {
      message: options.hideUserMessage ? '' : content,
      reply: assistantDialogueFromSplit(pres.replyText, preSplit),
      reply_aside: preSplit.aside,
      bot_emotion: res.bot_emotion,
      role_id: roleId,
      scene_id: sid,
      turn_id: streamId,
      skip_auto_tts: voiceTextOnlyForTurn,
      stream_id: streamBubbleActive ? streamId : undefined,
      stream_spoken_prefix: streamBubbleActive ? streamSpokenPrefix : undefined,
      stream_full_raw: streamBubbleActive ? lastStreamAccumulated : undefined,
      stream_spoken_end_index: streamBubbleActive ? voiceChunker.rawEndIndex : undefined,
    })
    const countAfterTurn = context.getMessageCountForRoleScene(roleId, sid)
    context.clampSceneHistorySplitForBucket(
      context.sceneHistorySplitIndex,
      roleId,
      sid,
      countAfterTurn,
      countBeforeTurn,
    )
    if (adultStore.backgroundQueueEnabled) {
      void startAdultBeatQueue(
        roleId,
        sid,
        res,
        streamId,
        adultQueueHooks(context),
      ).catch(toastAsyncError)
    }
    else {
      scheduleNextAdultBeat(context, roleId, sid, res, streamId)
    }
    return res
  }
  catch (err) {
    if (isAbortError(err, streamAbort.signal) || isStale()) {
      if (streamBubbleActive)
        context.deleteMessage(roleId, sid, assistantLocalId)
      if (secondStreamBubbleActive)
        context.deleteMessage(roleId, sid, `${assistantLocalId}#s1`)
      return
    }
    if (!options.hideUserMessage)
      context.deleteMessage(roleId, sid, userLocalId)
    if (streamBubbleActive)
      context.deleteMessage(roleId, sid, assistantLocalId)
    if (secondStreamBubbleActive)
      context.deleteMessage(roleId, sid, `${assistantLocalId}#s1`)
    throw err
  }
  finally {
    if (!isStale())
      context.setLoading(false)
    if (inFlightStreamAbort === streamAbort)
      inFlightStreamAbort = null
  }
}

async function displayCommittedAdultBeat(
  context: ChatStoreSendContext,
  response: SendMessageResponse,
  roleId: string,
  sceneId: string,
  turnId: string,
): Promise<void> {
  const adultStore = useAdultInteractionStore()
  const roleStore = useRoleStore()
  const countBeforeTurn = context.getMessageCountForRoleScene(roleId, sceneId)
  const presentation = presentationFromSendResponse(response)
  const split = response.adult_beat
    ? {
        dialogue: response.adult_beat.dialogue,
        aside: response.adult_beat.narration,
      }
    : splitRoleplayReply(presentation.replyText)
  const aside = split.aside.trim()
  const dialogue = assistantDialogueFromSplit(presentation.replyText, split)
  const voiceTextOnlyForTurn = adultStore.sessionFor(roleId, sceneId).voiceTextOnly
  context.addMessage(roleId, sceneId, {
    id: response.assistant_message_id ?? turnId,
    role: 'assistant',
    content: dialogue,
    timestamp: parseMessageTimestamp(response.assistant_message_timestamp),
    emotion: presentation.assistantEmotionLabel,
    presenceVariant: presentation.presenceVariant,
    replyIsFallback: presentation.replyIsFallback,
    ...(aside ? { aside } : {}),
  }, { persistIdbCache: false })
  if (response.adult_beat) {
    adultStore.updateSession(
      roleId,
      sceneId,
      response.adult_beat.interaction_state,
    )
  }
  useDebugStore().recordKnowledgeFromSend(response)
  roleStore.updateLocalAfterMessage(
    presentation.assistantEmotionLabel,
    response.favorability_current,
    {
      visualStateId: response.visual_state_id,
      portraitAssetPath:
        response.performance_directive?.path
        ?? response.performance_directive?.fallback_image,
    },
  )
  if (response.relation_state)
    roleStore.updateRelationState(response.relation_state)
  hostEventBus.emitBuiltin('message:sent', {
    message: '',
    reply: dialogue,
    reply_aside: split.aside,
    bot_emotion: response.bot_emotion,
    role_id: roleId,
    scene_id: sceneId,
    turn_id: turnId,
    skip_auto_tts: voiceTextOnlyForTurn,
  })
  context.clampSceneHistorySplitForBucket(
    context.sceneHistorySplitIndex,
    roleId,
    sceneId,
    context.getMessageCountForRoleScene(roleId, sceneId),
    countBeforeTurn,
  )
}

function adultQueueHooks(context: ChatStoreSendContext) {
  return {
    display: (
      response: SendMessageResponse,
      roleId: string,
      sceneId: string,
      turnId: string,
    ) => displayCommittedAdultBeat(context, response, roleId, sceneId, turnId),
    reportError: (message: string) => {
      console.warn('[adult-stage]', message)
      toastAsyncError(new Error(message))
    },
  }
}

export function resumeAdultBeatQueueForChat(
  context: ChatStoreSendContext,
  roleId: string,
  sceneId: string,
): void {
  void resumeAdultBeatQueue(
    roleId,
    sceneId || 'default',
    adultQueueHooks(context),
  )
}

function scheduleNextAdultBeat(
  context: ChatStoreSendContext,
  roleId: string,
  sceneId: string,
  response: SendMessageResponse,
  voiceTurnId: string,
): void {
  const beat = response.adult_beat
  if (!beat || beat.interaction_state !== 'active')
    return
  const adultStore = useAdultInteractionStore()
  const scheduleSeq = ++adultScheduleSeq
  const intervalMs = adultStore.pacingOverrideEnabled
    ? adultStore.pacingIntervalMs
    : (beat.next_beat_interval_ms ?? 4_000)

  void (async () => {
    // D21/D22: the interval starts as soon as both bubbles are committed. The
    // next beat waits for both that interval and this beat's voice, rather than
    // adding the full interval after voice playback has already completed.
    const intervalElapsed = new Promise<void>(
      resolve => window.setTimeout(resolve, Math.max(1, intervalMs)),
    )
    const voiceSettled = waitForVoicePlaybackSettled(voiceTurnId)
    const [voiceStatus] = await Promise.all([voiceSettled, intervalElapsed])
    if (scheduleSeq !== adultScheduleSeq)
      return
    if (voiceStatus === 'error' || voiceStatus === 'timeout')
      adultStore.markVoiceTextOnly(roleId, sceneId)
    const roleStore = useRoleStore()
    if (
      roleStore.currentRoleId !== roleId
      || !adultStore.gatesOpen
      || !adultStore.roleIsEnabled(roleId)
      || !adultStore.sessionFor(roleId, sceneId).active
    ) {
      return
    }
    await sendChatStoreMessage(
      context,
      '（系统：继续当前互动的下一拍，不要虚构用户的发言、动作、选择或感受。）',
      sceneId,
      {
        adultAction: 'continue',
        hideUserMessage: true,
      },
    )
  })()
}
