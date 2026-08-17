<script setup lang="ts">
import { useEasterEggSkin } from '@oclive/shared/composables/useEasterEggSkin'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { computed } from 'vue'

const { win98Enabled } = useEasterEggSkin()
const roleStore = useRoleStore()

const titleText = computed(() => {
  const role = roleStore.roles.find(r => r.id === roleStore.currentRoleId)
  const roleName = role?.name?.trim()
  return roleName ? `OCLive Chat Pro - ${roleName}` : 'OCLive Chat Pro'
})

function isTauriWebview(): boolean {
  return typeof window !== 'undefined' && Object.hasOwn(window, '__TAURI_INTERNALS__')
}

async function withCurrentWindow(
  action: (win: import('@tauri-apps/api/webviewWindow').WebviewWindow) => Promise<void> | void,
): Promise<void> {
  if (!isTauriWebview())
    return
  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    await action(getCurrentWebviewWindow())
  }
  catch {
    /* web build or permission denied */
  }
}

function onMinimize(): void {
  void withCurrentWindow(win => win.minimize())
}

function onToggleMaximize(): void {
  void withCurrentWindow(win => win.toggleMaximize())
}

function onClose(): void {
  void withCurrentWindow(win => win.close())
}

function onCaptionDblClick(event: MouseEvent): void {
  if ((event.target as HTMLElement).closest('.win98-titlebar__btn'))
    return
  onToggleMaximize()
}
</script>

<template>
  <header
    v-if="win98Enabled"
    class="win98-titlebar"
    data-tauri-drag-region
    @dblclick="onCaptionDblClick"
  >
    <img
      class="win98-titlebar__icon"
      src="/oclive-icon.png"
      alt=""
      width="16"
      height="16"
      aria-hidden="true"
      draggable="false"
    >
    <span class="win98-titlebar__title" data-tauri-drag-region>{{ titleText }}</span>
    <div class="win98-titlebar__controls">
      <button
        type="button"
        class="win98-titlebar__btn"
        aria-label="Minimize"
        @click.stop="onMinimize"
      >
        <span aria-hidden="true">─</span>
      </button>
      <button
        type="button"
        class="win98-titlebar__btn"
        aria-label="Maximize"
        @click.stop="onToggleMaximize"
      >
        <span aria-hidden="true">□</span>
      </button>
      <button
        type="button"
        class="win98-titlebar__btn win98-titlebar__btn--close"
        aria-label="Close"
        @click.stop="onClose"
      >
        <span aria-hidden="true">✕</span>
      </button>
    </div>
  </header>
</template>

<style>
@import '@oclive/shared/styles/win98/titlebar.css';
</style>
