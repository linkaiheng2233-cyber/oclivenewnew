import { useOverlayWindow } from '@oclive/shared/composables/useOverlayWindow'

export interface UseModelManagerWindowOptions {
  closeMoreMenu: () => void
}

export function useModelManagerWindow(opts: UseModelManagerWindowOptions) {
  const { open: modelManagerOpen, toggle, close } = useOverlayWindow({
    closeMoreMenu: opts.closeMoreMenu,
  })

  return {
    modelManagerOpen,
    openModelManager: toggle,
    closeModelManager: close,
  }
}
