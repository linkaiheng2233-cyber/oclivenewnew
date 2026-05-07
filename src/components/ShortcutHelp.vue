<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import PluginSlotEmbed from "./PluginSlotEmbed.vue";
import {
  shortcutHelpCtrlShiftADescription,
  shortcutHelpCtrlShiftFDescription,
} from "../lib/pluginManagerEntryCopy";
import { formatChordModShift } from "../lib/shortcutDisplay";
import { SLOT_LAUNCHER_PALETTE } from "../stores/pluginStore";
import { useUiStore } from "../stores/uiStore";

withDefaults(
  defineProps<{
    modelValue: boolean;
    /** 与插件 bootstrap 同步 */
    bootstrapEpoch?: number;
  }>(),
  { bootstrapEpoch: 0 },
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const uiStore = useUiStore();
const { t } = useI18n();

const rows = computed(() => {
  const pluginF = shortcutHelpCtrlShiftFDescription(
    uiStore.experimentalPluginManagerV2,
  );
  const pluginA = shortcutHelpCtrlShiftADescription();
  return [
    { keys: formatChordModShift("S"), desc: String(t("shortcutHelp.rows.ctrlShiftS")) },
    { keys: formatChordModShift("F"), desc: pluginF },
    { keys: formatChordModShift("A"), desc: pluginA },
    { keys: String(t("shortcutHelp.rows.ctrlHoldKey")), desc: String(t("shortcutHelp.rows.ctrlHoldDesc")) },
  ];
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="sh-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="String(t('shortcutHelp.dialogLabel'))"
      @click.self="emit('update:modelValue', false)"
    >
      <div class="sh-dialog" @click.stop>
        <header class="sh-head">
          <h2 class="sh-title">{{ t("shortcutHelp.title") }}</h2>
          <button
            type="button"
            class="sh-close"
            :aria-label="String(t('common.close'))"
            @click="emit('update:modelValue', false)"
          >
            ×
          </button>
        </header>
        <table class="sh-table">
          <tbody>
            <tr v-for="(r, i) in rows" :key="`${r.keys}-${i}`">
              <td class="sh-keys">{{ r.keys }}</td>
              <td class="sh-desc">{{ r.desc }}</td>
            </tr>
          </tbody>
        </table>
        <p class="sh-foot">{{ t("shortcutHelp.footer") }}</p>
        <section class="sh-slot" :aria-label="String(t('shortcutHelp.launcherSlot.aria'))">
          <h3 class="sh-slot-h">{{ t("shortcutHelp.launcherSlot.title") }}</h3>
          <PluginSlotEmbed
            :slot-name="SLOT_LAUNCHER_PALETTE"
            :aria-label="String(t('shortcutHelp.launcherSlot.embedAria'))"
            :bootstrap-epoch="bootstrapEpoch"
          />
        </section>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.sh-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10060;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.sh-dialog {
  width: min(420px, 100%);
  max-height: min(80vh, 520px);
  overflow: auto;
  padding: 16px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.sh-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.sh-title {
  margin: 0;
  font-size: 17px;
}
.sh-close {
  border: none;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.sh-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.sh-table td {
  padding: 8px 6px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 60%, transparent);
  vertical-align: top;
}
.sh-keys {
  white-space: nowrap;
  font-weight: 600;
  color: var(--text-primary);
  width: 46%;
}
.sh-desc {
  color: var(--text-secondary);
}
.sh-foot {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.sh-slot {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid color-mix(in srgb, var(--border-light) 60%, transparent);
}
.sh-slot-h {
  margin: 0 0 8px;
  font-size: 13px;
}
</style>
