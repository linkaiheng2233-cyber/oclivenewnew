<script setup lang="ts">
import { computed } from "vue";
import { usePluginStore } from "../stores/pluginStore";

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
    label: (x.label?.trim() || x.appearanceId || "默认").trim(),
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
      <label class="pm-appearance-label">外观</label>
      <select v-model="selectedAppearance" class="pm-appearance-select">
        <option value="">默认</option>
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
      隐藏本槽
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
