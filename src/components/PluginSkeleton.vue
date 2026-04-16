<script setup lang="ts">
withDefaults(
  defineProps<{
    /** toolbar: 矮条；block: 较大区域 */
    variant?: "toolbar" | "block";
  }>(),
  { variant: "toolbar" },
);
</script>

<template>
  <div class="psk" :class="`psk--${variant}`" aria-hidden="true">
    <div class="psk-line psk-a" />
    <div v-if="variant === 'block'" class="psk-line psk-b" />
    <div v-if="variant === 'block'" class="psk-line psk-c" />
  </div>
</template>

<style scoped>
.psk {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
  border-radius: var(--radius-btn);
  overflow: hidden;
}
.psk--toolbar {
  min-height: 32px;
  max-width: min(420px, 100%);
  padding: 4px 0;
}
.psk--block {
  min-height: 120px;
  padding: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.psk-line {
  height: 10px;
  border-radius: 4px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--border-light) 70%, var(--bg-primary)) 0%,
    color-mix(in srgb, var(--border-light) 40%, var(--bg-primary)) 50%,
    color-mix(in srgb, var(--border-light) 70%, var(--bg-primary)) 100%
  );
  background-size: 200% 100%;
  animation: psk-shimmer 1.1s ease-in-out infinite;
}
.psk--toolbar .psk-line {
  height: 8px;
}
.psk-a {
  width: 72%;
}
.psk-b {
  width: 100%;
}
.psk-c {
  width: 55%;
}
@keyframes psk-shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
