import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { clearVoicePlaybackSettlements, markVoicePlaybackSettled } from '@oclive/shared/lib/voicePlaybackSettlement'
import { onBeforeUnmount, onMounted, watch } from 'vue'

/** Keep existing text interaction pacing working without mounting a voice runtime. */
export function useTextOnlyVoiceSettlement(voiceHostEnabled: () => boolean): void {
  // A scheduler may register only after its asynchronous next-beat preparation.
  // Remember recent turns across closing, with the same bound as settlement history.
  const recentTurns = new Set<string>()
  function onMessageSent(payload: unknown): void {
    const turnId = (payload as { turn_id?: string } | null)?.turn_id
    if (typeof turnId !== 'string' || !turnId.trim())
      return
    recentTurns.add(turnId)
    if (recentTurns.size > 128)
      recentTurns.delete(recentTurns.values().next().value!)
    if (!voiceHostEnabled())
      markVoicePlaybackSettled(turnId, 'disabled')
  }

  watch(voiceHostEnabled, (enabled) => {
    if (!enabled) {
      clearVoicePlaybackSettlements()
      for (const turnId of recentTurns)
        markVoicePlaybackSettled(turnId, 'disabled')
    }
  }, { immediate: true, flush: 'sync' })

  onMounted(() => hostEventBus.on('message:sent', onMessageSent))
  onBeforeUnmount(() => {
    hostEventBus.off('message:sent', onMessageSent)
    clearVoicePlaybackSettlements()
  })
}
