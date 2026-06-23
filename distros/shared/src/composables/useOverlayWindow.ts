import { ref } from 'vue'

export interface UseOverlayWindowOptions {
  closeMoreMenu: () => void
  /** Called before toggling open (e.g. close sibling panels). */
  onOpen?: () => void
}

/** Shared toggle overlay for plugin manager, model manager, etc. */
export function useOverlayWindow(opts: UseOverlayWindowOptions) {
  const open = ref(false)

  function toggle(forceOpen = false): void {
    opts.onOpen?.()
    open.value = forceOpen ? true : !open.value
    opts.closeMoreMenu()
  }

  function close(): void {
    open.value = false
  }

  return { open, toggle, close }
}
