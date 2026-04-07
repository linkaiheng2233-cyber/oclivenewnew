<script setup lang="ts">
defineProps<{
  visible: boolean;
  fromLabel: string;
  toLabel: string;
}>();

const emit = defineEmits<{
  dismiss: [];
}>();
</script>

<template>
  <div v-if="visible" class="autonomous-scene-notice" role="status">
    <span class="autonomous-scene-notice__text"
      >系统：虚拟时间变化后，角色场景已从「{{ fromLabel }}」切换为「{{ toLabel }}」（叙事视角未自动改变）。</span
    >
    <button
      type="button"
      class="autonomous-scene-notice__close"
      aria-label="关闭"
      @click="emit('dismiss')"
    >
      ×
    </button>
  </div>
</template>

<style scoped>
.autonomous-scene-notice {
  flex-shrink: 0;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 10px;
  margin: 0 8px 10px;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
  background: var(--bg-elevated);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
}
.autonomous-scene-notice__text {
  flex: 1;
  min-width: 0;
}
.autonomous-scene-notice__close {
  flex-shrink: 0;
  line-height: 1;
  padding: 0 4px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 16px;
}
.autonomous-scene-notice__close:hover {
  color: var(--text-primary);
}
</style>
