import type { SendMessageResponse } from '@oclive/shared/api'
import { sendMessage, sendMessageStream } from '@oclive/shared/api'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { VOICE_STREAM_SENTENCE_EVENT } from '@oclive/shared/lib/voiceAsrEvents'
import { getRelationUpgradeMessage } from '@oclive/shared/utils/relation'
import { presentationFromSendResponse } from '@oclive/shared/utils/replyPresentation'
import { isChatStreamEnabled } from '@oclive/shared/utils/chatStreamSettings'
import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '@oclive/shared/utils/roleplayReplySplit'
import { StreamingVoiceChunker } from '@oclive/shared/utils/streamingVoiceChunker'
import { parseMessageTimestamp } from './chatStoreLoad'
import { useDebugStore } from './debugStore'
import { useRoleStore } from './roleStore'
import type { ChatMessage, SceneHistorySplitIndex } from './chatStore'

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

function isAbortError(err: unknown, signal?: AbortSignal): boolean {
  if (signal?.aborted)
    return true
  return err instanceof DOMException && err.name === 'AbortError'
}

export async function sendChatStoreMessage(
  context: ChatStoreSendContext,
  content: string,
  sceneId: string,
): Promise<SendMessageResponse | void> {
  const roleStore = useRoleStore()
  const roleId = roleStore.currentRoleId
  const sid = sceneId || 'default'
  const countBeforeTurn = context.getMessageCountForRoleScene(roleId, sid)
  const userLocalId = `u-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  const sendSeq = ++activeSendSeq
  const isStale = () => sendSeq !== activeSendSeq

  inFlightStreamAbort?.abort()
  const streamAbort = new AbortController()
  inFlightStreamAbort = streamAbort

  context.addMessage(roleId, sid, {
    id: userLocalId,
    role: 'user',
    content,
    timestamp: Date.now(),
  }, { persistIdbCache: false })
  context.setLoading(true)
  hostEventBus.emitBuiltin('message:submit', { role_id: roleId, scene_id: sid })
  const relationBefore = roleStore.roleInfo.relationState
    const assistantLocalId = `a-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
    const streamId = assistantLocalId
    let streamBubbleActive = false
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
    const streamEnabled = isChatStreamEnabled()
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
          },
          {
            signal: streamAbort.signal,
            onToken: (_token, accumulated) => {
              if (isStale())
                return
              lastStreamAccumulated = accumulated
              context.patchMessageById(roleId, sid, assistantLocalId, {
                content: accumulated,
              })
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
          return
        }
        console.warn('[chat] stream failed, fallback to /chat', streamErr)
        if (streamBubbleActive) {
          context.deleteMessage(roleId, sid, assistantLocalId)
          streamBubbleActive = false
        }
        res = await sendMessage({
          role_id: roleId,
          user_message: content,
          scene_id: sid || null,
        })
      }
    }
    else {
      res = await sendMessage({
        role_id: roleId,
        user_message: content,
        scene_id: sid || null,
      })
    }

    if (isStale())
      return

    if (res.user_message_id) {
      context.patchMessageById(roleId, sid, userLocalId, {
        id: res.user_message_id,
        timestamp: parseMessageTimestamp(res.user_message_timestamp),
      })
    }

    const pres = presentationFromSendResponse(res)
    const preSplit = splitRoleplayReply(pres.replyText)
    const aside = preSplit.aside.trim()
    const dialogue = assistantDialogueFromSplit(pres.replyText, preSplit)
    const assistantMsg: ChatMessage = {
      id: res.assistant_message_id
        ?? assistantLocalId,
      role: 'assistant',
      content: dialogue,
      timestamp: parseMessageTimestamp(res.assistant_message_timestamp),
      emotion: pres.assistantEmotionLabel,
      presenceVariant: pres.presenceVariant,
      replyIsFallback: pres.replyIsFallback,
      streaming: false,
      ...(aside.length > 0 ? { aside } : {}),
    }

    if (streamBubbleActive) {
      context.deleteMessage(roleId, sid, assistantLocalId)
    }
    context.addMessage(roleId, sid, assistantMsg, { persistIdbCache: false })

    useDebugStore().recordKnowledgeFromSend(res)
    roleStore.updateLocalAfterMessage(
      pres.assistantEmotionLabel,
      res.favorability_current,
      {
        visualStateId: res.visual_state_id ?? null,
        portraitAssetPath:
          res.performance_directive?.path
          ?? res.performance_directive?.fallback_image
          ?? null,
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
      message: content,
      reply: assistantDialogueFromSplit(pres.replyText, preSplit),
      reply_aside: preSplit.aside,
      bot_emotion: res.bot_emotion,
      role_id: roleId,
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
    return res
  }
  catch (err) {
    if (isAbortError(err, streamAbort.signal) || isStale()) {
      if (streamBubbleActive)
        context.deleteMessage(roleId, sid, assistantLocalId)
      return
    }
    context.deleteMessage(roleId, sid, userLocalId)
    if (streamBubbleActive)
      context.deleteMessage(roleId, sid, assistantLocalId)
    throw err
  }
  finally {
    if (!isStale())
      context.setLoading(false)
    if (inFlightStreamAbort === streamAbort)
      inFlightStreamAbort = null
  }
}
