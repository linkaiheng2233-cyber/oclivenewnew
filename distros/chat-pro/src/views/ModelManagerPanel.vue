<script setup lang="ts">
import ModelManagerBody from '@oclive/shared/components/model/ModelManagerBody.vue'
import UiCloseButton from '@oclive/shared/components/ui/UiCloseButton.vue'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import MumuFeatureIcon from '../components/MumuFeatureIcon.vue'

const props = withDefaults(
  defineProps<{
    visible: boolean
    embedded?: boolean
    initialProvider?: 'local' | 'cloud'
  }>(),
  { embedded: false },
)

const emit = defineEmits<{
  close: []
  openSettings: []
  saved: [roleId: string]
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
defineExpose({ focus: () => dialogRef.value?.focus({ preventScroll: true }) })
useModalFocusRestore(computed(() => props.visible && !props.embedded), dialogRef, { trapFocus: true })
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
          <MumuFeatureIcon name="model" size="heading" />
          <h2 class="mm-heading">
            {{ t("modelManager.title") }}
          </h2>
          <UiCloseButton class="mm-close" :label="t('modelManager.close')" @click="emit('close')" />
        </header>
        <ModelManagerBody :initial-provider="initialProvider" @saved="emit('saved', $event)" @open-settings="emit('openSettings')" />
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
  width: min(760px, 100%);
  max-height: min(88dvh, 800px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
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
  gap: 16px;
  padding: 18px 22px;
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}
.mm-heading {
  flex: 1;
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
}
.mm-dialog :deep(.mm-root) {
  padding: 18px 22px 22px;
  scrollbar-gutter: stable;
  scroll-padding-bottom: 90px;
}
.mm-dialog :deep(.mm-footer) {
  position: sticky;
  bottom: 0;
  z-index: 1;
  padding-bottom: 12px;
  background: var(--bg-primary);
}
@media (max-width: 600px) {
  .mm-header { padding: 14px 16px; }
  .mm-dialog :deep(.mm-root) { padding: 14px 16px 18px; }
}
</style>

<style>
@import '@oclive/shared/styles/win98/panel-model.css';
</style>
