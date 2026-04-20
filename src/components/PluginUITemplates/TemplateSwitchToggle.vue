<script setup lang="ts">
import { ref, watch } from "vue";

interface SwitchToggleSchema {
  checked: boolean;
  label?: string;
  hint?: string;
}

const props = defineProps<{
  schema: SwitchToggleSchema;
  busy?: boolean;
}>();

const emit = defineEmits<{
  submit: [{ enabled: boolean }];
}>();

const checked = ref(props.schema.checked);

watch(
  () => props.schema.checked,
  (v) => {
    checked.value = v;
  },
);

function onSubmit() {
  emit("submit", { enabled: checked.value });
}
</script>

<template>
  <div class="tpl-root">
    <label class="tpl-toggle">
      <input v-model="checked" type="checkbox" :disabled="busy" />
      <span>{{ schema.label || "异地心声" }}</span>
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
.tpl-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
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
