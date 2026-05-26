import { ref } from 'vue'

export interface UseModelManagerWindowOptions {
  closeMoreMenu: () => void
}

export function useModelManagerWindow(opts: UseModelManagerWindowOptions) {
  const modelManagerOpen = ref(false)

  function openModelManager(forceOpen = false): void {
    if (forceOpen) {
      modelManagerOpen.value = true
    }
    else {
      modelManagerOpen.value = !modelManagerOpen.value
    }
    opts.closeMoreMenu()
  }

  function closeModelManager(): void {
    modelManagerOpen.value = false
  }

  return {
    modelManagerOpen,
    openModelManager,
    closeModelManager,
  }
}
