<script setup lang="ts">
import type { ExpertRoutingDoc } from '@oclive/shared/api/role/expert'
import { computed, ref, toRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '@oclive/shared/composables/useModalFocusRestore'
import { computeLineDiff, formatJson } from '@oclive/shared/lib/jsonLineDiff'

const props = defineProps<{
  open: boolean
  beforeDoc: ExpertRoutingDoc | null | undefined
  afterDoc: ExpertRoutingDoc | null | undefined
  busy?: boolean
}>()

const emit = defineEmits<{
  close: []
  confirm: []
}>()

const { t } = useI18n()
const dialogRef = ref<HTMLElement | null>(null)
useModalFocusRestore(toRef(props, 'open'), dialogRef)

const beforeText = computed(() => formatJson(props.beforeDoc ?? { routes: [] }))
const afterText = computed(() => formatJson(props.afterDoc ?? { routes: [] }))
const diffLines = computed(() => computeLineDiff(beforeText.value, afterText.value))

watch(
  () => props.open,
  (v) => {
    if (!v) {
      return
    }
  },
)

function onBackdrop(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('bpm-backdrop')) {
    emit('close')
  }
}
</script>

<template>
  <div
    v-if="open"
    class="bpm-backdrop"
    role="dialog"
    aria-modal="true"
    :aria-label="t('expertConfig.blueprintPreview.title')"
    @click="onBackdrop"
  >
    <div ref="dialogRef" class="bpm-panel" tabindex="-1">
      <h3 class="bpm-title">
        {{ t("expertConfig.blueprintPreview.title") }}
      </h3>
      <p class="bpm-hint">
        {{ t("expertConfig.blueprintPreview.hint") }}
      </p>
      <div class="bpm-columns">
        <section class="bpm-col">
          <h4>{{ t("expertConfig.blueprintPreview.before") }}</h4>
          <pre class="bpm-pre bpm-pre--before" aria-readonly="true">{{ beforeText }}</pre>
        </section>
        <section class="bpm-col">
          <h4>{{ t("expertConfig.blueprintPreview.after") }}</h4>
          <pre class="bpm-pre bpm-pre--after" aria-readonly="true">{{ afterText }}</pre>
        </section>
      </div>
      <div class="bpm-diff-wrap">
        <h4>{{ t("expertConfig.blueprintPreview.diff") }}</h4>
        <pre class="bpm-diff" aria-readonly="true"><code
          v-for="(line, idx) in diffLines"
          :key="idx"
          class="bpm-diff-line"
          :class="`bpm-diff-line--${line.kind}`"
        ><template v-if="line.kind === 'remove' || line.kind === 'change'">- {{ line.before }}
</template><template v-if="line.kind === 'add' || line.kind === 'change'">+ {{ line.after }}
</template><template v-if="line.kind === 'same'">  {{ line.before }}
</template></code></pre>
      </div>
      <div class="bpm-actions">
        <button type="button" class="bpm-btn" :disabled="busy" @click="emit('close')">
          {{ t("expertConfig.cancel") }}
        </button>
        <button
          type="button"
          class="bpm-btn bpm-btn--primary"
          :disabled="busy"
          @click="emit('confirm')"
        >
          {{ t("expertConfig.confirmSave") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bpm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1250;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  padding: 12px;
}
.bpm-panel {
  width: min(960px, 96vw);
  max-height: 90vh;
  overflow: auto;
  padding: 16px 18px;
  border-radius: 10px;
  background: var(--bg-elevated, #1e1e24);
  border: 1px solid var(--border-light, #444);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
.bpm-title {
  margin: 0 0 6px;
  font-size: 15px;
}
.bpm-hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--text-secondary);
}
.bpm-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
@media (max-width: 720px) {
  .bpm-columns {
    grid-template-columns: 1fr;
  }
}
.bpm-col h4,
.bpm-diff-wrap h4 {
  margin: 0 0 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}
.bpm-pre {
  margin: 0;
  padding: 10px;
  border-radius: 6px;
  font-size: 11px;
  line-height: 1.45;
  overflow: auto;
  max-height: 200px;
  background: #121218;
  border: 1px solid #333;
}
.bpm-diff-wrap {
  margin-top: 12px;
}
.bpm-diff {
  margin: 0;
  padding: 10px;
  border-radius: 6px;
  font-size: 11px;
  line-height: 1.45;
  max-height: 180px;
  overflow: auto;
  background: #0e0e12;
  border: 1px solid #333;
}
.bpm-diff-line {
  display: block;
  white-space: pre-wrap;
  word-break: break-all;
}
.bpm-diff-line--add {
  background: rgba(80, 180, 100, 0.15);
  color: #8fd99a;
}
.bpm-diff-line--remove {
  background: rgba(200, 80, 80, 0.15);
  color: #e8a0a0;
}
.bpm-diff-line--change {
  background: rgba(232, 163, 23, 0.12);
  color: #e8c878;
}
.bpm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
.bpm-btn {
  font-size: 12px;
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid var(--border-light, #444);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
}
.bpm-btn--primary {
  background: #e8a317;
  border-color: transparent;
  color: #1a1a1a;
  font-weight: 600;
}
.bpm-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
