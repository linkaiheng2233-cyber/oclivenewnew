import type { Ref } from 'vue'
import { nextTick, onBeforeUnmount, watch } from 'vue'

const modalRoots: HTMLElement[] = []

/**
 * When a modal opens: stash `document.activeElement`, then focus dialog root or primary control.
 * When it closes: restore focus to the opener.
 */
export function useModalFocusRestore(
  isOpen: Ref<boolean>,
  dialogRoot: Ref<HTMLElement | null>,
  options?: { primary?: Ref<HTMLElement | null | undefined>, opener?: Ref<HTMLElement | null | undefined>, trapFocus?: boolean, onEscape?: () => void },
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
      // An async operation may disable and blur its trigger before opening.
      const el = options?.opener?.value ?? document.activeElement
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
    modalRoots.push(root)
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
    // Native WebView activation can place keyboard focus on the document,
    // outside the dialog subtree. The foreground recovery dialog still owns
    // Escape and must consume it before lower-window global handlers run.
    const onEscape = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.isComposing || !isOpen.value
        || modalRoots.at(-1) !== root || !options.onEscape) {
        return
      }
      event.preventDefault()
      event.stopImmediatePropagation()
      options.onEscape()
    }
    if (options.onEscape)
      window.addEventListener('keydown', onEscape, true)
    cleanup(() => {
      const index = modalRoots.lastIndexOf(root)
      if (index >= 0)
        modalRoots.splice(index, 1)
      root.removeEventListener('keydown', onKeydown)
      window.removeEventListener('keydown', onEscape, true)
    })
  }, { flush: 'post' })

  onBeforeUnmount(() => {
    generation += 1
    restoreFocus()
  })
}
