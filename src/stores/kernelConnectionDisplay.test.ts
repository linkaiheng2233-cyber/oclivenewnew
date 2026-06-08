import { describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { KernelConnectionStatus } from '../api/kernel'
import { useKernelConnectionStore } from './kernelConnectionStore'

function status(
  partial: Partial<KernelConnectionStatus> & Pick<KernelConnectionStatus, 'mode' | 'healthy'>,
): KernelConnectionStatus {
  return {
    baseUrl: 'http://127.0.0.1:8420',
    port: 8420,
    binaryPath: null,
    kernelTier: null,
    ...partial,
  }
}

describe('kernelConnectionStore display', () => {
  it('shows checking while phase is checking', () => {
    setActivePinia(createPinia())
    const store = useKernelConnectionStore()
    store.phase = 'checking'
    expect(store.display.labelKey).toBe('kernel.status.checking')
    expect(store.display.clickable).toBe(false)
  })

  it('does not show attached when unhealthy', () => {
    setActivePinia(createPinia())
    const store = useKernelConnectionStore()
    store.phase = 'ready'
    store.applyStatus(status({ mode: 'attached', healthy: false }))
    expect(store.display.labelKey).toBe('kernel.status.offlineTapReconnect')
  })

  it('shows attached only when healthy', () => {
    setActivePinia(createPinia())
    const store = useKernelConnectionStore()
    store.phase = 'ready'
    store.applyStatus(status({ mode: 'attached', healthy: true }))
    expect(store.display.labelKey).toBe('kernel.status.connectedLocal')
    expect(store.display.ok).toBe(true)
  })
})
