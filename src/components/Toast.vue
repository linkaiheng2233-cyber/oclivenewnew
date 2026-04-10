<script setup lang="ts">
defineProps<{
  show: boolean;
  type: "success" | "error" | "warning" | "info";
  message: string;
}>();
</script>

<template>
  <transition name="toast">
    <div v-if="show" class="toast" :class="type">{{ message }}</div>
  </transition>
</template>

<style scoped>
.toast {
  position: fixed;
  right: 12px;
  bottom: 12px;
  max-width: min(360px, calc(100vw - 24px));
  color: var(--text-primary);
  padding: 10px 14px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md), var(--frame-inset-highlight);
  backdrop-filter: blur(10px) saturate(108%);
  -webkit-backdrop-filter: blur(10px) saturate(108%);
  z-index: 12000;
  font-size: 13px;
}
.success {
  background: color-mix(in srgb, var(--success) 16%, var(--bg-elevated));
  border-color: color-mix(in srgb, var(--success) 55%, var(--border-light));
  color: var(--text-primary);
}
.error {
  background: color-mix(in srgb, var(--error) 18%, var(--bg-elevated));
  border-color: var(--error);
  color: var(--text-primary);
}
.warning {
  background: color-mix(in srgb, var(--warning) 14%, var(--bg-elevated));
  border-color: var(--warning);
  color: var(--text-primary);
}
.info {
  background: color-mix(in srgb, var(--info) 14%, var(--bg-elevated));
  border-color: color-mix(in srgb, var(--info) 45%, var(--border-light));
  color: var(--text-primary);
}
.toast-enter-active,
.toast-leave-active {
  transition: opacity 260ms ease, transform 260ms ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(12px);
}
</style>
