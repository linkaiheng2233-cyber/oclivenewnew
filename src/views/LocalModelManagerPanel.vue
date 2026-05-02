<script setup lang="ts">
import { provide } from "vue";
import { useI18n } from "vue-i18n";
import BuiltinLlamaModelManager from "../components/BuiltinLlamaModelManager.vue";
import TrustConsentModal from "../components/TrustConsentModal.vue";
import { cloudLlmTrustModalKey, useCloudLlmTrustModal } from "../composables/useCloudLlmTrustModal";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
const cloudTrust = useCloudLlmTrustModal();
provide(cloudLlmTrustModalKey, cloudTrust);

function onTrustModalVisible(v: boolean): void {
  cloudTrust.visible.value = v;
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="lmm-stack">
      <div class="lmm-dim" role="presentation" @click.self="emit('close')">
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
      <!-- 嵌套在同一 fixed 层内，避免与外层 dim 两个 body 级 fixed 在 WebView2 下争用命中 -->
      <TrustConsentModal
        :model-value="cloudTrust.visible"
        :title="cloudTrust.modalTitle"
        :subtitle="cloudTrust.modalSubtitle"
        :trust-summary-title="cloudTrust.trustSummaryTitle"
        :trust-summary="cloudTrust.trustSummaryBody"
        :hint="cloudTrust.modalHint"
        :capabilities="cloudTrust.capabilities"
        :confirm-label="cloudTrust.confirmLabel"
        variant="trust"
        require-explicit-dismiss
        teleport-disabled
        @update:model-value="onTrustModalVisible"
      />
    </div>
  </Teleport>
</template>

<style scoped>
.lmm-stack {
  position: fixed;
  inset: 0;
  z-index: 10062;
  isolation: isolate;
  pointer-events: auto;
}
.lmm-dim {
  position: absolute;
  inset: 0;
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
