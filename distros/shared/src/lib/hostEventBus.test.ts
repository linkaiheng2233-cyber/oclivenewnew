import { describe, expect, it, vi } from 'vitest'
import { hostEventBus } from './hostEventBus'

describe('hostEventBus', () => {
  it('always delivers internal host voice lifecycle events', () => {
    const onSubmit = vi.fn()
    const onSentence = vi.fn()
    hostEventBus.on('message:submit', onSubmit)
    hostEventBus.on('com.oclive.voice:stream-sentence', onSentence)

    hostEventBus.emitBuiltin('message:submit', { role_id: 'mumu' })
    hostEventBus.emitBuiltin('com.oclive.voice:stream-sentence', { sentence: '你好。' })

    expect(onSubmit).toHaveBeenCalledWith({ role_id: 'mumu' })
    expect(onSentence).toHaveBeenCalledWith({ sentence: '你好。' })
    hostEventBus.off('message:submit', onSubmit)
    hostEventBus.off('com.oclive.voice:stream-sentence', onSentence)
  })
})
