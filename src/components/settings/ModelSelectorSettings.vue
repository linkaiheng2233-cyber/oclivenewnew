<script setup lang="ts">
import { watch } from "vue";
import { useI18n } from "vue-i18n";
import HostModelPickRow from "../HostModelPickRow.vue";
import { useHostModelPick } from "../../composables/useHostModelPick";

const props = defineProps<{
  /** 当前侧栏选中本页时为 true，用于按需刷新列表 */
  active: boolean;
}>();

const { t } = useI18n();
const pick = useHostModelPick();

watch(
  () => props.active,
  (active) => {
    if (active) void pick.bootstrap();
  },
  { immediate: true },
);
</script>

<template>
  <div class="mss">
    <HostModelPickRow select-id="oclive-settings-default-model" :show-gear="false" />
    <p class="mss-hint">{{ t("settings.modelSelector.syncHint") }}</p>
  </div>
</template>

<style scoped>
.mss {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 520px;
}
.mss-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
}
</style>
