import { describe, expect, it } from 'vitest'
import { VoiceAsrSubmitDeduper } from './voiceAsrEvents'

describe('voiceAsrSubmitDeduper', () => {
  it('accepts each current submission id only once', () => {
    const deduper = new VoiceAsrSubmitDeduper()
    const payload = { text: '你好', mode: 'send' as const, submissionId: 'turn-1' }

    expect(deduper.accept(payload, 1_000)).toBe(true)
    expect(deduper.accept(payload, 2_000)).toBe(false)
    expect(deduper.accept({ ...payload, submissionId: 'turn-2' }, 2_000)).toBe(true)
  })

  it('coalesces duplicate legacy plugin events only inside the short retry window', () => {
    const deduper = new VoiceAsrSubmitDeduper()
    const payload = { text: ' 你好 ', mode: 'send' as const }

    expect(deduper.accept(payload, 1_000)).toBe(true)
    expect(deduper.accept({ ...payload, text: '你好' }, 2_000)).toBe(false)
    expect(deduper.accept(payload, 3_000)).toBe(true)
  })
})
