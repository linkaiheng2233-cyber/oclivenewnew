<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  visible: boolean
  /** 已解析的展示名，如「客厅」 */
  pendingSceneLabel: string
}>()

const emit = defineEmits<{
  confirm: [together: boolean]
  dismiss: []
}>()

const { t } = useI18n()

const primaryTogetherRef = ref<HTMLButtonElement | null>(null)

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      void nextTick(() => primaryTogetherRef.value?.focus())
    }
  },
)
</script>

<template>
  <div
    v-if="visible"
    class="scene-mode-dialog-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="top-bar-scene-mode-title"
  >
    <div class="scene-mode-dialog">
      <p id="top-bar-scene-mode-title" class="scene-mode-title">
        {{ t("common.sceneMode.title", { label: pendingSceneLabel }) }}
      </p>
      <p class="scene-mode-desc">
        {{ t("common.sceneMode.desc") }}
      </p>
      <div class="scene-mode-actions">
        <button type="button" class="post-reply-btn" @click="emit('confirm', false)">
          {{ t("common.sceneMode.solo") }}
        </button>
        <button
          ref="primaryTogetherRef"
          type="button"
          class="post-reply-btn post-reply-btn--primary"
          @click="emit('confirm', true)"
        >
          {{ t("common.sceneMode.together") }}
        </button>
        <button type="button" class="post-reply-btn" @click="emit('dismiss')">
          {{ t("common.cancel") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scene-mode-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--dialog-backdrop, color-mix(in srgb, var(--bg-page) 55%, transparent));
  padding: 16px;
}
.scene-mode-dialog {
  max-width: 400px;
  width: 100%;
  padding: 18px 20px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--dialog-panel-bg, var(--bg-primary));
  box-shadow: var(--shadow-md), var(--frame-inset-highlight);
}
.scene-mode-title {
  margin: 0 0 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}
.scene-mode-desc {
  margin: 0 0 14px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.scene-mode-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.post-reply-btn {
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.post-reply-btn--primary {
  background: linear-gradient(135deg, var(--btn-grad-a), var(--btn-grad-b));
  color: var(--text-accent);
  border-color: var(--accent);
}
</style>
