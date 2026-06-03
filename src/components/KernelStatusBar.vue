<script setup lang="ts">
import type { DesktopKernelMode, KernelConnectionStatus } from '../api/kernel'
import { listen } from '@tauri-apps/api/event'
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getKernelConnectionStatus, reconnectKernel } from '../api/kernel'

const { t } = useI18n()

const status = ref<KernelConnectionStatus | null>(null)
const busy = ref(false)

function labelForMode(mode: DesktopKernelMode): string {
  switch (mode) {
    case 'attached':
      return t('kernel.status.attached')
    case 'spawned':
      return t('kernel.status.spawned')
    case 'reconnecting':
      return t('kernel.status.reconnecting')
    default:
      return t('kernel.status.offline')
  }
}

async function refresh() {
  try {
    status.value = await getKernelConnectionStatus()
  }
  catch {
    status.value = null
  }
}

async function onReconnect() {
  if (busy.value) {
    return
  }
  busy.value = true
  try {
    status.value = await reconnectKernel()
  }
  catch {
    await refresh()
  }
  finally {
    busy.value = false
  }
}

let unlistenLost: (() => void) | undefined
let unlistenOk: (() => void) | undefined

onMounted(() => {
  void refresh()
  void listen<KernelConnectionStatus>('kernel:upstream_lost', (e) => {
    status.value = e.payload
  }).then(u => { unlistenLost = u })
  void listen<KernelConnectionStatus>('kernel:reconnected', (e) => {
    status.value = e.payload
  }).then(u => { unlistenOk = u })
})

onBeforeUnmount(() => {
  unlistenLost?.()
  unlistenOk?.()
})

defineExpose({ refresh })
</script>

<template>
  <button
    type="button"
    class="kernel-status"
    :class="{
      'kernel-status--ok': status?.healthy,
      'kernel-status--warn': status && !status.healthy,
    }"
    :aria-label="t('kernel.status.aria')"
    :disabled="busy"
    @click="onReconnect"
  >
    <span class="kernel-status__dot" aria-hidden="true" />
    <span class="kernel-status__text">
      {{ status ? labelForMode(status.mode) : t('kernel.status.offline') }}
      <template v-if="status?.port">
        :{{ status.port }}
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
  cursor: pointer;
}

.kernel-status--ok .kernel-status__dot {
  background: #22c55e;
}

.kernel-status--warn .kernel-status__dot {
  background: #f59e0b;
}

.kernel-status__dot {
  width: 0.45rem;
  height: 0.45rem;
  border-radius: 50%;
  background: #94a3b8;
}
</style>
