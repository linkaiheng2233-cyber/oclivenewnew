import { afterEach, describe, expect, it, vi } from 'vitest'
import { useAppToast } from './useAppToast'

describe('readable failure notifications', () => {
  afterEach(() => {
    useAppToast().dismissToast()
    vi.useRealTimers()
  })

  it('keeps an error visible for reading until explicitly dismissed', () => {
    vi.useFakeTimers()
    const { toast, showToast, dismissToast } = useAppToast()
    showToast('error', 'Model unavailable', 'LLM_ERROR: connection refused')
    vi.advanceTimersByTime(60_000)
    expect(toast.value.show).toBe(true)
    expect(toast.value.details).toContain('LLM_ERROR')
    dismissToast()
    expect(toast.value.show).toBe(false)
  })

  it('does not let an earlier success timer dismiss a later failure', () => {
    vi.useFakeTimers()
    const { toast, showToast } = useAppToast()
    showToast('success', 'saved')
    vi.advanceTimersByTime(3100)
    showToast('error', 'not connected', 'transport detail')
    vi.advanceTimersByTime(2000)
    expect(toast.value.show).toBe(true)
    expect(toast.value.message).toBe('not connected')
  })
})
