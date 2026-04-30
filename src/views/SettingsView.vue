<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import HelpHint from "../components/HelpHint.vue";
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
const uiStore = useUiStore();
const { showToast } = useAppToast();
const { t } = useI18n();

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
    if (!visible || marketSourcesLoaded.value) return;
    marketSourcesLoaded.value = true;
    void loadMarketSources();
  },
);

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
    <div
      v-if="visible"
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
          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">{{ t("settings.shortcuts.label") }}</span>
              <HelpHint :text="settingsShortcutsHelpHint()" />
            </div>
            <p class="sv-muted">
              {{ t("settings.shortcuts.immersiveHint") }}
            </p>
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
          <section class="sv-section">
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
          <section class="sv-section">
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
              <span class="sv-label">安全</span>
            </div>
            <label class="sv-toggle-row">
              <input
                type="checkbox"
                :checked="pluginStore.pluginState.force_iframe_mode === true"
                @change="onToggleForceIframe"
              />
              <span class="sv-toggle-text">
                <strong>强制 iframe 模式</strong>
                <span class="sv-muted sv-toggle-desc">
                  开启后，所有插件界面将使用 iframe 加载，更安全但体验可能下降。保存后需重启应用以完全生效。
                </span>
              </span>
            </label>
          </section>

          <section class="sv-section">
            <div class="sv-row-h">
              <span class="sv-label">高级（开发者模式）</span>
              <HelpHint
                :paragraphs="[
                  '开发者模式用于启用第三方插件索引源与侧载安装入口。',
                  '开启后请仅添加你信任的索引源；未签名源安装会强提示。',
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
                <strong>启用开发者模式</strong>
                <span class="sv-muted sv-toggle-desc">
                  开启后可使用第三方索引源与侧载安装。建议仅高级用户开启。
                </span>
              </span>
            </label>
            <div v-if="marketDeveloperModeLocal" class="sv-dev-box">
              <p class="sv-muted">
                第三方索引源（每行一个 URL）。保存后可在插件管理页用“同步在线索引”并传入该 URL（后续会提供源选择 UI）。
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
                  保存第三方源列表
                </button>
              </div>
            </div>
          </section>
        </div>

        <div v-show="tab === 'plugins'" class="sv-body">
          <section class="sv-section">
            <div class="sv-row-h">
              <h3 class="sv-h3">目录插件 · 设置页插槽</h3>
              <HelpHint
                :paragraphs="[
                  '在插件 manifest 的 ui_slots 中声明 slot 为 settings.panel，即可在此嵌入配置页。',
                  '与 chat_toolbar 相同，使用 https://ocliveplugin.localhost/<id>/<entry> 加载；可在插件管理中调整顺序或隐藏。',
                ]"
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
</style>
