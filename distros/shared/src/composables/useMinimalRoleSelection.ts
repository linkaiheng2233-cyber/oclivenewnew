import type { MinimalRoleLocalSource } from '@oclive/shared/api/chat'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { snapshotMinimalRoleSource, useMinimalRoleChatStore } from '@oclive/shared/stores/minimalRoleChatStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { effectiveChatSceneId } from '@oclive/shared/utils/pureChatScene'

/** Frontend context selection; does not activate or fabricate a rich Host role. */
export function useMinimalRoleSelection() {
  const chat = useChatStore()
  const minimal = useMinimalRoleChatStore()
  const role = useRoleStore()
  const ui = useUiStore()
  let selectionGeneration = 0

  async function bindSource(input: MinimalRoleLocalSource): Promise<void> {
    const snapshot = snapshotMinimalRoleSource(input)
    const ownGeneration = ++selectionGeneration
    const richRoleId = role.currentRoleId
    chat.cancelPendingSend()
    minimal.cancelPendingSend()
    if (richRoleId) {
      await chat.clearAdultInteractionForContextChange(
        richRoleId,
        effectiveChatSceneId(role.roleInfo.interactionMode, ui.sceneId),
      )
    }
    // A return/switch/new selection during queue cancellation wins.
    if (ownGeneration !== selectionGeneration || role.currentRoleId !== richRoleId)
      return
    chat.cancelPendingSend()
    minimal.bindSource(snapshot)
  }

  function returnToRichRole(): void {
    selectionGeneration += 1
    minimal.unbindSource()
  }

  return { bindSource, returnToRichRole }
}
