import { nextTick, ref, watch, type Ref } from 'vue'

/**
 * After a dialog / panel closes (including Escape), restore focus to the element active before open.
 */
export function useReturnFocusOnClose(visibleRef: Ref<boolean>) {
  const focusReturn = ref<HTMLElement | null>(null)

  watch(visibleRef, (open) => {
    if (open) {
      const active = document.activeElement
      focusReturn.value = active instanceof HTMLElement ? active : null
      return
    }
    const el = focusReturn.value
    focusReturn.value = null
    void nextTick(() => el?.focus({ preventScroll: true }))
  })
}
