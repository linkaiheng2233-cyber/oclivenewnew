<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { confirm } from "@tauri-apps/api/dialog";
import HelpHint from "../components/HelpHint.vue";
import TrustConsentModal from "../components/TrustConsentModal.vue";
import CloudLlmQuickSetup from "../components/CloudLlmQuickSetup.vue";
import { buildCloudLlmTrustPlainText, useCloudLlmTrustModal } from "../composables/useCloudLlmTrustModal";
import { isTauriWebview } from "../utils/isTauriWebview";
import HotkeySettingsSection from "../components/HotkeySettingsSection.vue";
import SettingsTierSection from "../components/SettingsTierSection.vue";
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
  unifiedOpenAgentDebugFromBackendsCta,
  unifiedOpenDebugCta,
  unifiedOpenPluginManagerBackendsCta,
  unifiedOpenPluginManagerInstalledCta,
  unifiedOpenPluginManagerSlotsCta,
  unifiedOpenPluginManagerV2HubCta,
  unifiedOpenPluginMarketCta,
} from "../lib/pluginManagerEntryCopy";
import type { SettingsDeepLink } from "../lib/settingsDeepLink";
import {
  ALL_SETTINGS_NAV_IDS,
  SETTINGS_NAV,
  SETTINGS_NAV_ROWS,
  filterSettingsNavRows,
  firstSelectableSettingsNavId,
  settingsNavLabelKey,
  type SettingsNavAnyId,
  type SettingsNavId,
  type SettingsNavRow,
} from "../lib/settingsNavKeys";
import { settingsDeepLinkFooterNote, settingsTierBadge, settingsTierDescription } from "../lib/settingsNavCopy";
import type { PluginPanelMainTab } from "../stores/pluginStore";
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
  /** 关闭设置并由宿主打开既有面板（插件管理 / 市场 / 本机模型 / 专家工作台 / 调试） */
  deepLink: [SettingsDeepLink];
}>();

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const uiStore = useUiStore();
const { showToast } = useAppToast();
const { t } = useI18n();
const cloudTrust = useCloudLlmTrustModal();

const showVueCloudTrustModal = computed(
  () => !isTauriWebview() || cloudTrust.visible.value,
);

const unifiedMarketCtaText = computed(() => unifiedOpenPluginMarketCta(uiStore.experimentalPluginManagerV2));
const unifiedDebugCtaText = computed(() => unifiedOpenDebugCta());

const PluginManagerPanel = defineAsyncComponent(() => import("./PluginManagerPanel.vue"));

function settingsNavToPluginTab(nav: SettingsNavId): PluginPanelMainTab | null {
  if (nav === SETTINGS_NAV.pluginsLinkInstalled) return "plugins";
  if (nav === SETTINGS_NAV.pluginsLinkSlots) return "slots";
  if (nav === SETTINGS_NAV.pluginsLinkBackends) return "backends";
  return null;
}

async function openPluginManagerEmbed(tab: PluginPanelMainTab): Promise<void> {
  await pluginStore.openPanelInSettingsEmbed(tab);
}

const selectedNavId = ref<SettingsNavId>(SETTINGS_NAV.generalOverview);

const visibleNavRows = computed(() =>
  filterSettingsNavRows(roleStore.interactionImmersive, SETTINGS_NAV_ROWS),
);

function navLabel(id: SettingsNavAnyId): string {
  return String(t(`settings.nav.items.${settingsNavLabelKey(id)}`));
}

function selectNav(id: SettingsNavAnyId): void {
  if (!ALL_SETTINGS_NAV_IDS.includes(id as SettingsNavId)) return;
  selectedNavId.value = id as SettingsNavId;
}

function tierAbbr(row: SettingsNavRow): string {
  if (!row.tier || row.isGroupLabel) return "";
  return settingsTierBadge(row.tier);
}

function tierTitle(row: SettingsNavRow): string {
  if (!row.tier || row.isGroupLabel) return "";
  return settingsTierDescription(row.tier);
}

function emitDeepLink(link: SettingsDeepLink): void {
  emit("deepLink", link);
}

async function openCloudLlmTrustReadme(): Promise<void> {
  if (!isTauriWebview()) {
    cloudTrust.open();
    return;
  }
  try {
    await confirm(buildCloudLlmTrustPlainText((k) => String(t(k))), {
      title: String(t("settings.cloudLlmTrust.modal.title")),
      type: "info",
      okLabel: String(t("settings.cloudLlmTrust.modal.allow")),
      cancelLabel: String(t("common.cancel")),
    });
  } catch (e) {
    console.warn("[cloudLlmTrust] native dialog failed, using in-app modal", e);
    cloudTrust.open();
  }
}

function onTrustModalVisible(v: boolean): void {
  cloudTrust.visible.value = v;
}

/** 设置窗关闭时递增，用于将各页内 L4 分区恢复默认折叠 */
const tierResetKey = ref(0);
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
      tierResetKey.value += 1;
    } else {
      selectedNavId.value = firstSelectableSettingsNavId(roleStore.interactionImmersive);
    }
    if (!visible || marketSourcesLoaded.value) return;
    marketSourcesLoaded.value = true;
    void loadMarketSources();
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  cloudTrust.close();
});

watch(
  () => [roleStore.interactionImmersive, roleStore.interactionPureChat] as const,
  () => {
    const immersive = roleStore.interactionImmersive;
    const allowed = new Set(
      filterSettingsNavRows(immersive, SETTINGS_NAV_ROWS)
        .filter((r) => !r.isGroupLabel)
        .map((r) => r.id as SettingsNavId),
    );
    if (!allowed.has(selectedNavId.value)) {
      selectedNavId.value = firstSelectableSettingsNavId(immersive);
    }
  },
);

watch(
  () =>
    [props.visible, selectedNavId.value, pluginStore.panelEmbedHost] as [
      boolean,
      SettingsNavId,
      typeof pluginStore.panelEmbedHost,
    ],
  ([visible, nav, host]) => {
    if (!visible) {
      if (host === "settings") pluginStore.closePanel();
      return;
    }
    if (host !== "settings") return;
    const tab = settingsNavToPluginTab(nav);
    if (tab) {
      if (pluginStore.panelMainTab !== tab) pluginStore.panelMainTab = tab;
    } else {
      pluginStore.closePanel();
    }
  },
);

async function onOpenPluginBackendsFromCloud(): Promise<void> {
  try {
    emitDeepLink({ kind: "plugin_manager", tab: "backends" });
    showToast("info", String(t("settings.cloudLlmTrust.toastOpenedBackends")));
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
          <div class="sv-dialog-scroll">
          <header class="sv-head sv-head--sticky">
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

          <div class="sv-shell">
            <nav class="sv-tree" :aria-label="t('settings.sectionsNavLabel')">
              <template v-for="(row, idx) in visibleNavRows" :key="`${row.id}-${idx}`">
                <div
                  v-if="row.isGroupLabel"
                  class="sv-tree-group"
                  :class="{ 'sv-tree-group--indented': row.depth === 1 }"
                >
                  {{ navLabel(row.id) }}
                </div>
                <button
                  v-else
                  type="button"
                  class="sv-tree-btn"
                  :class="{ 'sv-tree-btn--depth1': row.depth === 1 }"
                  :aria-current="selectedNavId === row.id ? 'page' : undefined"
                  @click="selectNav(row.id)"
                >
                  <span class="sv-tree-btn-label">{{ navLabel(row.id) }}</span>
                  <abbr
                    v-if="row.tier"
                    class="sv-tier"
                    :title="tierTitle(row)"
                  >{{ tierAbbr(row) }}</abbr>
                </button>
              </template>
            </nav>

            <div class="sv-pane">
              <div v-show="selectedNavId === SETTINGS_NAV.generalOverview" class="sv-pane-section">
                <SettingsTierSection tier="L1" :reset-key="tierResetKey">
                  <p class="sv-lead" v-html="settingsGeneralLeadHtml()" />
                  <p v-if="roleStore.interactionPureChat" class="sv-boundary sv-muted">
                    {{ t("settings.pureChatBoundary") }}
                  </p>
                  <p v-if="roleStore.interactionPureChat" class="sv-boundary-foot sv-muted">
                    {{ t("settings.pureChatMoreInImmersive") }}
                  </p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.generalLanguage" class="sv-pane-section">
                <SettingsTierSection tier="L1" :reset-key="tierResetKey">
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
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.shortcutsMain" class="sv-pane-section">
                <SettingsTierSection tier="L1" :reset-key="tierResetKey">
                  <section class="sv-section">
                    <div class="sv-row-h">
                      <span class="sv-label">{{ t("settings.shortcuts.label") }}</span>
                      <HelpHint :text="settingsShortcutsHelpHint()" />
                    </div>
                    <p class="sv-muted">
                      {{ t("settings.shortcuts.immersiveHint") }}
                    </p>
                  </section>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.modelsCloud" class="sv-pane-section">
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
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
                      <CloudLlmQuickSetup />
                    </div>
                    <div class="sv-cloud-actions-row">
                      <button type="button" class="sv-btn sv-btn--accent" @click="openCloudLlmTrustReadme">
                        {{ t("settings.cloudLlmTrust.reviewCta") }}
                      </button>
                      <button type="button" class="sv-btn" @click="onOpenPluginBackendsFromCloud">
                        {{ t("settings.cloudLlmTrust.openBackendsCta") }}
                      </button>
                    </div>
                  </section>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.modelsOllama" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.modelsOllama") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button type="button" class="sv-btn sv-btn--accent" @click="emitDeepLink({ kind: 'local_models' })">
                    {{ t("settings.nav.cta.openLocalModels") }}
                  </button>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.modelsExpert" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.modelsExpert") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button
                    type="button"
                    class="sv-btn sv-btn--accent"
                    @click="emitDeepLink({ kind: 'expert_workbench', draftMode: 'effective' })"
                  >
                    {{ t("settings.nav.cta.openExpertWorkbench") }}
                  </button>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsDirectory" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
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
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsHotkeys" class="sv-pane-section">
                <SettingsTierSection tier="L1" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("hotkeySettings.tierL1Intro") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <HotkeySettingsSection headless />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsLinkInstalled" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsInstalled") }}</p>
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn secondary" @click="openPluginManagerEmbed('plugins')">
                      {{ t("settings.nav.cta.openPluginManagerInPage") }}
                    </button>
                  </div>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <div class="sv-btn-row">
                    <button
                      type="button"
                      class="sv-btn sv-btn--accent"
                      @click="emitDeepLink({ kind: 'plugin_manager', tab: 'plugins' })"
                    >
                      {{ unifiedOpenPluginManagerInstalledCta() }}
                    </button>
                  </div>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsLinkSlots" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsSlots") }}</p>
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn secondary" @click="openPluginManagerEmbed('slots')">
                      {{ t("settings.nav.cta.openPluginManagerInPage") }}
                    </button>
                  </div>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <div class="sv-btn-row">
                    <button
                      type="button"
                      class="sv-btn sv-btn--accent"
                      @click="emitDeepLink({ kind: 'plugin_manager', tab: 'slots' })"
                    >
                      {{ unifiedOpenPluginManagerSlotsCta() }}
                    </button>
                  </div>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsLinkBackends" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsBackends") }}</p>
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn secondary" @click="openPluginManagerEmbed('backends')">
                      {{ t("settings.nav.cta.openPluginManagerInPage") }}
                    </button>
                  </div>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <div class="sv-btn-row">
                    <button
                      type="button"
                      class="sv-btn sv-btn--accent"
                      @click="emitDeepLink({ kind: 'plugin_manager', tab: 'backends' })"
                    >
                      {{ unifiedOpenPluginManagerBackendsCta() }}
                    </button>
                  </div>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsV2Hub" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsV2Hub") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button type="button" class="sv-btn sv-btn--accent" @click="emit('openPluginV2')">
                    {{ unifiedOpenPluginManagerV2HubCta() }}
                  </button>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.marketBrowse" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.marketBrowse") }}</p>
                  <p v-if="uiStore.experimentalPluginManagerV2" class="sv-callout sv-muted">
                    {{ t("settings.nav.lead.marketBrowseV2Hint") }}
                  </p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button type="button" class="sv-btn sv-btn--accent" @click="emitDeepLink({ kind: 'plugin_market' })">
                    {{ unifiedMarketCtaText }}
                  </button>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.securityHost" class="sv-pane-section">
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
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
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.advancedExperimental" class="sv-pane-section">
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
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
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.advancedEmbed" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <section class="sv-section">
                    <h3 class="sv-h3">{{ t("settings.advancedSlot.title") }}</h3>
                    <p class="sv-muted">{{ t("settings.advancedSlot.hint") }}</p>
                    <PluginSlotEmbed
                      :slot-name="SLOT_SETTINGS_ADVANCED"
                      :aria-label="String(t('settings.advancedSlot.aria'))"
                      :bootstrap-epoch="pluginStore.bootstrapEpoch"
                    />
                  </section>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.advancedMarketSources" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <section class="sv-section">
                    <div class="sv-row-h">
                      <span class="sv-label">{{ t("settings.plugins.devMode.sectionLabel") }}</span>
                      <HelpHint
                        :paragraphs="[
                          t('settings.plugins.devMode.help.p1'),
                          t('settings.plugins.devMode.help.p2'),
                        ]"
                      />
                    </div>
                  </section>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <section class="sv-section">
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
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.diagnosticsDebug" class="sv-pane-section">
                <SettingsTierSection tier="L2" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.diagnosticsDebug") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button type="button" class="sv-btn sv-btn--accent" @click="emitDeepLink({ kind: 'debug_panel' })">
                    {{ unifiedDebugCtaText }}
                  </button>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.diagnosticsAgent" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.diagnosticsAgent") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button
                    type="button"
                    class="sv-btn sv-btn--accent"
                    @click="emitDeepLink({ kind: 'plugin_manager', tab: 'backends' })"
                  >
                    {{ unifiedOpenAgentDebugFromBackendsCta() }}
                  </button>
                  <p class="sv-muted sv-foot">{{ settingsDeepLinkFooterNote() }}</p>
                </SettingsTierSection>
              </div>
            </div>
          </div>

          <div
            v-if="pluginStore.panelVisible && pluginStore.panelEmbedHost === 'settings'"
            class="sv-pm-embed"
          >
            <PluginManagerPanel embedded />
          </div>
          </div>
        </div>
      </div>
      <TrustConsentModal
        v-if="showVueCloudTrustModal"
        :model-value="cloudTrust.visible"
        :title="cloudTrust.modalTitle"
        :subtitle="cloudTrust.modalSubtitle"
        :trust-summary-title="cloudTrust.trustSummaryTitle"
        :trust-summary="cloudTrust.trustSummaryBody"
        :hint="cloudTrust.modalHint"
        :capabilities="cloudTrust.capabilities"
        :confirm-label="cloudTrust.confirmLabel"
        variant="trust"
        require-explicit-dismiss
        @update:model-value="onTrustModalVisible"
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
  width: min(920px, 100%);
  max-height: min(90vh, 820px);
  overflow: hidden;
  padding: 0;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
  display: flex;
  flex-direction: column;
}
.sv-dialog-scroll {
  flex: 1;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px 18px 18px;
}
.sv-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 8px;
  flex-shrink: 0;
}
.sv-head--sticky {
  position: sticky;
  top: 0;
  z-index: 3;
  margin: -4px -6px 0 -4px;
  padding: 6px 6px 10px 10px;
  background: var(--bg-primary);
  box-shadow: 0 10px 14px -12px color-mix(in srgb, var(--bg-primary) 55%, #000);
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
.sv-shell {
  display: flex;
  gap: 14px;
  min-height: 0;
  flex: 0 0 auto;
  align-items: flex-start;
}
.sv-tree {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: visible;
  padding-right: 10px;
  border-right: 1px solid var(--border-light);
}
.sv-tree-group {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-secondary);
  margin-top: 10px;
  margin-bottom: 2px;
  padding: 4px 8px 2px;
}
.sv-tree-group:first-child {
  margin-top: 0;
}
.sv-tree-group--indented {
  padding-left: 12px;
}
.sv-tree-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  font-size: 13px;
  line-height: 1.35;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
}
.sv-tree-btn--depth1 {
  padding-left: 16px;
  font-size: 12px;
}
.sv-tree-btn:hover {
  background: color-mix(in srgb, var(--border-light) 45%, transparent);
  color: var(--text-primary);
}
.sv-tree-btn[aria-current="page"] {
  border-color: var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.sv-tree-btn-label {
  flex: 1;
  min-width: 0;
}
.sv-tier {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 700;
  text-decoration: none;
  padding: 2px 5px;
  border-radius: 4px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
  cursor: help;
}
.sv-pane {
  flex: 1;
  min-width: 0;
  overflow: visible;
  padding-right: 4px;
}
.sv-callout {
  margin: 0;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--accent) 28%, var(--border-light));
  background: color-mix(in srgb, var(--accent) 8%, var(--bg-secondary));
  font-size: 12px;
  line-height: 1.5;
}
.sv-btn-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}
.sv-pm-embed {
  margin-top: 8px;
  padding-top: 14px;
  border-top: 1px solid var(--border-light);
}
.sv-pane-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding-bottom: 8px;
}
.sv-foot {
  margin-top: 4px;
  font-size: 11px;
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
