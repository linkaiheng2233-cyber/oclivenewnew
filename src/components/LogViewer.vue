<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  lines: string[]
}>()

const emit = defineEmits<{
  clear: []
  export: []
}>()

const { t } = useI18n()
const q = ref('')

const filtered = computed(() => {
  const needle = q.value.trim().toLowerCase()
  if (!needle)
    return props.lines
  return props.lines.filter(l => l.toLowerCase().includes(needle))
})

function lineClass(line: string): string {
  const u = line.toUpperCase()
  if (u.includes('[STDERR]') || u.includes('ERROR'))
    return 'is-err'
  if (u.includes('WARN'))
    return 'is-warn'
  return ''
}
</script>

<template>
  <div class="pm-dbg-log">
    <div class="pm-dbg-log-head">
      <input v-model="q" class="pm-dbg-filter" type="search" :placeholder="t('devTools.logViewer.filterPh')">
      <button type="button" class="pm-dbg-btn secondary" @click="emit('clear')">
        {{ t("devTools.logViewer.clear") }}
      </button>
      <button type="button" class="pm-dbg-btn secondary" @click="emit('export')">
        {{ t("devTools.logViewer.export") }}
      </button>
    </div>
    <div class="pm-dbg-log-body" role="log" aria-live="polite">
      <div v-for="(line, i) in filtered" :key="`${i}-${line.slice(0, 24)}`" class="pm-dbg-line" :class="lineClass(line)">
        {{ line }}
      </div>
      <p v-if="!filtered.length" class="pm-dbg-empty">
        {{ t("devTools.logViewer.empty") }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.pm-dbg-log {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
  flex: 1;
}
.pm-dbg-log-head {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
}
.pm-dbg-filter {
  flex: 1;
  min-width: 120px;
  padding: 5px 8px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  font-size: 12px;
}
.pm-dbg-btn {
  padding: 5px 10px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
  cursor: pointer;
}
.pm-dbg-btn.secondary {
  background: transparent;
}
.pm-dbg-log-body {
  flex: 1;
  min-height: 140px;
  max-height: 220px;
  overflow: auto;
  padding: 6px 8px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 11px;
  line-height: 1.4;
}
.pm-dbg-line {
  white-space: pre-wrap;
  word-break: break-word;
}
.pm-dbg-line.is-err {
  color: var(--error);
}
.pm-dbg-line.is-warn {
  color: color-mix(in srgb, #fbbf24 85%, var(--text-primary));
}
.pm-dbg-empty {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
