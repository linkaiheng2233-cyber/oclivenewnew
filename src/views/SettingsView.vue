<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import HelpHint from "../components/HelpHint.vue";
import TrustConsentModal from "../components/TrustConsentModal.vue";
import { useCloudLlmTrustModal } from "../composables/useCloudLlmTrustModal";
import HotkeySettingsSection from "../components/HotkeySettingsSection.vue";
import PluginSettingsPanelSlots from "../components/PluginSettingsPanelSlots.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import { useAppToast } from "../composables/useAppToast";
import type { LanguagePref } from "../i18n";
import {
  settingsExperimentalSectionHelpHint,
  settingsExperimentalToggleDescriptionHtml,
  settingsGeneralLeadHtml,
  settingsOpenV2PreviewButtonLabel,
  settingsShortcutsHelpHint,
} from "../lib/pluginManagerEntryCopy";
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import {
  getPluginMarketSourcesConfig,
  setPluginIndexSources,
  setPluginMarketDeveloperMode,
} from "../utils/tauri-api";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  /** 在开启 V2 实验开关时，由设置页打开 V2 预览窗 */
  openPluginV2: [];
}>();

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const uiStore = useUiStore();
const { showToast } = useAppToast();
const { t } = useI18n();
const cloudTrust = useCloudLlmTrustModal();

type SettingsTab = "general" | "plugins";

const tab = ref<SettingsTab>("general");

const marketSourcesLoading = ref(false);
const marketDeveloperModeLocal = ref(false);
const marketSourcesText = ref("");
const marketSourcesLoaded = ref(false);

async function loadMarketSources(): Promise<void> {
  marketSourcesLoading.value = true;
  try {
    const cfg = await getPluginMarketSourcesConfig();
    marketDeveloperModeLocal.value = cfg.developerMode === true;
    marketSourcesText.value = (cfg.pluginIndexSources ?? []).join("\n");
  } finally {
    marketSourcesLoading.value = false;
  }
}

watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      cloudTrust.close();
    }
    if (!visible || marketSourcesLoaded.value) return;
    marketSourcesLoaded.value = true;
    void loadMarketSources();
  },
);

watch(
  () => roleStore.interactionPureChat,
  (pure) => {
    if (pure && tab.value === "plugins") tab.value = "general";
  },
);

async function onOpenPluginBackendsFromCloud(): Promise<void> {
  try {
    await pluginStore.openPanel("backends");
    showToast("info", String(t("settings.cloudLlmTrust.toastOpenedBackends")));
    emit("close");
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function onToggleMarketDeveloperMode(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  marketSourcesLoading.value = true;
  try {
    const cfg = await setPluginMarketDeveloperMode(checked);
    marketDeveloperModeLocal.value = cfg.developerMode === true;
    showToast(
      "success",
      String(t(checked ? "settings.plugins.devMode.enabledToast" : "settings.plugins.devMode.disabledToast")),
    );
    await pluginStore.syncDirectoryPluginBootstrap();
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    marketSourcesLoading.value = false;
  }
}

async function onSaveMarketSources() {
  const lines = marketSourcesText.value
    .split(/\r?\n/g)
    .map((s) => s.trim())
    .filter(Boolean);
  marketSourcesLoading.value = true;
  try {
    const cfg = await setPluginIndexSources(lines);
    marketSourcesText.value = (cfg.pluginIndexSources ?? []).join("\n");
    showToast("success", String(t("settings.plugins.sources.savedToast")));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    marketSourcesLoading.value = false;
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
    showToast("info", String(t("settings.security.forceIframeSavedToast")));
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
    <template v-if="visible">
    <div
      class="sv-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="t('settings.title')"
      @click.self="emit('close')"
    >
      <div class="sv-dialog" @click.stop>
        <header class="sv-head">
          <h2 class="sv-title">{{ t("settings.title") }}</h2>
          <button
            type="button"
            class="sv-close"
            :aria-label="t('common.close')"
            @click="emit('close')"
          >
            ×
          </button>
        </header>

        <nav class="sv-nav" :aria-label="t('settings.sectionsNavLabel')">
          <button
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'general' ? 'page' : undefined"
            @click="tab = 'general'"
          >
            {{ t("settings.tabs.general") }}
          </button>
          <button
            v-if="roleStore.interactionImmersive"
            type="button"
            class="sv-nav-btn"
            :aria-current="tab === 'plugins' ? 'page' : undefined"
            @click="tab = 'plugins'"
          >
            {{ t("settings.tabs.plugins") }}
          </button>
        </nav>

        <div v-show="tab === 'general'" class="sv-body">
          <p class="sv-lead" v-html="settingsGeneralLeadHtml()" />
          <p v-if="roleStore.interactionPureChat" class="sv-boundary sv-muted">
            {{ t("settings.pureChatBoundary") }}
          </p>
          <section v-if="roleStore.interactionImmersive" class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.shortcuts.label") }}</span>
              <HelpHint :text="settingsShortcutsHelpHint()" />
            </div>
            <p class="sv-muted">
              {{ t("settings.shortcuts.immersiveHint") }}
            </p>
          </section>
          <section v-else class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.shortcuts.pureChatLabel") }}</span>
            </div>
            <p class="sv-muted">{{ t("settings.shortcuts.pureChatHint") }}</p>
          </section>
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.language.label") }}</span>
            </div>
            <div class="sv-row-controls">
              <select
                class="sv-select"
                :value="uiStore.languagePref"
                @change="uiStore.setLanguagePref(($event.target as HTMLSelectElement).value as LanguagePref)"
              >
                <option value="system">{{ t("settings.language.options.system") }}</option>
                <option value="zh-CN">{{ t("settings.language.options.zhCN") }}</option>
                <option value="en-US">{{ t("settings.language.options.enUS") }}</option>
              </select>
              <p class="sv-muted">{{ t("settings.language.hint") }}</p>
            </div>
          </section>

          <section class="sv-section sv-cloud-section">
            <h3 class="sv-h3">{{ t("settings.cloudLlmTrust.sectionTitle") }}</h3>
            <p class="sv-muted">{{ t("settings.cloudLlmTrust.sectionLead") }}</p>
            <div class="sv-cloud-card">
              <div class="sv-cloud-card-h">{{ t("settings.cloudLlmTrust.envTitle") }}</div>
              <ul class="sv-cloud-env-list">
                <li>{{ t("settings.cloudLlmTrust.envLineBase") }}</li>
                <li>{{ t("settings.cloudLlmTrust.envLineKey") }}</li>
                <li>{{ t("settings.cloudLlmTrust.envLineModel") }}</li>
                <li>{{ t("settings.cloudLlmTrust.envLineTimeout") }}</li>
              </ul>
            </div>
            <div class="sv-cloud-actions-row">
              <button type="button" class="sv-btn sv-btn--accent" @click="cloudTrust.open">
                {{ t("settings.cloudLlmTrust.reviewCta") }}
              </button>
              <button type="button" class="sv-btn" @click="onOpenPluginBackendsFromCloud">
                {{ t("settings.cloudLlmTrust.openBackendsCta") }}
              </button>
            </div>
          </section>

          <section v-if="roleStore.interactionImmersive" class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.experimental.label") }}</span>
              <HelpHint :text="settingsExperimentalSectionHelpHint()" />
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="uiStore.experimentalPluginManagerV2 === true"
                @change="uiStore.setExperimentalPluginManagerV2(($event.target as HTMLInputElement).checked)"
              />
              <span class="sv-toggle-text">
                <strong>{{ t("pluginManager.entry.settingsExperimentalToggleTitle") }}</strong>
                <span class="sv-muted sv-toggle-desc" v-html="settingsExperimentalToggleDescriptionHtml()" />
              </span>
            </label>
            <div v-if="uiStore.experimentalPluginManagerV2" class="sv-v2-launch">
              <button type="button" class="sv-v2-launch-btn" @click="emit('openPluginV2')">
                {{ settingsOpenV2PreviewButtonLabel() }}
              </button>
            </div>
          </section>
          <section v-if="roleStore.interactionImmersive" class="sv-section">
            <h3 class="sv-h3">{{ t("settings.advancedSlot.title") }}</h3>
            <p class="sv-muted">{{ t("settings.advancedSlot.hint") }}</p>
            <PluginSlotEmbed
              :slot-name="SLOT_SETTINGS_ADVANCED"
              :aria-label="String(t('settings.advancedSlot.aria'))"
              :bootstrap-epoch="pluginStore.bootstrapEpoch"
            />
          </section>

          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.security.label") }}</span>
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="pluginStore.pluginState.force_iframe_mode === true"
                @change="onToggleForceIframe"
              />
              <span class="sv-toggle-text">
                <strong>{{ t("settings.security.forceIframe.title") }}</strong>
                <span class="sv-muted sv-toggle-desc">
                  {{ t("settings.security.forceIframe.hint") }}
                </span>
              </span>
            </label>
          </section>

          <section v-if="roleStore.interactionImmersive" class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.plugins.devMode.sectionLabel") }}</span>
              <HelpHint
                :paragraphs="[
                  t('settings.plugins.devMode.help.p1'),
                  t('settings.plugins.devMode.help.p2'),
                ]"
              />
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :disabled="marketSourcesLoading"
                :checked="marketDeveloperModeLocal === true"
                @change="onToggleMarketDeveloperMode"
              />
              <span class="sv-toggle-text">
                <strong>{{ t("settings.plugins.devMode.title") }}</strong>
                <span class="sv-muted sv-toggle-desc">
                  {{ t("settings.plugins.devMode.hint") }}
                </span>
              </span>
            </label>
            <div v-if="marketDeveloperModeLocal" class="sv-dev-box">
              <p class="sv-muted">
                {{ t("settings.plugins.sources.hint") }}
              </p>
              <textarea
                v-model="marketSourcesText"
                class="sv-textarea"
                rows="4"
                spellcheck="false"
                placeholder="https://example.com/plugins.json"
              />
              <div class="sv-row-actions">
                <button
                  type="button"
                  class="sv-btn"
                  :disabled="marketSourcesLoading"
                  @click="onSaveMarketSources"
                >
                  {{ t("settings.plugins.sources.saveButton") }}
                </button>
              </div>
            </div>
          </section>
          <p v-if="roleStore.interactionPureChat" class="sv-boundary-foot sv-muted">
            {{ t("settings.pureChatMoreInImmersive") }}
          </p>
        </div>

        <div v-show="tab === 'plugins'" class="sv-body">
          <section class="sv-section">
            <div class="sv-row-h">
              <h3 class="sv-h3">{{ t("settings.plugins.directorySlot.title") }}</h3>
              <HelpHint
                :paragraphs="[
                  t('settings.plugins.directorySlot.help.p1'),
                  t('settings.plugins.directorySlot.help.p2'),
                ]"
              />
            </div>
            <PluginSettingsPanelSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
          </section>

          <HotkeySettingsSection />
        </div>
      </div>
    </div>
    <TrustConsentModal
      v-model="cloudTrust.visible"
      :title="cloudTrust.modalTitle"
      :subtitle="cloudTrust.modalSubtitle"
      :trust-summary-title="cloudTrust.trustSummaryTitle"
      :trust-summary="cloudTrust.trustSummaryBody"
      :hint="cloudTrust.modalHint"
      :capabilities="cloudTrust.capabilities"
      :confirm-label="cloudTrust.confirmLabel"
      variant="trust"
      require-explicit-dismiss
    />
    </template>
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
.sv-boundary {
  margin: -4px 0 12px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px dashed color-mix(in srgb, var(--border-light) 85%, var(--text-secondary) 15%);
  line-height: 1.45;
}
.sv-boundary-foot {
  margin: 12px 0 0;
  padding-top: 10px;
  border-top: 1px solid var(--border-light);
  line-height: 1.45;
}
.sv-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sv-dev-box {
  padding: 10px 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: 10px;
  background: var(--bg-elevated);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sv-textarea {
  width: 100%;
  font-size: 12px;
  line-height: 1.4;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  resize: vertical;
}
.sv-row-actions {
  display: flex;
  justify-content: flex-end;
}
.sv-btn {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
}
.sv-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.sv-row-h {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.sv-row-controls {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.sv-select {
  width: min(320px, 100%);
  padding: 6px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
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
.sv-cloud-section {
  padding-top: 4px;
}
.sv-cloud-card {
  margin-top: 6px;
  padding: 10px 12px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sv-cloud-card-h {
  font-size: 12px;
  font-weight: 650;
  color: var(--text-primary);
  margin-bottom: 8px;
}
.sv-cloud-env-list {
  margin: 0;
  padding-left: 18px;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-secondary);
}
.sv-cloud-env-list li {
  margin-bottom: 4px;
}
.sv-cloud-actions-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}
.sv-btn--accent {
  background: color-mix(in srgb, var(--accent, #3b82f6) 14%, var(--bg-primary));
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 38%, var(--border-light));
}
.sv-btn--accent:hover {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 55%, var(--border-light));
}
</style>
