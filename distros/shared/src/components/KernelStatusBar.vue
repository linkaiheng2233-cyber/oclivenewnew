<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useKernelConnectionStore } from '@oclive/shared/stores/kernelConnectionStore'

const { t } = useI18n()
const kernelStore = useKernelConnectionStore()

const label = computed(() => t(kernelStore.display.labelKey))

const tooltip = computed(() => {
  const parts = [t(kernelStore.display.labelKey)]
  if (kernelStore.display.detailKey) {
    parts.push(t(kernelStore.display.detailKey))
  }
  if (kernelStore.status?.port) {
    parts.push(`:${kernelStore.status.port}`)
  }
  return parts.join(' · ')
})

async function onClick() {
  if (!kernelStore.display.clickable || kernelStore.disabled) {
    return
  }
  try {
    await kernelStore.reconnect()
  }
  catch {
    // lastError set in store
  }
}

onMounted(() => {
  void kernelStore.init()
})

onBeforeUnmount(() => {
  // Keep listeners for app lifetime; only tear down if this is the sole consumer.
})

defineExpose({ refresh: () => kernelStore.refresh() })
</script>

<template>
  <button
    type="button"
    class="kernel-status"
    :class="{
      'kernel-status--ok': kernelStore.display.ok,
      'kernel-status--warn': kernelStore.phase === 'ready' && !kernelStore.display.ok && !kernelStore.display.checking,
      'kernel-status--checking': kernelStore.display.checking,
      'kernel-status--clickable': kernelStore.display.clickable,
    }"
    :aria-label="tooltip"
    :title="tooltip"
    :disabled="kernelStore.disabled"
    @click="onClick"
  >
    <span class="kernel-status__dot" aria-hidden="true" />
    <span class="kernel-status__text">
      {{ label }}
      <template v-if="kernelStore.status?.port">
        :{{ kernelStore.status.port }}
      </template>
    </span>
  </button>
</template>

<style scoped>
.kernel-status {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.15rem 0.5rem;
  border: 1px solid var(--border-subtle, rgba(127, 127, 127, 0.35));
  border-radius: 999px;
  background: var(--bg-elevated, rgba(0, 0, 0, 0.04));
  color: inherit;
  font-size: 0.75rem;
  cursor: default;
}

.kernel-status--clickable {
  cursor: pointer;
}

.kernel-status--clickable:hover:not(:disabled) {
  border-color: var(--border-strong, rgba(127, 127, 127, 0.55));
}

.kernel-status:disabled {
  opacity: 0.85;
}

.kernel-status--ok .kernel-status__dot {
  background: #22c55e;
}

.kernel-status--warn .kernel-status__dot {
  background: #f59e0b;
}

.kernel-status--checking .kernel-status__dot {
  background: #94a3b8;
}

.kernel-status__dot {
  width: 0.45rem;
  height: 0.45rem;
  border-radius: 50%;
  background: #94a3b8;
}
</style>
