import { nextTick, ref, watch, type Ref } from 'vue'

/**
 * 对话框 / 面板关闭（含 Escape）后，将焦点还原到打开前的活动元素。
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
