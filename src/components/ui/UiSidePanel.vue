<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  visible: boolean
  title: string
  ariaLabel?: string
  flushBody?: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const panelRef = ref<HTMLElement | null>(null)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void nextTick(() => {
        panelRef.value?.focus({ preventScroll: true })
      })
    }
  },
)
</script>

<template>
  <aside
    v-show="visible"
    ref="panelRef"
    class="ui-side-panel"
    role="complementary"
    :aria-label="ariaLabel ?? title"
    tabindex="-1"
    @keydown.escape.stop="emit('close')"
  >
    <header class="ui-side-panel__head">
      <h2 class="ui-side-panel__title">
        {{ title }}
      </h2>
      <button
        type="button"
        class="ui-side-panel__close"
        :aria-label="t('settings.closeAria')"
        @click="emit('close')"
      >
        ×
      </button>
    </header>
    <div v-if="$slots.tabs" class="ui-side-panel__tabs">
      <slot name="tabs" />
    </div>
    <div
      class="ui-side-panel__scroll"
      :class="{ 'ui-side-panel__scroll--flush': flushBody }"
    >
      <slot />
    </div>
  </aside>
</template>

<style scoped>
.ui-side-panel {
  flex: 0 0 var(--tool-sidepanel-w, 400px);
  width: var(--tool-sidepanel-w, 400px);
  min-width: 0;
  align-self: stretch;
  height: auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--tool-chrome-sidebar, var(--tool-elevated, var(--bg-primary)));
  border-left: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  box-shadow: none;
}

.ui-side-panel__tabs {
  flex-shrink: 0;
}

.ui-side-panel__head {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--tool-space-2, 8px);
  min-height: var(--tool-topbar-h, 36px);
  height: var(--tool-topbar-h, 36px);
  padding: 0 var(--tool-space-4, 16px);
  border-bottom: 1px solid var(--tool-divider, var(--tool-border, var(--border-light)));
  background: var(--tool-chrome-editor, var(--tool-elevated, var(--bg-primary)));
}

.ui-side-panel__title {
  margin: 0;
  font-size: var(--tool-fs-lg, 15px);
  font-weight: 600;
  color: var(--tool-text, var(--text-primary));
}

.ui-side-panel__close {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--tool-radius, 4px);
  background: transparent;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  color: var(--tool-text-muted, var(--text-secondary));
  transition: var(--tool-transition, var(--control-transition));
}

.ui-side-panel__close:hover {
  background: color-mix(in srgb, var(--tool-border, var(--border-light)) 45%, transparent);
  color: var(--tool-text, var(--text-primary));
}

.ui-side-panel__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: var(--tool-space-4, 16px);
  scrollbar-color: var(--scrollbar-panel-thumb, var(--scrollbar-chat-thumb)) var(--scrollbar-panel-track, var(--scrollbar-chat-track));
  scrollbar-width: thin;
}

.ui-side-panel__scroll::-webkit-scrollbar {
  width: 8px;
}

.ui-side-panel__scroll::-webkit-scrollbar-track {
  background: var(--scrollbar-panel-track, var(--scrollbar-chat-track));
}

.ui-side-panel__scroll::-webkit-scrollbar-thumb {
  background: var(--scrollbar-panel-thumb, var(--scrollbar-chat-thumb));
  border-radius: 4px;
}

.ui-side-panel__scroll::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-panel-thumb-hover, var(--scrollbar-chat-thumb-hover));
}

.ui-side-panel__scroll--flush {
  padding: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
</style>
