import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  clearVoicePlaybackSettlements,
  markVoicePlaybackSettled,
  waitForVoicePlaybackSettled,
} from './voicePlaybackSettlement'

describe('voice playback settlement', () => {
  beforeEach(() => {
    clearVoicePlaybackSettlements()
    vi.useRealTimers()
  })

  it('remembers an early completion until the beat scheduler subscribes', async () => {
    markVoicePlaybackSettled('turn-1', 'complete')
    await expect(waitForVoicePlaybackSettled('turn-1')).resolves.toBe('complete')
  })

  it('delivers a later voice error to a waiting scheduler', async () => {
    const waiting = waitForVoicePlaybackSettled('turn-2')
    markVoicePlaybackSettled('turn-2', 'error')
    await expect(waiting).resolves.toBe('error')
  })
})
