<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginStore } from "../stores/pluginStore";

const { t } = useI18n();

const props = defineProps<{
  pluginId: string;
  slotKey: string;
}>();

const pluginStore = usePluginStore();

const appearanceChoices = computed(() => {
  const entry = pluginStore.catalog.find((c) => c.id === props.pluginId);
  const raw = entry?.uiSlotVariants?.filter((x) => x.slot === props.slotKey) ?? [];
  if (raw.length <= 1) {
    return [] as { appearanceId: string; label: string }[];
  }
  return raw.map((x) => ({
    appearanceId: x.appearanceId,
    label: (x.label?.trim() || x.appearanceId || t("pluginManager.pmSlot.defaultVariant")).trim(),
  }));
});

const selectedAppearance = computed({
  get(): string {
    return (
      pluginStore.pluginState.slot_appearance?.[props.pluginId]?.[
        props.slotKey
      ] ?? ""
    );
  },
  set(v: string) {
    pluginStore.setSlotAppearance(props.pluginId, props.slotKey, v);
  },
});
</script>

<template>
  <div class="pm-slot-tools">
    <div v-if="appearanceChoices.length > 1" class="pm-appearance">
      <label class="pm-appearance-label">{{ t("pluginManager.pmSlot.appearance") }}</label>
      <select v-model="selectedAppearance" class="pm-appearance-select">
        <option value="">{{ t("pluginManager.pmSlot.defaultVariant") }}</option>
        <option
          v-for="opt in appearanceChoices"
          :key="`${opt.appearanceId}`"
          :value="opt.appearanceId"
        >
          {{ opt.label }}
        </option>
      </select>
    </div>
    <label class="chk pm-slot-hide">
      <input
        type="checkbox"
        :checked="pluginStore.isSlotContributionDisabled(slotKey, pluginId)"
        @change="
          pluginStore.setSlotContributionDisabled(
            slotKey,
            pluginId,
            ($event.target as HTMLInputElement).checked,
          )
        "
      />
      {{ t("pluginManager.pmSlot.hideSlot") }}
    </label>
  </div>
</template>

<style scoped>
.pm-slot-tools {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin-left: auto;
}
.pm-appearance {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}
.pm-appearance-label {
  color: var(--text-secondary);
  user-select: none;
}
.pm-appearance-select {
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  max-width: 160px;
}
.pm-slot-hide {
  font-size: 12px;
  user-select: none;
}
</style>
