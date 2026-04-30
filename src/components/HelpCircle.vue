<script setup lang="ts">
defineProps<{
  label: string;
  inline?: boolean;
}>();
</script>

<template>
  <details class="hc" :class="{ 'hc--inline': inline }">
    <summary class="hc-btn" :aria-label="label">?</summary>
    <div class="hc-pop" role="note" aria-live="polite">
      <slot />
    </div>
  </details>
</template>

<style scoped>
.hc {
  position: relative;
  display: inline-block;
}
.hc--inline {
  vertical-align: middle;
}
.hc-btn {
  list-style: none;
  cursor: pointer;
  user-select: none;
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-weight: 700;
  font-size: 12px;
}
.hc-btn::-webkit-details-marker {
  display: none;
}
.hc[open] .hc-btn {
  color: var(--text-primary);
  border-color: var(--border-strong, var(--border-light));
}
.hc-pop {
  position: absolute;
  right: 0;
  top: calc(100% + 6px);
  z-index: 40;
  width: min(440px, 70vw);
  padding: 10px 12px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.55;
}
</style>

