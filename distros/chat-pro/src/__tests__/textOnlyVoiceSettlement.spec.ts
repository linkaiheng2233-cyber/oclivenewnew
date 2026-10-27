// @vitest-environment jsdom
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { clearVoicePlaybackSettlements, markVoicePlaybackSettled, waitForVoicePlaybackSettled } from '@oclive/shared/lib/voicePlaybackSettlement'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, ref } from 'vue'
import { useTextOnlyVoiceSettlement } from '../composables/useTextOnlyVoiceSettlement'

const wrappers: Array<ReturnType<typeof mount>> = []
function fixture(enabled = false) {
  const active = ref(enabled)
  const wrapper = mount(defineComponent({
    setup() {
      useTextOnlyVoiceSettlement(() => active.value)
      return () => null
    },
  }))
  wrappers.push(wrapper)
  return active
}

describe('text-only playback settlement', () => {
  beforeEach(() => {
    clearVoicePlaybackSettlements()
    vi.useFakeTimers()
  })
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    clearVoicePlaybackSettlements()
    vi.useRealTimers()
  })

  it('immediately releases pacing for new text turns without a voice host', async () => {
    fixture()
    hostEventBus.emitBuiltin('message:sent', { turn_id: 'text-turn' })
    await expect(waitForVoicePlaybackSettled('text-turn')).resolves.toBe('disabled')
    expect(vi.getTimerCount()).toBe(0)
  })

  it('releases an already waiting turn when advanced voice closes', async () => {
    const active = fixture(true)
    const waiting = waitForVoicePlaybackSettled('pending-turn')
    expect(vi.getTimerCount()).toBe(1)
    active.value = false
    await expect(waiting).resolves.toBe('disabled')
    expect(vi.getTimerCount()).toBe(0)
  })

  it('remembers a disabled turn when its scheduler starts waiting only after closing', async () => {
    const active = fixture(true)
    hostEventBus.emitBuiltin('message:sent', { turn_id: 'future-waiter' })
    active.value = false
    const result = vi.fn()
    void waitForVoicePlaybackSettled('future-waiter').then(result)
    await Promise.resolve()
    expect(result).toHaveBeenCalledWith('disabled')
    expect(vi.getTimerCount()).toBe(0)
  })

  it('leaves enabled voice responsible for completing its own turn', async () => {
    fixture(true)
    hostEventBus.emitBuiltin('message:sent', { turn_id: 'audio-turn' })
    const waiting = waitForVoicePlaybackSettled('audio-turn')
    expect(vi.getTimerCount()).toBe(1)
    markVoicePlaybackSettled('audio-turn', 'complete')
    await expect(waiting).resolves.toBe('complete')
  })
})
