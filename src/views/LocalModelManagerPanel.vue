<script setup lang="ts">
import BuiltinLlamaModelManager from "../components/BuiltinLlamaModelManager.vue";
import { useI18n } from "vue-i18n";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="lmm-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="String(t('localModelManagerPanel.aria'))"
      @click.self="emit('close')"
    >
      <div class="lmm-dialog" @click.stop>
        <header class="lmm-head">
          <div class="lmm-head-text">
            <h2 class="lmm-title">{{ t("localModelManagerPanel.title") }}</h2>
            <p class="lmm-hint">{{ t("localModelManagerPanel.hint") }}</p>
          </div>
          <button type="button" class="lmm-close" @click="emit('close')">
            {{ t("localModelManagerPanel.close") }}
          </button>
        </header>
        <div class="lmm-body">
          <BuiltinLlamaModelManager @request-close="emit('close')" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.lmm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10062;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.lmm-dialog {
  width: min(800px, 100%);
  max-height: min(88vh, 860px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 12px 14px 14px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.lmm-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
  flex-shrink: 0;
}
.lmm-head-text {
  min-width: 0;
}
.lmm-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}
.lmm-hint {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.lmm-close {
  border-radius: 8px;
  padding: 6px 12px;
  font-size: 12px;
  border: 1px solid var(--border-light);
  background: transparent;
  color: inherit;
  cursor: pointer;
}
.lmm-close:hover {
  background: var(--bg-hover, rgba(255, 255, 255, 0.06));
}
.lmm-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
</style>
