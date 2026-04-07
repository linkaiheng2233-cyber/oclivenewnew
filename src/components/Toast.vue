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
  box-shadow: var(--shadow-md);
  z-index: 12000;
  font-size: 13px;
}
.success {
  background: rgba(124, 184, 124, 0.25);
  border-color: var(--success);
  color: #e8f5e8;
}
.error {
  background: rgba(201, 122, 122, 0.3);
  border-color: var(--error);
}
.warning {
  background: rgba(212, 165, 116, 0.2);
  border-color: var(--warning);
  color: var(--text-accent);
}
.info {
  background: rgba(125, 148, 178, 0.25);
  border-color: var(--info);
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
