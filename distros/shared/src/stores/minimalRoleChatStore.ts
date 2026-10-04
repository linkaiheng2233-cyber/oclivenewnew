import type { MinimalRoleLocalSource, MinimalRoleMessageResponse } from '@oclive/shared/api/chat'
import type { ChatMessage } from './chatStore'
import { sendMinimalMessage } from '@oclive/shared/api/chat'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'

/**
 * Transient state for a distro's main chat to consume. Binding is not Host
 * activation: the Host validates the explicit source on every text call.
 * No rich RoleInfo, historical context, stored row IDs or persistence is implied.
 */
export const useMinimalRoleChatStore = defineStore('minimal-role-chat', () => {
  const source = shallowRef<Readonly<MinimalRoleLocalSource> | null>(null)
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  let generation = 0
  let pendingUserId: string | null = null

  function removePendingUser(): void {
    if (pendingUserId)
      messages.value = messages.value.filter(message => message.id !== pendingUserId)
    pendingUserId = null
  }

  /** Cancel client presentation only; the non-streaming Host call may finish. */
  function cancelPendingSend(): void {
    generation += 1
    removePendingUser()
    isLoading.value = false
  }

  function bindSource(input: MinimalRoleLocalSource): void {
    if (!input.role_id.trim() || !input.asset_root.trim() || !input.definition_reference.trim())
      throw new Error('minimal source fields must not be blank')
    // Validate before replacing the old binding; copy to exclude caller mutation.
    const snapshot = Object.freeze({
      role_id: input.role_id,
      asset_root: input.asset_root,
      definition_reference: input.definition_reference,
    })
    cancelPendingSend()
    source.value = snapshot
    messages.value = []
  }

  function unbindSource(): void {
    cancelPendingSend()
    source.value = null
    messages.value = []
  }

  async function sendMessage(content: string): Promise<MinimalRoleMessageResponse | undefined> {
    const bound = source.value
    if (!bound)
      throw new Error('minimal source is not bound')
    if (!content.trim())
      throw new Error('minimal message must not be blank')
    cancelPendingSend()
    const ownGeneration = generation
    const isCurrent = () => ownGeneration === generation && source.value === bound
    const userId = `minimal-local-user-${crypto.randomUUID()}`
    const turnId = `minimal-local-turn-${crypto.randomUUID()}`
    pendingUserId = userId
    messages.value.push({ id: userId, role: 'user', content, timestamp: Date.now() })
    isLoading.value = true
    try {
      hostEventBus.emitBuiltin('message:submit', {
        role_id: bound.role_id,
        submitted_at_ms: Date.now(),
        skip_auto_tts: true,
      })
      // A synchronous listener may switch or cancel before transport starts.
      if (!isCurrent())
        return
      const response = await sendMinimalMessage({
        source: bound,
        message: { user_message: content, requirements: '' },
      })
      if (!isCurrent())
        return
      if (
        !response || response.role_id !== bound.role_id
        || response.product_extensions !== 'unavailable' || typeof response.reply !== 'string'
      ) {
        throw new Error('invalid minimal response identity or shape')
      }
      messages.value.push({
        id: `minimal-local-assistant-${crypto.randomUUID()}`,
        role: 'assistant',
        content: response.reply,
        timestamp: Date.now(),
      })
      pendingUserId = null
      hostEventBus.emitBuiltin('message:sent', {
        message: content,
        reply: response.reply,
        role_id: bound.role_id,
        product_extensions: response.product_extensions,
        turn_id: turnId,
        skip_auto_tts: true,
      })
      return response
    }
    catch (error) {
      if (!isCurrent())
        return
      removePendingUser()
      throw error
    }
    finally {
      if (isCurrent())
        isLoading.value = false
    }
  }

  return { source, messages, isLoading, bindSource, unbindSource, cancelPendingSend, sendMessage }
})
