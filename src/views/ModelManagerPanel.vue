<script setup lang="ts">
import { ref, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import ModelManagerBody from '../components/model/ModelManagerBody.vue'
import { useModalFocusRestore } from '../composables/useModalFocusRestore'

const props = withDefaults(
  defineProps<{
    visible: boolean
    embedded?: boolean
  }>(),
  { embedded: false },
)

const emit = defineEmits<{
  close: []
  openSettings: []
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'visible'), dialogRef)
</script>

<template>
  <component :is="embedded ? 'div' : 'Teleport'" v-bind="embedded ? {} : { to: 'body' }">
    <div
      v-if="visible"
      :class="embedded ? 'mm-embedded' : 'mm-backdrop'"
      :role="embedded ? undefined : 'dialog'"
      :aria-modal="embedded ? undefined : 'true'"
      :aria-label="embedded ? undefined : t('modelManager.panelAria')"
      @click.self="!embedded && emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="dialogRef"
        :class="embedded ? 'mm-embedded-inner' : 'mm-dialog'"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <header v-if="!embedded" class="mm-header">
          <h2 class="mm-heading">
            {{ t("modelManager.title") }}
          </h2>
          <button type="button" class="mm-close" @click="emit('close')">
            {{ t("modelManager.close") }}
          </button>
        </header>
        <ModelManagerBody @open-settings="emit('openSettings')" />
      </div>
    </div>
  </component>
</template>

<style scoped>
.mm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10056;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.mm-dialog {
  width: min(640px, 100%);
  max-height: min(88vh, 720px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 14px 16px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.mm-embedded {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.mm-embedded-inner {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: visible;
  background: var(--tool-chrome-editor, var(--bg-primary));
}
.mm-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
  flex-shrink: 0;
}
.mm-heading {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
}
.mm-close {
  padding: 4px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 12px;
}
</style>
