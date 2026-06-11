import type {
  DirectorPhase,
  TheaterSession,
  TheaterSessionTurn,
  TheaterSpeaker,
  TheaterTurnSpeaker,
} from './types'
import { computed, ref } from 'vue'

let turnCounter = 0

function nextTurnId(): string {
  turnCounter += 1
  return `turn_${turnCounter}`
}

/** After user speaks → A; after A → B; after B → user. */
export function nextSpeakerAfter(speaker: TheaterTurnSpeaker): TheaterSpeaker | 'user' {
  if (speaker === 'user') {
    return 'a'
  }
  if (speaker === 'a') {
    return 'b'
  }
  return 'user'
}

export function useTheaterDirector(
  sceneMeta: () => { scene_id: string, title: string, role_a: string, role_b: string },
) {
  const turns = ref<TheaterSessionTurn[]>([])
  const phase = ref<DirectorPhase>('waiting_user')
  const maxRounds = ref(6)

  const roundCount = computed(() =>
    turns.value.filter(t => t.speaker === 'user').length,
  )

  const canUserSpeak = computed(() =>
    phase.value === 'waiting_user' && roundCount.value < maxRounds.value,
  )

  const session = computed((): TheaterSession => {
    const meta = sceneMeta()
    return {
      schema_version: 1,
      scene_id: meta.scene_id,
      title: meta.title,
      role_a: meta.role_a,
      role_b: meta.role_b,
      turns: turns.value,
    }
  })

  function resetDirector() {
    turns.value = []
    phase.value = 'waiting_user'
    turnCounter = 0
  }

  function submitUserLine(text: string) {
    const trimmed = text.trim()
    if (!trimmed || !canUserSpeak.value) {
      return false
    }
    turns.value = [
      ...turns.value,
      { id: nextTurnId(), speaker: 'user', text: trimmed },
    ]
    phase.value = 'generating_a'
    return true
  }

  function appendOcLine(speaker: TheaterSpeaker, text: string) {
    const trimmed = text.trim()
    if (!trimmed) {
      return false
    }
    turns.value = [
      ...turns.value,
      { id: nextTurnId(), speaker, text: trimmed },
    ]
    const next = nextSpeakerAfter(speaker)
    if (next === 'user') {
      phase.value = roundCount.value >= maxRounds.value ? 'ended' : 'waiting_user'
    }
    else {
      phase.value = next === 'a' ? 'generating_a' : 'generating_b'
    }
    return true
  }

  function pendingSpeaker(): TheaterSpeaker | null {
    if (phase.value === 'generating_a') {
      return 'a'
    }
    if (phase.value === 'generating_b') {
      return 'b'
    }
    return null
  }

  function endSession() {
    phase.value = 'ended'
  }

  return {
    turns,
    phase,
    roundCount,
    maxRounds,
    canUserSpeak,
    session,
    resetDirector,
    submitUserLine,
    appendOcLine,
    pendingSpeaker,
    endSession,
  }
}
