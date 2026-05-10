<script setup lang="ts">
import { computed, ref, unref, watch } from "vue";
import { useI18n } from "vue-i18n";
import HostModelPickRow from "../HostModelPickRow.vue";
import { useAppToast } from "../../composables/useAppToast";
import { useHostModelPick } from "../../composables/useHostModelPick";

const props = defineProps<{
  /** 当前侧栏选中本页时为 true，用于按需刷新列表 */
  active: boolean;
}>();

const { t } = useI18n();
const { showToast } = useAppToast();
const pick = useHostModelPick();
const booting = ref(false);
const loadError = ref<string | null>(null);

const showEmptyHint = computed(() => {
  if (booting.value || loadError.value) return false;
  const ollama = unref(pick.ollamaNames);
  const cloudOpts = unref(pick.cloudSelectOptions);
  return !ollama.length && !cloudOpts.length;
});

async function runBootstrap(): Promise<void> {
  loadError.value = null;
  booting.value = true;
  try {
    await pick.bootstrap();
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
    showToast("error", loadError.value);
  } finally {
    booting.value = false;
  }
}

watch(
  () => props.active,
  (active) => {
    if (active) void runBootstrap();
  },
  { immediate: true },
);
</script>

<template>
  <div class="mss">
    <p v-if="booting" class="mss-status">{{ t("settings.modelSelector.loading") }}</p>
    <p v-else-if="loadError" class="mss-err">
      {{ loadError }}
      <button type="button" class="mss-retry" @click="runBootstrap">
        {{ t("settings.modelSelector.retry") }}
      </button>
    </p>
    <template v-else>
      <HostModelPickRow select-id="oclive-settings-default-model" :show-gear="false" />
      <p v-if="showEmptyHint" class="mss-empty">{{ t("settings.modelSelector.emptyHint") }}</p>
      <p class="mss-hint">{{ t("settings.modelSelector.syncHint") }}</p>
    </template>
  </div>
</template>

<style scoped>
.mss {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 520px;
}
.mss-status,
.mss-err,
.mss-empty {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.mss-err {
  color: var(--text-accent, #b45309);
}
.mss-retry {
  margin-left: 8px;
  padding: 2px 8px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.mss-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
}
</style>
