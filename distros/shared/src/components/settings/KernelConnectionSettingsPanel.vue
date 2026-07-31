<script setup lang="ts">
import type { KernelDiagnostics } from '@oclive/shared/api'
import { getKernelDiagnostics } from '@oclive/shared/api'
import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useKernelConnectionStore } from '@oclive/shared/stores/kernelConnectionStore'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import UiButton from '../ui/UiButton.vue'

const props = defineProps<{
  active: boolean
}>()

const { t } = useI18n()
const { showToast } = useAppToast()
const kernelStore = useKernelConnectionStore()

const loading = ref(false)
const kernelDiag = ref<KernelDiagnostics | null>(null)
const statusLabel = computed(() => t(kernelStore.display.labelKey))
const statusDetail = computed(() =>
  kernelStore.display.detailKey ? t(kernelStore.display.detailKey) : null,
)

const statusTone = computed(() => {
  if (kernelStore.display.checking || kernelStore.busy) {
    return 'checking'
  }
  if (kernelStore.display.ok) {
    return 'ok'
  }
  return 'warn'
})

onMounted(() => {
  void kernelStore.init()
})

watch(
  () => props.active,
  (on) => {
    if (on) {
      void kernelStore.refresh()
      if (!kernelDiag.value) {
        void refreshDiagnostics()
      }
    }
  },
  { immediate: true },
)

async function refreshDiagnostics(): Promise<void> {
  loading.value = true
  try {
    kernelDiag.value = await getKernelDiagnostics()
    kernelStore.applyStatus(kernelDiag.value.status)
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}

async function onReconnect(): Promise<void> {
  if (!kernelStore.canReconnect) {
    if (kernelStore.alreadyConnected) {
      showToast('info', t('kernel.diagnostics.alreadyConnected'))
    }
    return
  }
  loading.value = true
  try {
    const status = await kernelStore.reconnect()
    if (status) {
      kernelDiag.value = kernelDiag.value
        ? { ...kernelDiag.value, status }
        : await getKernelDiagnostics()
    }
    showToast('info', t('kernel.status.reconnect'))
  }
  catch (err) {
    showToast('error', err instanceof Error ? err.message : String(err))
  }
  finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="kernel-panel">
    <div
      class="kernel-panel__status"
      :class="`kernel-panel__status--${statusTone}`"
      role="status"
      :aria-label="t('kernel.status.aria')"
    >
      <span class="kernel-panel__dot" aria-hidden="true" />
      <div class="kernel-panel__status-text">
        <p class="kernel-panel__label">
          {{ statusLabel }}
          <span v-if="kernelStore.status?.port" class="kernel-panel__port">
            :{{ kernelStore.status.port }}
          </span>
        </p>
        <p v-if="statusDetail" class="kernel-panel__detail">
          {{ statusDetail }}
        </p>
        <p v-else-if="kernelStore.alreadyConnected" class="kernel-panel__detail">
          {{ t('kernel.diagnostics.alreadyConnectedHint') }}
        </p>
        <p v-else-if="kernelStore.lastError" class="kernel-panel__detail kernel-panel__detail--error">
          {{ kernelStore.lastError }}
        </p>
      </div>
    </div>

    <div class="kernel-panel__actions">
      <UiButton
        size="sm"
        variant="secondary"
        :disabled="loading"
        @click="refreshDiagnostics"
      >
        {{ loading ? t('settings.envCheckRunning') : t('kernel.diagnostics.refresh') }}
      </UiButton>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="loading || !kernelStore.canReconnect"
        :title="kernelStore.alreadyConnected ? t('kernel.diagnostics.alreadyConnected') : undefined"
        @click="onReconnect"
      >
        {{ t('kernel.diagnostics.reconnect') }}
      </UiButton>
    </div>

    <div v-if="kernelDiag" class="kernel-panel__facts">
      <p class="kernel-panel__fact">
        <span class="kernel-panel__fact-key">{{ t('kernel.diagnostics.mode') }}</span>
        <span>{{ kernelDiag.status.mode }}</span>
        <span
          class="kernel-panel__health"
          :class="kernelDiag.status.healthy ? 'kernel-panel__health--ok' : 'kernel-panel__health--bad'"
        >
          {{
            kernelDiag.status.healthy
              ? t('kernel.diagnostics.healthyYes')
              : t('kernel.diagnostics.healthyNo')
          }}
        </span>
      </p>
      <p v-if="kernelDiag.status.binaryPath" class="kernel-panel__fact">
        <span class="kernel-panel__fact-key">{{ t('kernel.diagnostics.binary') }}</span>
        <code class="kernel-panel__code">{{ kernelDiag.status.binaryPath }}</code>
      </p>
      <p v-if="kernelDiag.status.kernelTier" class="kernel-panel__fact">
        <span class="kernel-panel__fact-key">{{ t('kernel.diagnostics.tier') }}</span>
        <span>{{ kernelDiag.status.kernelTier }}</span>
      </p>
      <p class="kernel-panel__fact">
        <span class="kernel-panel__fact-key">{{ t('kernel.diagnostics.sharedRuntime') }}</span>
        <code class="kernel-panel__code">{{ kernelDiag.sharedRuntimePath }}</code>
      </p>
    </div>

    <details v-if="kernelDiag?.healthJson" class="kernel-panel__raw">
      <summary class="kernel-panel__raw-summary">
        {{ t('kernel.diagnostics.healthJson') }}
      </summary>
      <pre class="kernel-panel__pre">{{ JSON.stringify(kernelDiag.healthJson, null, 2) }}</pre>
    </details>
  </div>
</template>

<style scoped>
.kernel-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.kernel-panel__status {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated, rgba(127, 127, 127, 0.06));
}

.kernel-panel__status--ok {
  border-color: color-mix(in srgb, #22c55e 35%, var(--border-light));
}

.kernel-panel__status--warn {
  border-color: color-mix(in srgb, #f59e0b 40%, var(--border-light));
}

.kernel-panel__status--checking {
  border-color: var(--border-light);
}

.kernel-panel__dot {
  width: 10px;
  height: 10px;
  margin-top: 4px;
  border-radius: 50%;
  flex-shrink: 0;
  background: #94a3b8;
}

.kernel-panel__status--ok .kernel-panel__dot {
  background: #22c55e;
}

.kernel-panel__status--warn .kernel-panel__dot {
  background: #f59e0b;
}

.kernel-panel__label {
  margin: 0;
  font-weight: 600;
  font-size: 14px;
  line-height: 1.35;
}

.kernel-panel__port {
  font-weight: 500;
  opacity: 0.85;
}

.kernel-panel__detail {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}

.kernel-panel__detail--error {
  color: var(--oc-danger, #dc2626);
}

.kernel-panel__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.kernel-panel__facts {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
  line-height: 1.45;
}

.kernel-panel__fact {
  margin: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px;
}

.kernel-panel__fact-key {
  font-weight: 600;
  color: var(--text-primary);
}

.kernel-panel__health--ok {
  color: #16a34a;
}

.kernel-panel__health--bad {
  color: #dc2626;
}

.kernel-panel__code {
  font-size: 11px;
  word-break: break-all;
}

.kernel-panel__raw {
  font-size: 12px;
}

.kernel-panel__raw-summary {
  cursor: pointer;
  color: var(--text-secondary);
  user-select: none;
}

.kernel-panel__pre {
  margin: 8px 0 0;
  padding: 10px;
  max-height: 200px;
  overflow: auto;
  border-radius: 8px;
  background: var(--bg-secondary);
  font-size: 11px;
  line-height: 1.4;
}
</style>
