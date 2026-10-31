<script setup lang="ts">
import { reportAppFailure } from '@oclive/shared/utils/appErrorReporting'
import { computed, nextTick, onErrorCaptured, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ErrorNotice from './ErrorNotice.vue'
import UiButton from './ui/UiButton.vue'

withDefaults(defineProps<{ root?: boolean }>(), { root: false })
const { t } = useI18n()
const failure = ref<unknown>(null)
const failed = ref(false)
const attempt = ref(0)
const region = ref<HTMLElement | null>(null)
const failureDetails = computed(() => failure.value instanceof Error ? failure.value.stack ?? '' : '')
onErrorCaptured((error, _instance, info) => {
  failure.value = error
  failed.value = true
  reportAppFailure(error, 'render-boundary', info)
  return false
})
watch(failed, async (value) => {
  if (value) {
    await nextTick()
    region.value?.focus()
  }
})
function retry(): void {
  attempt.value += 1
  failure.value = null
  failed.value = false
}
</script>

<template>
  <slot v-if="!failed" :key="attempt" />
  <section v-else ref="region" class="render-recovery" :class="{ 'render-recovery--root': root }" role="region" :aria-label="t('support.regionFailed')" tabindex="-1">
    <ErrorNotice :error="failure" :details="failureDetails" :summary="t('support.regionFailed')" />
    <p>{{ t('support.renderRecoveryHint') }}</p>
    <UiButton variant="primary" @click="retry">
      {{ t('support.retryRegion') }}
    </UiButton>
  </section>
</template>

<style scoped>
.render-recovery { flex:1;min-height:0;display:flex;flex-direction:column;align-items:flex-start;justify-content:center;gap:16px;padding:24px;overflow:auto; }
.render-recovery--root { min-height:100dvh;max-width:1000px;margin:auto;box-sizing:border-box; }
.render-recovery p { margin:0;color:var(--text-secondary,#64748b);line-height:1.6; }
.render-recovery:focus { outline:none; }
</style>
