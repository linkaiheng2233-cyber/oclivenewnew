<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { fetchKernelStartupWarnings } from '../composables/useStartupHealthWarnings'

const { t } = useI18n()
const warnings = ref<string[]>([])
const dismissed = ref(false)

const visible = computed(() => !dismissed.value && warnings.value.length > 0)

onMounted(async () => {
  warnings.value = await fetchKernelStartupWarnings()
})

function dismiss() {
  dismissed.value = true
}
</script>

<template>
  <div
    v-if="visible"
    class="startup-warnings"
    role="status"
    aria-live="polite"
  >
    <p class="startup-warnings__title">
      {{ t('kernel.startupWarnings.title') }}
    </p>
    <ul class="startup-warnings__list">
      <li v-for="(msg, idx) in warnings" :key="idx">
        {{ msg }}
      </li>
    </ul>
    <button type="button" class="startup-warnings__dismiss" @click="dismiss">
      {{ t('kernel.startupWarnings.dismiss') }}
    </button>
  </div>
</template>

<style scoped>
.startup-warnings {
  margin: 0.5rem 1rem 0;
  padding: 0.65rem 0.85rem;
  border-radius: 8px;
  border: 1px solid rgba(245, 158, 11, 0.45);
  background: rgba(245, 158, 11, 0.12);
  font-size: 0.82rem;
}

.startup-warnings__title {
  margin: 0 0 0.35rem;
  font-weight: 600;
}

.startup-warnings__list {
  margin: 0;
  padding-left: 1.1rem;
}

.startup-warnings__dismiss {
  margin-top: 0.45rem;
  border: none;
  background: transparent;
  color: inherit;
  text-decoration: underline;
  cursor: pointer;
  font-size: 0.78rem;
  opacity: 0.85;
}
</style>
