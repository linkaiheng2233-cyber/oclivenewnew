import { describe, expect, it } from 'vitest'
import { VoiceSpeakDeduper } from './voiceSpeakDeduper'

describe('voiceSpeakDeduper', () => {
  it('rejects duplicate queueing while a key is pending', () => {
    const deduper = new VoiceSpeakDeduper()
    expect(deduper.markQueued('role|neutral|tts|你好')).toBe(true)
    expect(deduper.markQueued('role|neutral|tts|你好')).toBe(false)
  })

  it('rejects duplicate queueing after a key was spoken', () => {
    const deduper = new VoiceSpeakDeduper()
    const key = 'role|neutral|tts|你好'
    expect(deduper.markQueued(key)).toBe(true)
    deduper.finish(key, true)
    expect(deduper.markQueued(key)).toBe(false)
  })

  it('allows retry when a queued key failed before speaking', () => {
    const deduper = new VoiceSpeakDeduper()
    const key = 'role|neutral|tts|你好'
    expect(deduper.markQueued(key)).toBe(true)
    deduper.finish(key, false)
    expect(deduper.markQueued(key)).toBe(true)
  })

  it('allows the same key again after reset for a new turn', () => {
    const deduper = new VoiceSpeakDeduper()
    const key = 'role|neutral|tts|你好'
    expect(deduper.markQueued(key)).toBe(true)
    deduper.finish(key, true)
    deduper.reset()
    expect(deduper.markQueued(key)).toBe(true)
  })
})
