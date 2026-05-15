import { nextTick, watch, type Ref } from "vue";

/**
 * When a modal opens: stash `document.activeElement`, then focus dialog root or primary control.
 * When it closes: restore focus to the opener.
 */
export function useModalFocusRestore(
  isOpen: Ref<boolean>,
  dialogRoot: Ref<HTMLElement | null>,
  options?: { primary?: Ref<HTMLElement | null | undefined> },
): void {
  let opener: HTMLElement | null = null;
  watch(isOpen, (open) => {
    if (open) {
      const el = document.activeElement;
      opener = el instanceof HTMLElement ? el : null;
      void nextTick(() => {
        const target = options?.primary?.value ?? dialogRoot.value;
        target?.focus({ preventScroll: true });
      });
    } else {
      const toRestore = opener;
      opener = null;
      void nextTick(() => {
        if (toRestore?.isConnected) {
          try {
            toRestore.focus({ preventScroll: true });
          } catch {
            /* ignore */
          }
        }
      });
    }
  });
}
