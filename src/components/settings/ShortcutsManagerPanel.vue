<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import PluginSlotEmbed from "../PluginSlotEmbed.vue";
import HotkeySettingsSection from "../HotkeySettingsSection.vue";
import {
  shortcutHelpCtrlShiftADescription,
  shortcutHelpCtrlShiftFDescription,
} from "../../lib/pluginManagerEntryCopy";
import { formatChordModShift } from "../../lib/shortcutDisplay";
import { SLOT_LAUNCHER_PALETTE } from "../../stores/pluginStore";
import { useUiStore } from "../../stores/uiStore";

defineProps<{
  bootstrapEpoch: number;
}>();

const uiStore = useUiStore();
const { t } = useI18n();

const builtinRows = computed(() => {
  const pluginF = shortcutHelpCtrlShiftFDescription(uiStore.experimentalPluginManagerV2);
  const pluginA = shortcutHelpCtrlShiftADescription();
  return [
    { keys: formatChordModShift("D"), desc: String(t("shortcutHelp.rows.ctrlShiftD")) },
    { keys: formatChordModShift("S"), desc: String(t("shortcutHelp.rows.ctrlShiftS")) },
    { keys: formatChordModShift("F"), desc: pluginF },
    { keys: formatChordModShift("A"), desc: pluginA },
    { keys: String(t("shortcutHelp.rows.ctrlHoldKey")), desc: String(t("shortcutHelp.rows.ctrlHoldDesc")) },
  ];
});
</script>

<template>
  <div class="smp">
    <section class="smp-section" aria-labelledby="smp-builtin-h">
      <h3 id="smp-builtin-h" class="smp-h">{{ t("settings.shortcutsManager.builtinTitle") }}</h3>
      <p class="smp-lead">{{ t("settings.shortcutsManager.builtinLead") }}</p>
      <table class="smp-table">
        <tbody>
          <tr v-for="(r, i) in builtinRows" :key="`${r.keys}-${i}`">
            <td class="smp-keys">{{ r.keys }}</td>
            <td class="smp-desc">{{ r.desc }}</td>
          </tr>
        </tbody>
      </table>
      <p class="smp-foot">{{ t("shortcutHelp.footer") }}</p>
    </section>

    <section class="smp-section" :aria-label="String(t('shortcutHelp.launcherSlot.aria'))">
      <h3 class="smp-h">{{ t("shortcutHelp.launcherSlot.title") }}</h3>
      <PluginSlotEmbed
        :slot-name="SLOT_LAUNCHER_PALETTE"
        :aria-label="String(t('shortcutHelp.launcherSlot.embedAria'))"
        :bootstrap-epoch="bootstrapEpoch"
      />
    </section>

    <section class="smp-section" aria-labelledby="smp-global-h">
      <h3 id="smp-global-h" class="smp-h">{{ t("settings.shortcutsManager.globalTitle") }}</h3>
      <p class="smp-lead">{{ t("settings.shortcutsManager.globalLead") }}</p>
      <HotkeySettingsSection headless />
    </section>
  </div>
</template>

<style scoped>
.smp {
  display: flex;
  flex-direction: column;
  gap: 22px;
  max-width: 720px;
}
.smp-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.smp-h {
  margin: 0;
  font-size: 15px;
}
.smp-lead {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.smp-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.smp-table td {
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-light);
  vertical-align: top;
}
.smp-keys {
  white-space: nowrap;
  font-family: ui-monospace, monospace;
  width: 38%;
  color: var(--text-primary);
}
.smp-desc {
  color: var(--text-secondary);
  line-height: 1.45;
}
.smp-foot {
  margin: 4px 0 0;
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
