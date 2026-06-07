<script setup lang="ts">
defineProps<{
  label?: string
  description?: string
}>()
</script>

<template>
  <div class="ui-list-row">
    <div class="ui-list-row__main">
      <div v-if="label || $slots.label" class="ui-list-row__label">
        <slot name="label">
          {{ label }}
        </slot>
      </div>
      <p v-if="description || $slots.description" class="ui-list-row__desc">
        <slot name="description">
          {{ description }}
        </slot>
      </p>
    </div>
    <div v-if="$slots.control" class="ui-list-row__control">
      <slot name="control" />
    </div>
  </div>
</template>

<style scoped>
.ui-list-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--tool-space-3, 12px);
  min-height: var(--tool-row-h, 32px);
  padding: var(--tool-space-2, 8px) 0;
}

.ui-list-row + .ui-list-row {
  border-top: 1px solid var(--tool-border, var(--border-light));
}

.ui-list-row__main {
  flex: 1;
  min-width: 0;
}

.ui-list-row__label {
  font-size: var(--tool-fs-md, 13px);
  font-weight: 500;
  color: var(--tool-text, var(--text-primary));
  line-height: var(--tool-line, 1.5);
}

.ui-list-row__desc {
  margin: var(--tool-space-1, 4px) 0 0;
  font-size: var(--tool-fs-sm, 12px);
  color: var(--tool-text-muted, var(--text-secondary));
  line-height: var(--tool-line, 1.5);
}

.ui-list-row__control {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  min-height: var(--tool-row-h, 32px);
}
</style>
