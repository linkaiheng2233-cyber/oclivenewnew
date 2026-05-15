<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import HelpHint from "../components/HelpHint.vue";
import HotkeySettingsSection from "../components/HotkeySettingsSection.vue";
import PluginSettingsPanelSlots from "../components/PluginSettingsPanelSlots.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import { useAppToast } from "../composables/useAppToast";
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from "../stores/pluginStore";
import { useUiStore } from "../stores/uiStore";
import {
  runEnvironmentDiagnostics,
  type EnvironmentDiagnostics,
} from "../utils/tauri-api";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  /** 在开启 V2 实验开关时，由设置页打开 V2 预览窗 */
  openPluginV2: [];
}>();

const { t } = useI18n();
const pluginStore = usePluginStore();
const uiStore = useUiStore();
const { showToast } = useAppToast();

type SettingsTab = "general" | "plugins";

const tab = ref<SettingsTab>("general");

const envDiagLoading = ref(false);
const envDiag = ref<EnvironmentDiagnostics | null>(null);

async function onRunEnvironmentDiagnostics() {
  envDiagLoading.value = true;
  envDiag.value = null;
  try {
    envDiag.value = await runEnvironmentDiagnostics();
    showToast("info", t("settings.envCheckDoneToast"));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    envDiagLoading.value = false;
  }
}

async function onToggleForceIframe(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  pluginStore.pluginState = {
    ...pluginStore.pluginState,
    force_iframe_mode: checked,
  };
  try {
    await pluginStore.persist();
    showToast("info", t("settings.iframeSavedInfo"));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
    pluginStore.pluginState = {
      ...pluginStore.pluginState,
      force_iframe_mode: !checked,
    };
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="sv-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('settings.ariaDialog')"
      @click.self="emit('close')"
    >
      <div class="sv-dialog" @click.stop>
        <header class="sv-head">
          <h2 class="sv-title">{{ t("settings.title") }}</h2>
          <button type="button" class="sv-close" :aria-label="t('settings.closeAria')" @click="emit('close')">×</button>
        </header>

        <nav class="sv-nav" :aria-label="t('settings.ariaNav')">
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'general' ? 'page' : undefined"
            @click="tab = 'general'"
          >
            {{ t("settings.tabGeneral") }}
          </button>
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'plugins' ? 'page' : undefined"
            @click="tab = 'plugins'"
          >
            {{ t("settings.tabPlugins") }}
          </button>
        </nav>

        <div v-show="tab === 'general'" class="sv-body">
          <p class="sv-lead" v-html="t('settings.generalLeadHtml')" />
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.shortcutsLabel") }}</span>
              <HelpHint :text="t('settings.shortcutsHelp')" />
            </div>
            <p class="sv-muted">
              {{ t("settings.immersiveOnlyNote") }}
            </p>
          </section>
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.envCheckTitle") }}</span>
              <HelpHint :text="t('settings.envCheckHelp')" />
            </div>
            <p class="sv-muted">{{ t("settings.envCheckLead") }}</p>
            <button
              type="button"
              class="sv-env-btn"
              :disabled="envDiagLoading"
              @click="onRunEnvironmentDiagnostics"
            >
              {{ envDiagLoading ? t("settings.envCheckRunning") : t("settings.envCheckRun") }}
            </button>
            <div v-if="envDiag" class="sv-env-results" role="status">
              <p class="sv-env-line">
                <strong>{{ t("settings.envCheckOllama", { url: envDiag.ollamaBaseUrl }) }}</strong>
                —
                <span :class="envDiag.ollamaReachable ? 'sv-ok' : 'sv-bad'">
                  {{
                    envDiag.ollamaReachable
                      ? t("settings.envCheckOllamaOk")
                      : t("settings.envCheckOllamaFail")
                  }}
                </span>
                <span v-if="envDiag.ollamaDetail" class="sv-muted sv-detail">
                  {{ t("settings.envCheckDetail") }} {{ envDiag.ollamaDetail }}
                </span>
              </p>
              <p class="sv-env-line">
                <strong>{{ t("settings.envCheckRoles") }}</strong>
                —
                <span
                  :class="
                    envDiag.rolesDirExists && envDiag.rolesDirReadable ? 'sv-ok' : 'sv-bad'
                  "
                >
                  {{
                    !envDiag.rolesDirExists
                      ? t("settings.envCheckRolesMissing")
                      : !envDiag.rolesDirReadable
                        ? t("settings.envCheckRolesUnreadable")
                        : t("settings.envCheckRolesOk")
                  }}
                </span>
                <code class="sv-code">{{ envDiag.rolesDir }}</code>
              </p>
              <p class="sv-muted sv-small">{{ t("settings.envCheckRolesHint") }}</p>
              <p class="sv-env-line">
                <strong>{{ t("settings.envCheckAppData") }}</strong>
                —
                <span :class="envDiag.appDataWritable ? 'sv-ok' : 'sv-bad'">
                  {{
                    envDiag.appDataWritable
                      ? t("settings.envCheckAppDataOk")
                      : t("settings.envCheckAppDataFail")
                  }}
                </span>
                <span v-if="envDiag.appDataDetail" class="sv-muted sv-detail">
                  {{ t("settings.envCheckDetail") }} {{ envDiag.appDataDetail }}
                </span>
                <code class="sv-code">{{ envDiag.appDataDir }}</code>
              </p>
            </div>
          </section>
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.experimentalLabel") }}</span>
              <HelpHint :text="t('settings.experimentalSectionHelp')" />
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="uiStore.experimentalPluginManagerV2 === true"
                @change="uiStore.setExperimentalPluginManagerV2(($event.target as HTMLInputElement).checked)"
              />
              <span class="sv-toggle-text">
                <strong>{{ t("settings.experimentalToggleTitle") }}</strong>
                <span class="sv-muted sv-toggle-desc" v-html="t('settings.experimentalToggleHtml')" />
              </span>
            </label>
            <div v-if="uiStore.experimentalPluginManagerV2" class="sv-v2-launch">
              <button type="button" class="sv-v2-launch-btn" @click="emit('openPluginV2')">
                {{ t("settings.openV2Preview") }}
              </button>
            </div>
          </section>
          <section class="sv-section">
            <h3 class="sv-h3">{{ t("settings.advancedTitle") }}</h3>
            <p class="sv-muted" v-html="t('settings.advancedDesc')" />
            <PluginSlotEmbed
              :slot-name="SLOT_SETTINGS_ADVANCED"
              :aria-label="t('settings.advancedSlotAria')"
              :bootstrap-epoch="pluginStore.bootstrapEpoch"
            />
          </section>

          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.securityLabel") }}</span>
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="pluginStore.pluginState.force_iframe_mode === true"
                @change="onToggleForceIframe"
              />
              <span class="sv-toggle-text">
                <strong>{{ t("settings.forceIframeTitle") }}</strong>
                <span class="sv-muted sv-toggle-desc">
                  {{ t("settings.forceIframeDesc") }}
                </span>
              </span>
            </label>
          </section>
        </div>

        <div v-show="tab === 'plugins'" class="sv-body">
          <section class="sv-section">
            <div class="sv-row-h">
              <h3 class="sv-h3">{{ t("settings.pluginsPanelTitle") }}</h3>
              <HelpHint
                :paragraphs="[t('settings.pluginsPanelHint1'), t('settings.pluginsPanelHint2')]"
              />
            </div>
            <PluginSettingsPanelSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
          </section>

          <HotkeySettingsSection />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.sv-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10040;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.sv-dialog {
  position: relative;
  width: min(640px, 100%);
  max-height: min(90vh, 800px);
  overflow: auto;
  padding: 16px 18px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.sv-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 8px;
}
.sv-title {
  margin: 0;
  font-size: 18px;
}
.sv-close {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.sv-nav {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid var(--border-light);
  padding-bottom: 8px;
}
.sv-nav-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-nav-btn[aria-current="page"] {
  border-color: var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.sv-body {
  flex: 1;
  min-height: 0;
}
.sv-lead {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.sv-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sv-row-h {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.sv-label {
  font-weight: 600;
  font-size: 14px;
}
.sv-h3 {
  margin: 0;
  font-size: 15px;
}
.sv-muted {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}
.sv-toggle-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  cursor: pointer;
  font-size: 13px;
  line-height: 1.45;
}
.sv-toggle-row input {
  margin-top: 3px;
  flex-shrink: 0;
}
.sv-toggle-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.sv-toggle-desc {
  display: block;
  font-weight: 400;
}
.sv-v2-launch {
  margin-top: 4px;
}
.sv-v2-launch-btn {
  padding: 8px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
}
.sv-v2-launch-btn:hover {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 45%, var(--border-light));
}
.sv-env-btn {
  align-self: flex-start;
  padding: 8px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
}
.sv-env-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.sv-env-results {
  margin-top: 4px;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sv-env-line {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-primary);
}
.sv-small {
  font-size: 11px;
}
.sv-detail {
  display: block;
  margin-top: 2px;
  word-break: break-word;
}
.sv-code {
  display: block;
  margin-top: 4px;
  font-size: 11px;
  padding: 4px 6px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--border-light) 35%, transparent);
  word-break: break-all;
}
.sv-ok {
  color: var(--success, #15803d);
}
.sv-bad {
  color: var(--danger, #b91c1c);
}
</style>
