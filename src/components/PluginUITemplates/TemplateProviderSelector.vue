<script setup lang="ts">
import { ref, watch } from "vue";

interface OptionItem {
  value: string;
  label: string;
}

interface ProviderSelectorSchema {
  current: string;
  options: OptionItem[];
  hint?: string;
}

const props = defineProps<{
  schema: ProviderSelectorSchema;
  busy?: boolean;
}>();

const emit = defineEmits<{
  submit: [{ backend: string | null }];
}>();

const draft = ref(props.schema.current);

watch(
  () => props.schema.current,
  (v) => {
    draft.value = v;
  },
);

function onSubmit() {
  emit("submit", {
    backend: draft.value === "__pack_default__" ? null : draft.value,
  });
}
</script>

<template>
  <div class="tpl-root">
    <label class="tpl-label">
      运行方式
      <select v-model="draft" class="tpl-select" :disabled="busy">
        <option v-for="opt in schema.options" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </label>
    <p v-if="schema.hint" class="tpl-hint">{{ schema.hint }}</p>
    <button type="button" class="tpl-btn" :disabled="busy" @click="onSubmit">应用改动</button>
  </div>
</template>

<style scoped>
.tpl-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.tpl-label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
.tpl-select {
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.tpl-hint {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.tpl-btn {
  align-self: flex-start;
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
</style>
