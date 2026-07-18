import { ApiInvokeError } from '@oclive/shared/api/helpers'
import { describe, expect, it } from 'vitest'
import {
  isKernelOfflineError,
  mapTheaterInvokeError,
  mapTheaterOutlineInvokeError,
} from './mapTheaterInvokeError'
import { SceneGenTimeoutError } from './theaterLogic'

describe('mapTheaterInvokeError', () => {
  it('maps client timeout', () => {
    const m = mapTheaterInvokeError(new SceneGenTimeoutError())
    expect(m.kind).toBe('client_timeout')
    expect(m.userMessageKey).toBe('theater.poke.sceneTimeout')
    expect(m.recoverable).toBe(true)
  })

  it('maps kernel offline', () => {
    const err = new ApiInvokeError({
      message: 'unavailable',
      code: 'KERNEL_HTTP_UNAVAILABLE',
      raw: '',
    })
    expect(isKernelOfflineError(err)).toBe(true)
    const m = mapTheaterInvokeError(err)
    expect(m.kind).toBe('kernel_offline')
    expect(m.userMessageKey).toBe('theater.poke.kernelOffline')
  })

  it('maps stale cast_rewrite kernel', () => {
    const err = new ApiInvokeError({
      message: 'bad',
      raw: 'base_beats must not be empty',
    })
    const m = mapTheaterInvokeError(err)
    expect(m.kind).toBe('kernel_stale_cast_rewrite')
  })

  it('outline mode uses outline.failed for generic invoke errors', () => {
    const m = mapTheaterOutlineInvokeError(new Error('boom'))
    expect(m.userMessageKey).toBe('theater.outline.failed')
  })
})
