<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

interface OptionItem {
  value: string;
  label: string;
}

export interface SlotRegistryTemplateSchema {
  slotKey: string;
  backend: string;
  packBackend: string;
  sessionOverridden: boolean;
  options: OptionItem[];
  directoryOptions?: OptionItem[];
  directoryId?: string;
}

const props = defineProps<{
  schema: SlotRegistryTemplateSchema;
  busy?: boolean;
}>();

const emit = defineEmits<{
  submit: [
    {
      slotKey: string;
      backend: string;
      applyMode: "session" | "pack";
      directoryId?: string | null;
    },
  ];
}>();

const { t } = useI18n();
const draftBackend = ref(props.schema.backend);
const draftDirectoryId = ref(props.schema.directoryId ?? "");

watch(
  () => props.schema.backend,
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

function submit(applyMode: "session" | "pack") {
  const usesDirectory = draftBackend.value === "directory";
  emit("submit", {
    slotKey: props.schema.slotKey,
    backend: draftBackend.value,
    applyMode,
    directoryId: usesDirectory ? draftDirectoryId.value.trim() || null : undefined,
  });
}
</script>

<template>
  <div class="tpl-root">
    <p v-if="schema.sessionOverridden" class="tpl-override">
      {{ t("pluginWorkbench.graph.sessionOverride") }}
    </p>
    <p class="tpl-meta">
      {{ t("pluginManager.v2.slotKey") }}: <code>{{ schema.slotKey }}</code>
    </p>
    <p class="tpl-meta">
      {{ t("pluginManager.v2.packBackend") }}: <code>{{ schema.packBackend }}</code>
    </p>
    <label class="tpl-label">
      {{ t("pluginTerms.field.backend") }}
      <select v-model="draftBackend" class="tpl-select" :disabled="busy">
        <option v-for="opt in schema.options" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </label>
    <template v-if="draftBackend === 'directory'">
      <label class="tpl-label">
        {{ t("pluginTerms.field.directory_plugin") }}
        <select
          v-model="draftDirectoryId"
          class="tpl-select"
          :disabled="busy"
        >
          <option value="">{{ t("pluginManager.template.directoryManualPick") }}</option>
          <option
            v-for="opt in schema.directoryOptions ?? []"
            :key="opt.value"
            :value="opt.value"
          >
            {{ opt.label }}
          </option>
        </select>
      </label>
    </template>
    <div class="tpl-actions">
      <button type="button" class="tpl-btn tpl-btn--primary" :disabled="busy" @click="submit('session')">
        {{ t("pluginWorkbench.graph.applySessionOnly") }}
      </button>
      <button type="button" class="tpl-btn" :disabled="busy" @click="submit('pack')">
        {{ t("pluginWorkbench.graph.applyPackDefault") }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.tpl-root {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.tpl-override {
  margin: 0;
  font-size: 12px;
  color: var(--arch-stroke, #7aad8f);
}
.tpl-meta {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
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
.tpl-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.tpl-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.tpl-btn--primary {
  font-weight: 600;
}
</style>
