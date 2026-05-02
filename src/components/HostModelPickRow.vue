<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useHostModelPick } from "../composables/useHostModelPick";

const props = withDefaults(
  defineProps<{ disabled?: boolean; selectId?: string; showGear?: boolean }>(),
  { disabled: false, selectId: "oclive-host-model", showGear: true },
);

const emit = defineEmits<{ openSettings: [] }>();

const { t } = useI18n();
const pick = useHostModelPick();
const selectModel = pick.selectModel;
const ollamaNames = pick.ollamaNames;
const useCustomModel = pick.useCustomModel;
const cloudSelectOptions = pick.cloudSelectOptions;
const modelId = pick.modelId;
const modelCustomInputRef = pick.customInputEl;
const CUSTOM_SENTINEL = pick.CUSTOM_SENTINEL;

onMounted(() => {
  void pick.bootstrap();
});
</script>

<template>
  <div class="hmpr">
    <label class="hmpr-label" :for="props.selectId">{{ t("chatComposer.modelLabel") }}</label>
    <div class="hmpr-mid">
      <select
        :id="props.selectId"
        class="hmpr-select"
        :value="selectModel"
        :disabled="props.disabled"
        @change="pick.onSelectModel"
      >
        <optgroup :label="String(t('chatComposer.localGroup'))">
          <option v-if="!ollamaNames.length" disabled value="__none__">
            {{ t("chatComposer.offlineLocal") }}
          </option>
          <option v-for="n in ollamaNames" :key="'loc-' + n" :value="n">{{ n }}</option>
        </optgroup>
        <optgroup :label="String(t('chatComposer.cloudGroup'))">
          <option v-for="n in cloudSelectOptions" :key="'cld-' + n" :value="n">{{ n }}</option>
        </optgroup>
        <optgroup :label="String(t('chatComposer.customGroup'))">
          <option :value="CUSTOM_SENTINEL">{{ t("chatComposer.customOption") }}</option>
        </optgroup>
      </select>
      <div v-if="useCustomModel" class="hmpr-custom-wrap">
        <input
          ref="modelCustomInputRef"
          v-model="modelId"
          type="text"
          class="hmpr-custom-input"
          spellcheck="false"
          autocomplete="off"
          :placeholder="String(t('chatComposer.customPlaceholder'))"
          :disabled="props.disabled"
          @input="pick.onCustomModelInput"
          @blur="pick.onCustomModelBlur"
        />
      </div>
    </div>
    <button
      v-if="props.showGear"
      type="button"
      class="hmpr-gear"
      :title="String(t('chatComposer.openSettings'))"
      :disabled="props.disabled"
      @click="emit('openSettings')"
    >
      {{ t("chatComposer.gear") }}
    </button>
  </div>
</template>

<style scoped>
.hmpr {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 0;
}
.hmpr-label {
  flex: 0 0 auto;
  margin-top: 7px;
  font-size: 12px;
  font-weight: 650;
  color: var(--text-secondary);
  white-space: nowrap;
}
.hmpr-mid {
  flex: 1 1 auto;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.hmpr-select {
  width: 100%;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  box-sizing: border-box;
  cursor: pointer;
}
.hmpr-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent);
}
.hmpr-custom-wrap {
  width: 100%;
}
.hmpr-custom-input {
  width: 100%;
  box-sizing: border-box;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px dashed color-mix(in srgb, var(--accent) 35%, var(--border-light));
  background: var(--bg-primary);
  color: var(--text-primary);
}
.hmpr-custom-input:focus {
  outline: none;
  border-style: solid;
  border-color: var(--accent);
}
.hmpr-gear {
  flex: 0 0 auto;
  margin-top: 4px;
  padding: 6px 10px;
  font-size: 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
}
.hmpr-gear:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: color-mix(in srgb, var(--accent) 35%, var(--border-light));
}
.hmpr-gear:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
