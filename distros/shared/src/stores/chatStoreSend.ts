import type { SendMessageResponse } from '@oclive/shared/api'
import { sendMessage } from '@oclive/shared/api'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { getRelationUpgradeMessage } from '@oclive/shared/utils/relation'
import { presentationFromSendResponse } from '@oclive/shared/utils/replyPresentation'
import {
  assistantDialogueFromSplit,
  splitRoleplayReply,
} from '@oclive/shared/utils/roleplayReplySplit'
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
    patch: Partial<Pick<ChatMessage, 'id' | 'timestamp'>>,
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

export async function sendChatStoreMessage(
  context: ChatStoreSendContext,
  content: string,
  sceneId: string,
): Promise<SendMessageResponse> {
  const roleStore = useRoleStore()
  const roleId = roleStore.currentRoleId
  const sid = sceneId || 'default'
  const countBeforeTurn = context.getMessageCountForRoleScene(roleId, sid)
  const userLocalId = `u-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  context.addMessage(roleId, sid, {
    id: userLocalId,
    role: 'user',
    content,
    timestamp: Date.now(),
  }, { persistIdbCache: false })
  context.setLoading(true)
  const relationBefore = roleStore.roleInfo.relationState
  try {
    const res = await sendMessage({
      role_id: roleId,
      user_message: content,
      scene_id: sid || null,
    })
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
        ?? `a-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
      role: 'assistant',
      content: dialogue,
      timestamp: parseMessageTimestamp(res.assistant_message_timestamp),
      emotion: pres.assistantEmotionLabel,
      presenceVariant: pres.presenceVariant,
      replyIsFallback: pres.replyIsFallback,
      ...(aside.length > 0 ? { aside } : {}),
    }
    context.addMessage(roleId, sid, assistantMsg, { persistIdbCache: false })
    const split = preSplit
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
      reply: assistantDialogueFromSplit(pres.replyText, split),
      reply_aside: split.aside,
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
    context.deleteMessage(roleId, sid, userLocalId)
    throw err
  }
  finally {
    context.setLoading(false)
  }
}
