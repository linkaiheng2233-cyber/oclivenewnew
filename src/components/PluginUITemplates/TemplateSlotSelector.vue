<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

interface OptionItem {
  value: string;
  label: string;
}

interface SlotSelectorSchema {
  current: string;
  options: OptionItem[];
  directoryOptions: OptionItem[];
  directoryId?: string;
}

const props = defineProps<{
  schema: SlotSelectorSchema;
  busy?: boolean;
}>();

const emit = defineEmits<{
  submit: [{ backend: string | null; directoryId?: string | null }];
}>();

const { t } = useI18n();
const draftBackend = ref(props.schema.current);
const draftDirectoryId = ref(props.schema.directoryId ?? "");

watch(
  () => props.schema.current,
  (v) => {
    draftBackend.value = v;
  },
);

watch(
  () => props.schema.directoryId,
  (v) => {
    draftDirectoryId.value = v ?? "";
  },
);

function onSubmit() {
  const usesDirectory = draftBackend.value === "directory";
  emit("submit", {
    backend: draftBackend.value === "__pack_default__" ? null : draftBackend.value,
    directoryId: usesDirectory ? (draftDirectoryId.value.trim() || null) : undefined,
  });
}
</script>

<template>
  <div class="tpl-root">
    <label class="tpl-label">
      {{ t("pluginTerms.field.backend") }}
      <select v-model="draftBackend" class="tpl-select" :disabled="busy">
        <option v-for="opt in schema.options" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </label>

    <label class="tpl-label">
      {{ t("pluginTerms.field.directory_plugin") }}
      <input
        v-model="draftDirectoryId"
        class="tpl-input"
        type="text"
        :placeholder="t('pluginManager.template.directoryIdPh')"
        :disabled="busy || draftBackend !== 'directory'"
      />
      <select
        v-model="draftDirectoryId"
        class="tpl-select"
        :disabled="busy || draftBackend !== 'directory'"
      >
        <option value="">{{ t("pluginManager.template.directoryManualPick") }}</option>
        <option v-for="opt in schema.directoryOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </label>

    <p v-if="schema.hint" class="tpl-hint">{{ schema.hint }}</p>
    <p class="tpl-hint">{{ t("pluginTerms.hint.directory_id_empty") }}</p>
    <button type="button" class="tpl-btn" :disabled="busy" @click="onSubmit">{{ t("pluginTerms.action.apply") }}</button>
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
.tpl-select,
.tpl-input {
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
