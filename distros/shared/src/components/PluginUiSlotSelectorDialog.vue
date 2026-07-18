<script setup lang="ts">
import type { PluginSlotSelectorState } from '@oclive/shared/composables/usePluginSlotEnable'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  state: PluginSlotSelectorState
  busy?: boolean
}>()

const emit = defineEmits<{
  close: []
  confirm: []
  toggleSlot: [slotId: string]
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
const visible = computed(() => props.state.open)
useModalFocusRestore(visible, dialogRef)
</script>

<template>
  <Teleport to="body">
    <div
      v-if="state.open"
      class="pss-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('simplePluginManager.slotSelector.title')"
      @click.self="emit('close')"
      @keydown.escape.stop="emit('close')"
    >
      <div
        ref="dialogRef"
        class="pss-dialog"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <h3 class="pss-title">
          {{ t("simplePluginManager.slotSelector.title") }}
        </h3>
        <p class="pss-lead">
          {{ t("simplePluginManager.slotSelector.lead", { id: state.pluginTitle }) }}
        </p>
        <ul class="pss-list" role="listbox" :aria-label="t('simplePluginManager.slotSelector.listAria')">
          <li v-for="slot in state.slots" :key="slot.id" class="pss-item">
            <label class="pss-label">
              <input
                type="checkbox"
                :checked="state.selected.includes(slot.id)"
                :disabled="busy"
                @change="emit('toggleSlot', slot.id)"
              >
              <span>{{ slot.label }}</span>
              <span class="pss-id">{{ slot.id }}</span>
            </label>
          </li>
        </ul>
        <footer class="pss-actions">
          <button type="button" class="pss-btn" :disabled="busy" @click="emit('close')">
            {{ t("simplePluginManager.slotSelector.cancel") }}
          </button>
          <button
            type="button"
            class="pss-btn primary"
            :disabled="busy || state.selected.length === 0"
            @click="emit('confirm')"
          >
            {{ t("simplePluginManager.slotSelector.confirm") }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pss-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10070;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 50%, transparent);
}
.pss-dialog {
  width: min(420px, 100%);
  padding: 16px 18px;
  border-radius: var(--radius-app, 10px);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pss-title {
  margin: 0 0 8px;
  font-size: 1.05rem;
}
.pss-lead {
  margin: 0 0 12px;
  font-size: 0.875rem;
  color: var(--text-muted, #64748b);
}
.pss-list {
  list-style: none;
  margin: 0 0 16px;
  padding: 0;
  max-height: 240px;
  overflow: auto;
  border: 1px solid var(--border-light);
  border-radius: 6px;
}
.pss-item {
  border-bottom: 1px solid var(--border-light);
}
.pss-item:last-child {
  border-bottom: none;
}
.pss-label {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
}
.pss-id {
  margin-left: auto;
  font-size: 0.75rem;
  color: var(--text-muted, #64748b);
}
.pss-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.pss-btn {
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  cursor: pointer;
}
.pss-btn.primary {
  background: var(--accent, #3b82f6);
  border-color: transparent;
  color: #fff;
}
.pss-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

<style>
@import '@oclive/shared/styles/win98/dialogs-shared.css';
</style>
