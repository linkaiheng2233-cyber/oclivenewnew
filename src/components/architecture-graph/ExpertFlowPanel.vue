<script setup lang="ts">
import type { ExpertRoute, ExpertRoutingDoc } from '../../api/role/expert'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { selectActiveExpertRoute } from '../../lib/expertRoutingGraph'

const props = defineProps<{
  doc: ExpertRoutingDoc | null | undefined
  sceneId: string
  userMessage?: string
}>()

const { t } = useI18n()
const expanded = ref(false)

const activeRoute = computed(() =>
  selectActiveExpertRoute(
    props.doc,
    props.sceneId,
    props.userMessage ?? '',
  ),
)

const steps = computed(() => activeRoute.value?.steps ?? [])

function stepLabel(action: string): string {
  const known: Record<string, string> = {
    'slot.personality.adjust': t('expertConfig.actions.personalityAdjust'),
    'slot.prompt_enhance.apply': t('expertConfig.actions.promptEnhance'),
    'slot.memory.inject': t('expertConfig.actions.memoryInject'),
    'slot.lora.apply': t('expertConfig.actions.loraApply'),
    'slot.expert.fallback': t('expertConfig.actions.expertFallback'),
  }
  return known[action] ?? action
}
</script>

<template>
  <div class="efp" :class="{ 'efp--open': expanded, 'efp--empty': !activeRoute }">
    <button
      type="button"
      class="efp-toggle"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      {{ t('expertConfig.flowPanel.title') }}
      <span v-if="activeRoute" class="efp-badge">{{ activeRoute.id ?? 'route' }}</span>
      <span v-else class="efp-badge efp-badge--muted">{{ t('expertConfig.flowPanel.inactive') }}</span>
    </button>
    <p v-show="expanded && !activeRoute" class="efp-empty">
      {{ t('expertConfig.flowPanel.noActive') }}
    </p>
    <ol v-show="expanded && activeRoute" class="efp-steps">
      <li
        v-for="(step, idx) in steps"
        :key="`${step.action}-${idx}`"
        class="efp-step"
      >
        <span class="efp-step-no">{{ idx + 1 }}</span>
        <div class="efp-step-body">
          <code class="efp-action">{{ stepLabel(step.action) }}</code>
          <span v-if="step.depends_on?.length" class="efp-deps">
            ← {{ step.depends_on.join(', ') }}
          </span>
        </div>
        <span v-if="idx < steps.length - 1" class="efp-arrow" aria-hidden="true">↓</span>
      </li>
    </ol>
  </div>
</template>

<style scoped>
.efp {
  position: absolute;
  right: 12px;
  bottom: 12px;
  z-index: 4;
  max-width: 280px;
  background: var(--bg-elevated, #1e1e24);
  border: 1px solid var(--border-light, #444);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  font-size: 11px;
}
.efp-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  text-align: left;
}
.efp-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(232, 163, 23, 0.2);
  color: #e8a317;
}
.efp-badge--muted {
  background: rgba(128, 128, 128, 0.2);
  color: var(--text-secondary);
}
.efp-empty {
  margin: 0;
  padding: 0 10px 10px;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.efp--empty .efp-toggle {
  opacity: 0.92;
}
.efp-steps {
  margin: 0;
  padding: 0 10px 10px 10px;
  list-style: none;
}
.efp-step {
  position: relative;
  padding: 6px 0 6px 28px;
  border-left: 2px solid #e8a317;
  margin-left: 8px;
}
.efp-step-no {
  position: absolute;
  left: -10px;
  top: 6px;
  width: 18px;
  height: 18px;
  line-height: 18px;
  text-align: center;
  border-radius: 50%;
  background: #e8a317;
  color: #1a1a1a;
  font-size: 10px;
  font-weight: 700;
}
.efp-action {
  display: block;
  font-size: 10px;
  word-break: break-all;
}
.efp-deps {
  display: block;
  color: var(--text-secondary);
  font-size: 9px;
  margin-top: 2px;
}
.efp-arrow {
  display: block;
  text-align: center;
  color: var(--text-secondary);
  margin-top: 2px;
}
</style>
