import type { PluginUiSlotInfo } from '@oclive/shared/api'
import type { PluginFrameRegistration } from '@oclive/shared/utils/pluginFrameBridge'
import { pluginBridgeInvoke } from '@oclive/shared/api'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { VOICE_ASR_PLUGIN_ID } from '@oclive/shared/lib/voiceAsrEvents'
import { createHostAudioCapture } from '@oclive/shared/utils/hostAudioCapture'
import { createPluginFrameBridge } from '@oclive/shared/utils/pluginFrameBridge'
import { onBeforeUnmount, onMounted } from 'vue'

export function usePluginFrameBridge(options: {
  onFrameError: (pluginId: string) => void
  onFrameLoad: (pluginId: string) => void
}) {
  const hostAudioCapture = createHostAudioCapture()
  const frameBridge = createPluginFrameBridge(pluginBridgeInvoke, {
    emit: (event, data) => hostEventBus.emit(event, data),
    subscribe: (event, handler) => {
      hostEventBus.on(event, handler)
      return () => hostEventBus.off(event, handler)
    },
    audioCapture: hostAudioCapture,
  })
  const registeredFrames = new Map<
    string,
    { element: HTMLIFrameElement, registration: PluginFrameRegistration }
  >()

  function frameKey(slot: PluginUiSlotInfo): string {
    return `${slot.pluginId}:${slot.appearanceId ?? ''}`
  }

  function framePermissions(_slot: PluginUiSlotInfo): undefined {
    // Opaque-origin frames cannot use getUserMedia. The trusted parent owns
    // microphone capture and exposes it through the source-bound broker.
    return undefined
  }

  function bindPluginFrame(slot: PluginUiSlotInfo, value: unknown): void {
    const key = frameKey(slot)
    const current = registeredFrames.get(key)
    if (current?.element === value)
      return
    current?.registration.unregister()
    registeredFrames.delete(key)

    if (!(value instanceof HTMLIFrameElement) || !value.contentWindow)
      return
    registeredFrames.set(key, {
      element: value,
      registration: frameBridge.register(value.contentWindow, {
        pluginId: slot.pluginId,
        assetRel: slot.entry,
        allowedEvents: slot.bridgeEvents,
        allowAudioCapture:
          slot.pluginId === VOICE_ASR_PLUGIN_ID && slot.entry === 'slots/toolbar.html',
      }),
    })
  }

  function onPluginFrameLoad(slot: PluginUiSlotInfo, event?: Event): void {
    const loadedFrame = event?.currentTarget
    if (loadedFrame instanceof HTMLIFrameElement)
      bindPluginFrame(slot, loadedFrame)
    const current = registeredFrames.get(frameKey(slot))
    if (!current?.registration.activate()) {
      options.onFrameError(slot.pluginId)
      return
    }
    options.onFrameLoad(slot.pluginId)
  }

  onMounted(() => window.addEventListener('message', frameBridge.handleMessage))
  onBeforeUnmount(() => {
    window.removeEventListener('message', frameBridge.handleMessage)
    for (const frame of registeredFrames.values())
      frame.registration.unregister()
    registeredFrames.clear()
    frameBridge.dispose()
    hostAudioCapture.cancel()
  })

  return { bindPluginFrame, framePermissions, onPluginFrameLoad }
}
