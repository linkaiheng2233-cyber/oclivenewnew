<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { SLOT_TYPE_ORDER } from "../../lib/slotRegistry";

const props = defineProps<{
  open: boolean;
  busy?: boolean;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [slotType: string, label: string];
}>();

const { t } = useI18n();
const slotType = ref("memory");
const label = ref("");

watch(
  () => props.open,
  (v) => {
    if (v) {
      slotType.value = "memory";
      label.value = "";
    }
  },
);

function onSubmit() {
  emit("confirm", slotType.value, label.value);
}

function onBackdrop(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains("aasd-backdrop")) {
    emit("close");
  }
}
</script>

<template>
  <div
    v-if="open"
    class="aasd-backdrop"
    role="dialog"
    aria-modal="true"
    :aria-label="t('pluginWorkbench.graph.addSlotWizardTitle')"
    @click="onBackdrop"
  >
    <form class="aasd-panel" @submit.prevent="onSubmit">
      <h3 class="aasd-title">{{ t("pluginWorkbench.graph.addSlotWizardTitle") }}</h3>
      <label class="aasd-field">
        <span>{{ t("pluginWorkbench.graph.addSlotType") }}</span>
        <select v-model="slotType" class="aasd-input" :disabled="busy">
          <option v-for="st in SLOT_TYPE_ORDER" :key="st" :value="st">
            {{
              t(
                `pluginWorkbench.graph.${st === "complex_emotion" ? "complexEmotion" : st}`,
              )
            }}
          </option>
        </select>
      </label>
      <label class="aasd-field">
        <span>{{ t("pluginWorkbench.graph.addSlotLabel") }}</span>
        <input
          v-model="label"
          class="aasd-input"
          type="text"
          :placeholder="t('pluginWorkbench.graph.addSlotLabelPlaceholder')"
          :disabled="busy"
        />
      </label>
      <p class="aasd-hint">{{ t("pluginWorkbench.graph.addSlotWizardHint") }}</p>
      <div class="aasd-actions">
        <button type="button" class="aasd-btn" :disabled="busy" @click="emit('close')">
          {{ t("pluginWorkbench.graph.addSlotCancel") }}
        </button>
        <button type="submit" class="aasd-btn aasd-btn--primary" :disabled="busy">
          {{ t("pluginWorkbench.graph.addSlotConfirm") }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.aasd-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, #000 55%, transparent);
}
.aasd-panel {
  width: min(360px, 92vw);
  padding: 16px 18px;
  border-radius: 10px;
  background: var(--surface-elevated, #1e1e1e);
  border: 1px solid var(--border-subtle, #333);
  box-shadow: 0 12px 40px rgb(0 0 0 / 0.45);
}
.aasd-title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
}
.aasd-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
  font-size: 12px;
  color: var(--text-secondary);
}
.aasd-input {
  font-size: 13px;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, #333);
  background: var(--surface-base, #141414);
  color: var(--text-primary);
}
.aasd-hint {
  margin: 0 0 12px;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.aasd-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.aasd-btn {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-subtle, #333);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
}
.aasd-btn--primary {
  border-color: var(--accent, #6af);
  background: color-mix(in srgb, var(--accent, #6af) 18%, transparent);
}
.aasd-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
