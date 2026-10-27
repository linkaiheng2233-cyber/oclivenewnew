import type { Ref } from 'vue'
import { nextTick, onBeforeUnmount, watch } from 'vue'

/**
 * When a modal opens: stash `document.activeElement`, then focus dialog root or primary control.
 * When it closes: restore focus to the opener.
 */
export function useModalFocusRestore(
  isOpen: Ref<boolean>,
  dialogRoot: Ref<HTMLElement | null>,
  options?: { primary?: Ref<HTMLElement | null | undefined>, trapFocus?: boolean },
): void {
  let opener: HTMLElement | null = null
  let generation = 0
  function restoreFocus(): void {
    const toRestore = opener
    opener = null
    void nextTick(() => {
      if (toRestore?.isConnected) {
        try {
          toRestore.focus({ preventScroll: true })
        }
        catch {
          /* opener may have left the active document */
        }
      }
    })
  }
  watch(isOpen, (open) => {
    const attempt = ++generation
    if (open) {
      const el = document.activeElement
      opener = el instanceof HTMLElement ? el : null
      void nextTick(() => {
        if (attempt === generation && isOpen.value) {
          const target = options?.primary?.value ?? dialogRoot.value
          target?.focus({ preventScroll: true })
        }
      })
    }
    else {
      restoreFocus()
    }
  }, { immediate: true })

  watch([isOpen, dialogRoot], ([open, root], _previous, cleanup) => {
    if (!open || !root || !options?.trapFocus)
      return
    const onKeydown = (event: KeyboardEvent) => {
      if (event.key !== 'Tab' || event.defaultPrevented)
        return
      const controls = [...root.querySelectorAll<HTMLElement>('button,input,select,textarea,a[href],[tabindex],summary')]
        .filter(element => !element.matches(':disabled,[tabindex="-1"]') && element.getClientRects().length > 0 && getComputedStyle(element).visibility !== 'hidden')
      const first = controls[0]
      const last = controls.at(-1)
      if (!first || !last) {
        event.preventDefault()
        root.focus({ preventScroll: true })
      }
      else if (event.shiftKey && (document.activeElement === first || document.activeElement === root)) {
        event.preventDefault()
        last.focus({ preventScroll: true })
      }
      else if (!event.shiftKey && (document.activeElement === last || document.activeElement === root)) {
        event.preventDefault()
        first.focus({ preventScroll: true })
      }
    }
    root.addEventListener('keydown', onKeydown)
    cleanup(() => root.removeEventListener('keydown', onKeydown))
  }, { flush: 'post' })

  onBeforeUnmount(() => {
    generation += 1
    restoreFocus()
  })
}
