<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { confirm } from "@tauri-apps/api/dialog";
import HelpHint from "../components/HelpHint.vue";
import TrustConsentModal from "../components/TrustConsentModal.vue";
import CloudLlmQuickSetup from "../components/CloudLlmQuickSetup.vue";
import { buildCloudLlmTrustPlainText, useCloudLlmTrustModal } from "../composables/useCloudLlmTrustModal";
import { notifyHostModelsInventoryChanged } from "../composables/useHostModelPick";
import { isTauriWebview } from "../utils/isTauriWebview";
import ShortcutsManagerPanel from "../components/settings/ShortcutsManagerPanel.vue";
import ModelSelectorSettings from "../components/settings/ModelSelectorSettings.vue";
import ExpertModelsSettingsHub from "../components/settings/ExpertModelsSettingsHub.vue";
import RoleManagerSettings from "../components/settings/RoleManagerSettings.vue";
import SettingsDebugEmbed from "../components/settings/SettingsDebugEmbed.vue";
import SettingsTierSection from "../components/SettingsTierSection.vue";
import PluginSettingsPanelSlots from "../components/PluginSettingsPanelSlots.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import { useAppToast } from "../composables/useAppToast";
import type { LanguagePref } from "../i18n";
import {
  settingsExperimentalSectionHelpHint,
  settingsExperimentalToggleDescriptionHtml,
  settingsGeneralLeadHtml,
  settingsShortcutsHelpHint,
  unifiedOpenAgentDebugFromBackendsCta,
  unifiedOpenPluginMarketCta,
} from "../lib/pluginManagerEntryCopy";
import {
  ALL_SETTINGS_NAV_IDS,
  SETTINGS_DEVELOPER_GATED_NAV_IDS,
  SETTINGS_NAV,
  SETTINGS_NAV_ROWS,
  filterSettingsNavRows,
  settingsNavLabelKey,
  type SettingsNavAnyId,
  type SettingsNavId,
  type SettingsNavRow,
} from "../lib/settingsNavKeys";
import { resetHostPreferencesToDefaults } from "../lib/resetHostPreferencesToDefaults";
import { settingsTierBadge, settingsTierDescription } from "../lib/settingsNavCopy";
import { formatChordModShift } from "../lib/shortcutDisplay";
import type { PluginPanelMainTab } from "../stores/pluginStore";
import { SLOT_SETTINGS_ADVANCED, usePluginStore } from "../stores/pluginStore";
import { useExpertModelsStore } from "../stores/expertModelsStore";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import type { ExpertWorkbenchDraftMode } from "../lib/expertWorkbenchOpen";
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
  /** 与顶栏切换角色相同的宿主流程（插件 bootstrap、叙事场景等） */
  switchRole: [roleId: string];
  /** 调试嵌入区内导入角色包 */
  packImported: [roleId: string];
  /** 调试嵌入区内「重载策略插件」 */
  reloadPolicy: [];
}>();

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const uiStore = useUiStore();
const expertModelsStore = useExpertModelsStore();
const { showToast } = useAppToast();
const { t } = useI18n();
const cloudTrust = useCloudLlmTrustModal();

const showVueCloudTrustModal = computed(
  () => !isTauriWebview() || cloudTrust.visible.value,
);

const unifiedMarketCtaText = computed(() =>
  unifiedOpenPluginMarketCta(
    uiStore.experimentalPluginManagerV2 && uiStore.settingsDeveloperMaster,
  ),
);

const PluginManagerPanel = defineAsyncComponent(() => import("./PluginManagerPanel.vue"));
const PluginManagerV2Panel = defineAsyncComponent(() => import("./PluginManagerV2Panel.vue"));
const PluginMarketV2Panel = defineAsyncComponent(() => import("./PluginMarketV2Panel.vue"));
const ExpertModelsPanel = defineAsyncComponent(() => import("../components/ExpertModels/ExpertModelsPanel.vue"));
const LocalModelManagerPanel = defineAsyncComponent(() => import("./LocalModelManagerPanel.vue"));
const PluginMarketPanel = defineAsyncComponent(() => import("./PluginMarketPanel.vue"));

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

const navFilterText = ref("");

function navLabel(id: SettingsNavAnyId): string {
  return String(t(`settings.nav.items.${settingsNavLabelKey(id)}`));
}

function navGroupHintText(id: SettingsNavAnyId): string {
  if (!id || typeof id !== "string" || !id.startsWith("settings.cat.")) return "";
  return String(t(`settings.nav.groupHints.${settingsNavLabelKey(id)}`));
}

const shortcutAccel = computed(() => ({
  s: formatChordModShift("S"),
  f: formatChordModShift("F"),
  a: formatChordModShift("A"),
  d: formatChordModShift("D"),
}));

function goToRoutineSettings(): void {
  selectNav(SETTINGS_NAV.generalOverview);
}

const showAdvancedSurfaceBanner = computed(
  () => roleStore.interactionImmersive && uiStore.settingsDeveloperMaster,
);

function filterSidebarByQuery(rows: SettingsNavRow[], q: string): SettingsNavRow[] {
  const needle = q.trim().toLowerCase();
  if (!needle) return rows;
  const hitIndex = new Set<number>();
  rows.forEach((row, i) => {
    if (row.isGroupLabel) return;
    if (navLabel(row.id).toLowerCase().includes(needle)) hitIndex.add(i);
  });
  const out: SettingsNavRow[] = [];
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i]!;
    if (row.isGroupLabel) {
      let any = false;
      for (let j = i + 1; j < rows.length; j++) {
        const n = rows[j]!;
        if (n.depth <= row.depth) break;
        if (hitIndex.has(j)) any = true;
      }
      if (any) out.push(row);
    } else if (hitIndex.has(i)) {
      out.push(row);
    }
  }
  return out;
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
/** 全局「恢复默认」后递增，用于强制重挂载依赖 onMounted 的子组件（云端表单、快捷键等） */
const hostPrefsReloadNonce = ref(0);
const globalResetBusy = ref(false);
const marketSourcesLoading = ref(false);
const marketDeveloperModeLocal = ref(false);
const marketSourcesText = ref("");
const marketSourcesLoaded = ref(false);

function stripEmptyNavGroups(rows: SettingsNavRow[]): SettingsNavRow[] {
  const out: SettingsNavRow[] = [];
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i]!;
    if (!row.isGroupLabel) {
      out.push(row);
      continue;
    }
    let any = false;
    for (let j = i + 1; j < rows.length; j++) {
      const n = rows[j]!;
      if (n.depth <= row.depth) break;
      if (!n.isGroupLabel) {
        any = true;
        break;
      }
    }
    if (any) out.push(row);
  }
  return out;
}

const visibleNavRows = computed(() => {
  const base = filterSettingsNavRows(roleStore.interactionImmersive, SETTINGS_NAV_ROWS);
  if (!roleStore.interactionImmersive || uiStore.settingsDeveloperMaster) {
    return stripEmptyNavGroups(base);
  }
  const gated = new Set(SETTINGS_DEVELOPER_GATED_NAV_IDS);
  const filtered = base.filter((row) => {
    if (row.isGroupLabel) return true;
    const id = row.id as SettingsNavId;
    if (!ALL_SETTINGS_NAV_IDS.includes(id)) return true;
    return !gated.has(id);
  });
  return stripEmptyNavGroups(filtered);
});

function firstSelectableFromRows(rows: SettingsNavRow[]): SettingsNavId {
  for (const r of rows) {
    if (!r.isGroupLabel && ALL_SETTINGS_NAV_IDS.includes(r.id as SettingsNavId)) {
      return r.id as SettingsNavId;
    }
  }
  return SETTINGS_NAV.generalOverview;
}

const sidebarNavRows = computed(() => filterSidebarByQuery(visibleNavRows.value, navFilterText.value));

/** 旧版深链 id（仍可能出现在外部脚本或历史会话中）→ 合并后的行为与偏好页 */
const LEGACY_SETTINGS_PENDING_NAV: Record<string, SettingsNavId> = {
  "settings.general.language": SETTINGS_NAV.generalBehavior,
  "settings.general.defaultModel": SETTINGS_NAV.generalBehavior,
};

function normalizePendingSettingsNavId(raw: string): SettingsNavId | null {
  const mapped = LEGACY_SETTINGS_PENDING_NAV[raw] ?? (raw as SettingsNavId);
  if (!ALL_SETTINGS_NAV_IDS.includes(mapped as SettingsNavId)) return null;
  return mapped as SettingsNavId;
}

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
      navFilterText.value = "";
      pluginStore.closeMarketPanel();
    } else {
      const pending = uiStore.consumeSettingsPendingNavId();
      const normalized = pending ? normalizePendingSettingsNavId(pending) : null;
      if (normalized) {
        selectedNavId.value = normalized;
      } else {
        selectedNavId.value = firstSelectableFromRows(visibleNavRows.value);
      }
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
  () => [roleStore.interactionImmersive, roleStore.interactionPureChat, visibleNavRows] as const,
  () => {
    const allowed = new Set(
      visibleNavRows.value.filter((r) => !r.isGroupLabel).map((r) => r.id as SettingsNavId),
    );
    if (!allowed.has(selectedNavId.value)) {
      selectedNavId.value = firstSelectableFromRows(visibleNavRows.value);
    }
  },
);

watch(
  () => uiStore.settingsDeveloperMaster,
  () => {
    if (!roleStore.interactionImmersive || uiStore.settingsDeveloperMaster) return;
    const gated = new Set(SETTINGS_DEVELOPER_GATED_NAV_IDS);
    if (gated.has(selectedNavId.value)) {
      selectedNavId.value = firstSelectableFromRows(visibleNavRows.value);
    }
  },
);

watch(
  () => [props.visible, selectedNavId.value] as const,
  ([vis, nav]) => {
    if (!vis || nav !== SETTINGS_NAV.dataExpertWorkbench) return;
    void expertModelsStore.refresh().catch(() => {});
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
    if (nav !== SETTINGS_NAV.marketBrowse && pluginStore.marketPanelEmbedHost === "settings") {
      pluginStore.closeMarketPanel();
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
    selectNav(SETTINGS_NAV.pluginsLinkBackends);
    await openPluginManagerEmbed("backends");
    showToast("info", String(t("settings.cloudLlmTrust.toastOpenedBackends")));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function onOpenExpertWorkbenchFromHub(o: { draftMode: ExpertWorkbenchDraftMode }): Promise<void> {
  try {
    pluginStore.expertWorkbenchDraftMode = o.draftMode;
    selectNav(SETTINGS_NAV.dataExpertWorkbench);
    await expertModelsStore.refresh();
    expertModelsStore.applyWorkbenchNavigationDraft(o.draftMode);
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

function onEmbeddedV2Close(): void {
  selectNav(SETTINGS_NAV.generalOverview);
}

async function onEmbeddedV2OpenV1(): Promise<void> {
  selectNav(SETTINGS_NAV.pluginsLinkInstalled);
  await openPluginManagerEmbed("plugins");
}

async function onEmbeddedV2OpenV1Backends(): Promise<void> {
  selectNav(SETTINGS_NAV.pluginsLinkBackends);
  await openPluginManagerEmbed("backends");
}

function onExpertModelsOpenPermissions(payload: { pluginId: string }): void {
  const pid = payload.pluginId.trim();
  if (!pid) return;
  selectNav(SETTINGS_NAV.pluginsLinkInstalled);
  void openPluginManagerEmbed("plugins");
  showToast("info", String(t("pluginManagerV1.ui.expertModels.permNavToast")));
}

async function onOpenMarketFromRoles(): Promise<void> {
  selectNav(SETTINGS_NAV.marketBrowse);
  await pluginStore.openMarketPanelInSettingsEmbed();
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

async function onGlobalResetDefaults(): Promise<void> {
  const title = String(t("settings.globalReset.confirmTitle"));
  const message = String(t("settings.globalReset.confirmMessage"));
  let ok = true;
  if (isTauriWebview()) {
    try {
      ok = await confirm(message, {
        title,
        type: "warning",
        okLabel: String(t("settings.globalReset.confirmOk")),
        cancelLabel: String(t("common.cancel")),
      });
    } catch {
      ok = window.confirm(`${title}\n\n${message}`);
    }
  } else {
    ok = window.confirm(`${title}\n\n${message}`);
  }
  if (!ok) return;
  globalResetBusy.value = true;
  try {
    await resetHostPreferencesToDefaults(roleStore.currentRoleId);
    uiStore.$patch({
      experimentalPluginManagerV2: false,
      languagePref: "system",
    });
    pluginStore.closePanel();
    await pluginStore.refresh();
    notifyHostModelsInventoryChanged();
    hostPrefsReloadNonce.value += 1;
    if (marketSourcesLoaded.value) await loadMarketSources();
    tierResetKey.value += 1;
    showToast("success", String(t("settings.globalReset.successToast")));
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    globalResetBusy.value = false;
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
              <label class="sv-nav-filter-wrap">
                <span class="sr-only">{{ t("settings.nav.filterLabel") }}</span>
                <input
                  v-model="navFilterText"
                  type="search"
                  class="sv-nav-filter"
                  :placeholder="String(t('settings.nav.filterPlaceholder'))"
                  autocomplete="off"
                  spellcheck="false"
                />
              </label>
              <template v-for="(row, idx) in sidebarNavRows" :key="`${row.id}-${idx}`">
                <div
                  v-if="row.isGroupLabel"
                  class="sv-tree-group"
                  :class="{ 'sv-tree-group--indented': row.depth === 1 }"
                >
                  <span class="sv-tree-group-label">{{ navLabel(row.id) }}</span>
                  <HelpHint
                    v-if="navGroupHintText(row.id)"
                    class="sv-tree-group-hint"
                    :text="navGroupHintText(row.id)"
                  />
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
              <div v-if="roleStore.interactionImmersive" class="sv-dev-top">
                <label class="sv-dev-toggle">
                  <input
                    type="checkbox"
                    :checked="uiStore.settingsDeveloperMaster === true"
                    @change="uiStore.setSettingsDeveloperMaster(($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ t("settings.centerDeveloperMaster.label") }}</span>
                </label>
                <p v-if="!uiStore.settingsDeveloperMaster" class="sv-muted sv-dev-hint">
                  {{ t("settings.centerDeveloperMaster.offHint") }}
                </p>
              </div>

              <div v-if="showAdvancedSurfaceBanner" class="sv-advanced-surface-banner">
                <p class="sv-advanced-surface-banner__text">{{ t("settings.advancedSurface.bannerLead") }}</p>
                <button type="button" class="sv-btn sv-btn--ghost sv-advanced-surface-banner__btn" @click="goToRoutineSettings">
                  {{ t("settings.advancedSurface.backToRoutine") }}
                </button>
              </div>

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
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.globalReset.lead") }}</p>
                  <p class="sv-muted sv-reset-scope">{{ t("settings.globalReset.scope") }}</p>
                  <button
                    type="button"
                    class="sv-btn sv-btn--danger"
                    :disabled="globalResetBusy"
                    @click="onGlobalResetDefaults"
                  >
                    {{ t("settings.globalReset.button") }}
                  </button>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.shortcutsManage" class="sv-pane-section">
                <SettingsTierSection tier="L2" :reset-key="tierResetKey">
                  <div class="sv-row-h">
                    <span class="sv-label">{{ t("settings.nav.items.shortcutsManage") }}</span>
                    <HelpHint :text="settingsShortcutsHelpHint()" />
                  </div>
                  <p class="sv-muted">{{ t("settings.shortcuts.immersiveHint") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorOpenSettings", { keys: shortcutAccel.s }) }}</p>
                  <ShortcutsManagerPanel
                    :key="`hk-${hostPrefsReloadNonce}`"
                    :bootstrap-epoch="pluginStore.bootstrapEpoch"
                  />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.generalBehavior" class="sv-pane-section sv-behavior-hub">
                <SettingsTierSection tier="L2" :reset-key="tierResetKey">
                  <p class="sv-muted sv-behavior-hub-lead">{{ t("settings.behaviorHub.pageLead") }}</p>
                  <div class="sv-settings-stack">
                    <section class="sv-settings-card" aria-labelledby="sv-bh-model-h">
                      <div class="sv-settings-card__head">
                        <h3 id="sv-bh-model-h" class="sv-settings-card__title">{{ t("settings.behaviorHub.cardModelTitle") }}</h3>
                        <p class="sv-settings-card__hint">{{ t("settings.behaviorHub.cardModelHint") }}</p>
                      </div>
                      <div class="sv-settings-card__body">
                        <ModelSelectorSettings
                          :active="selectedNavId === SETTINGS_NAV.generalBehavior && visible"
                          @open-local-models="selectNav(SETTINGS_NAV.modelsOllama)"
                          @configure-cloud="selectNav(SETTINGS_NAV.modelsCloud)"
                        />
                        <p class="sv-muted sv-cross-links">{{ t("settings.modelSelector.advancedLinksLead") }}</p>
                        <div class="sv-cross-link-row">
                          <button type="button" class="sv-btn sv-btn--ghost" @click="selectNav(SETTINGS_NAV.modelsCloud)">
                            {{ t("settings.modelSelector.linkCloud") }}
                          </button>
                          <button type="button" class="sv-btn sv-btn--ghost" @click="selectNav(SETTINGS_NAV.modelsOllama)">
                            {{ t("settings.modelSelector.linkLocal") }}
                          </button>
                        </div>
                      </div>
                    </section>

                    <section class="sv-settings-card" aria-labelledby="sv-bh-lang-h">
                      <div class="sv-settings-card__head">
                        <h3 id="sv-bh-lang-h" class="sv-settings-card__title">{{ t("settings.behaviorHub.cardLanguageTitle") }}</h3>
                        <p class="sv-settings-card__hint">{{ t("settings.behaviorHub.cardLanguageHint") }}</p>
                      </div>
                      <div class="sv-settings-card__body">
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
                      </div>
                    </section>

                    <section class="sv-settings-card" aria-labelledby="sv-bh-notify-h">
                      <div class="sv-settings-card__head">
                        <h3 id="sv-bh-notify-h" class="sv-settings-card__title">{{ t("settings.behaviorHub.cardNotificationsTitle") }}</h3>
                        <p class="sv-settings-card__hint">{{ t("settings.behaviorHub.cardNotificationsHint") }}</p>
                      </div>
                    </section>
                  </div>
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
                      <CloudLlmQuickSetup :key="`cloud-${hostPrefsReloadNonce}`" />
                    </div>
                    <div class="sv-cloud-actions-row">
                      <button type="button" class="sv-btn sv-btn--accent" @click="openCloudLlmTrustReadme">
                        {{ t("settings.cloudLlmTrust.reviewCta") }}
                      </button>
                      <button type="button" class="sv-btn" @click="onOpenPluginBackendsFromCloud">
                        {{ t("settings.cloudLlmTrust.openBackendsCta") }}
                      </button>
                      <button type="button" class="sv-btn sv-btn--ghost" @click="selectNav(SETTINGS_NAV.generalBehavior)">
                        {{ t("settings.nav.jumpDefaultModel") }}
                      </button>
                    </div>
                  </section>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.modelsOllama" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.modelsOllama") }}</p>
                  <p class="sv-muted">{{ t("settings.modelsOllama.downloadHint") }}</p>
                  <button type="button" class="sv-btn sv-btn--ghost sv-cross-link-top" @click="selectNav(SETTINGS_NAV.generalBehavior)">
                    {{ t("settings.nav.jumpDefaultModel") }}
                  </button>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <LocalModelManagerPanel
                    :visible="selectedNavId === SETTINGS_NAV.modelsOllama && visible"
                    embedded
                    @close="() => {}"
                  />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.dataExpertModels" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.dataExpertModels") }}</p>
                  <ExpertModelsSettingsHub
                    :active="selectedNavId === SETTINGS_NAV.dataExpertModels && visible"
                    @open-expert-workbench="onOpenExpertWorkbenchFromHub"
                  />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.dataExpertWorkbench" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.dataExpertWorkbench") }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <ExpertModelsPanel embedded @open-permissions="onExpertModelsOpenPermissions" />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.dataRoles" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <RoleManagerSettings
                    @switch-role="(id) => emit('switchRole', id)"
                    @open-market="onOpenMarketFromRoles"
                    @pack-imported="(id) => emit('packImported', id)"
                  />
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

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsLinkInstalled" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsInstalled") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorPluginManager", { keys: shortcutAccel.f }) }}</p>
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn secondary" @click="openPluginManagerEmbed('plugins')">
                      {{ t("settings.nav.cta.openPluginManagerInPage") }}
                    </button>
                  </div>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsLinkSlots" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsSlots") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorPluginManager", { keys: shortcutAccel.f }) }}</p>
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn secondary" @click="openPluginManagerEmbed('slots')">
                      {{ t("settings.nav.cta.openPluginManagerInPage") }}
                    </button>
                  </div>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsLinkBackends" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsBackends") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorPluginManager", { keys: shortcutAccel.f }) }}</p>
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn secondary" @click="openPluginManagerEmbed('backends')">
                      {{ t("settings.nav.cta.openPluginManagerInPage") }}
                    </button>
                  </div>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.pluginsV2Hub" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.pluginsV2Hub") }}</p>
                  <p v-if="!uiStore.experimentalPluginManagerV2" class="sv-muted">
                    {{ t("settings.nav.needV2Experimental") }}
                  </p>
                </SettingsTierSection>
                <SettingsTierSection
                  v-if="uiStore.experimentalPluginManagerV2"
                  tier="L4"
                  :reset-key="tierResetKey"
                >
                  <PluginManagerV2Panel
                    :visible="selectedNavId === SETTINGS_NAV.pluginsV2Hub && visible"
                    embedded
                    @close="onEmbeddedV2Close"
                    @open-v1="onEmbeddedV2OpenV1"
                    @open-v1-backends="onEmbeddedV2OpenV1Backends"
                  />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.marketBrowse" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.marketBrowse") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorPluginMarket", { keys: shortcutAccel.a }) }}</p>
                  <p
                    v-if="uiStore.experimentalPluginManagerV2 && uiStore.settingsDeveloperMaster"
                    class="sv-callout sv-muted"
                  >
                    {{ t("settings.nav.lead.marketBrowseV2NavHint") }}
                  </p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <button type="button" class="sv-btn sv-btn--accent" @click="pluginStore.openMarketPanelInSettingsEmbed()">
                    {{ unifiedMarketCtaText }}
                  </button>
                  <p class="sv-muted sv-foot">{{ t("settings.nav.embedMarketFoot") }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.marketBrowseV2" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.marketBrowseV2") }}</p>
                  <p v-if="!uiStore.experimentalPluginManagerV2" class="sv-muted">
                    {{ t("settings.nav.needV2Experimental") }}
                  </p>
                </SettingsTierSection>
                <SettingsTierSection
                  v-if="uiStore.experimentalPluginManagerV2"
                  tier="L4"
                  :reset-key="tierResetKey"
                >
                  <PluginMarketV2Panel
                    :visible="selectedNavId === SETTINGS_NAV.marketBrowseV2 && visible"
                    embedded
                    @close="onEmbeddedV2Close"
                  />
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
                      <button type="button" class="sv-v2-launch-btn" @click="selectNav(SETTINGS_NAV.pluginsV2Hub)">
                        {{ t("settings.nav.gotoV2HubButton") }}
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

              <div v-show="selectedNavId === SETTINGS_NAV.systemDeveloper" class="sv-pane-section">
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
                    <p class="sv-muted">{{ t("settings.plugins.devMode.pageLead") }}</p>
                    <label class="sv-toggle-row">
                      <input
                        type="checkbox"
                        :disabled="marketSourcesLoading"
                        :checked="marketDeveloperModeLocal === true"
                        @change="onToggleMarketDeveloperMode"
                      />
                      <span class="sv-toggle-text">
                        <strong>{{ t("settings.plugins.devMode.title") }}</strong>
                        <span class="sv-muted sv-toggle-desc">{{ t("settings.plugins.devMode.hint") }}</span>
                      </span>
                    </label>
                  </section>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <section v-if="marketDeveloperModeLocal" class="sv-section">
                    <div class="sv-dev-box">
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
                  <p v-else class="sv-muted">{{ t("settings.developerGate.offSources") }}</p>
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.diagnosticsDebug" class="sv-pane-section">
                <SettingsTierSection tier="L2" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.diagnosticsDebug") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorDiagnostics", { keys: shortcutAccel.d }) }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <SettingsDebugEmbed
                    :active="selectedNavId === SETTINGS_NAV.diagnosticsDebug && visible"
                    @imported="(id) => emit('packImported', id)"
                    @reload-policy="emit('reloadPolicy')"
                  />
                </SettingsTierSection>
              </div>

              <div v-show="selectedNavId === SETTINGS_NAV.diagnosticsAgent" class="sv-pane-section">
                <SettingsTierSection tier="L3" :reset-key="tierResetKey">
                  <p class="sv-muted">{{ t("settings.nav.lead.diagnosticsAgent") }}</p>
                  <p class="sv-muted sv-accel-hint">{{ t("settings.shortcuts.acceleratorDiagnostics", { keys: shortcutAccel.d }) }}</p>
                </SettingsTierSection>
                <SettingsTierSection tier="L4" :reset-key="tierResetKey">
                  <div class="sv-btn-row">
                    <button type="button" class="sv-btn sv-btn--accent" @click="openPluginManagerEmbed('backends')">
                      {{ unifiedOpenAgentDebugFromBackendsCta() }}
                    </button>
                  </div>
                  <p class="sv-muted sv-foot">{{ t("settings.nav.embedAgentDebugFoot") }}</p>
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
          <div
            v-if="pluginStore.marketPanelVisible && pluginStore.marketPanelEmbedHost === 'settings'"
            class="sv-pm-embed sv-market-embed"
          >
            <PluginMarketPanel embedded />
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
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-secondary);
  margin-top: 10px;
  margin-bottom: 2px;
  padding: 4px 8px 2px;
}
.sv-tree-group-label {
  flex: 1 1 auto;
  min-width: 0;
}
.sv-tree-group-hint {
  flex: 0 0 auto;
  text-transform: none;
  letter-spacing: normal;
  font-weight: 500;
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
.sv-dev-top {
  margin-bottom: 14px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sv-dev-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 650;
  cursor: pointer;
  user-select: none;
}
.sv-dev-hint {
  margin: 8px 0 0;
  font-size: 12px;
}
.sv-advanced-surface-banner {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px 14px;
  margin-bottom: 14px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--accent) 32%, var(--border-light));
  background: color-mix(in srgb, var(--accent) 7%, var(--bg-elevated));
}
.sv-advanced-surface-banner__text {
  margin: 0;
  flex: 1 1 200px;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.sv-advanced-surface-banner__btn {
  flex-shrink: 0;
  align-self: center;
}
.sv-accel-hint {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.45;
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
.sv-btn--danger {
  border-color: color-mix(in srgb, var(--text-danger, #c33) 42%, var(--border-light));
  color: var(--text-danger, #c33);
  background: color-mix(in srgb, var(--text-danger, #c33) 7%, var(--bg-primary));
}
.sv-btn--danger:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--text-danger, #c33) 58%, var(--border-light));
}
.sv-btn--ghost {
  font-size: 12px;
  padding: 5px 10px;
  background: transparent;
}
.sv-btn--ghost:hover {
  background: var(--bg-hover, rgba(255, 255, 255, 0.06));
}
.sv-cross-links {
  margin-top: 8px;
}
.sv-cross-link-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 2px;
}
.sv-cross-link-top {
  margin-top: 8px;
  align-self: flex-start;
}
.sv-reset-scope {
  white-space: pre-wrap;
  line-height: 1.45;
}
.sv-behavior-hub-lead {
  margin: 0 0 12px;
  line-height: 1.45;
}
.sv-settings-stack {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.sv-settings-card {
  padding: 12px 14px 14px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sv-settings-card__head {
  margin-bottom: 10px;
  padding-bottom: 8px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-light) 85%, transparent);
}
.sv-settings-card__title {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 650;
  color: var(--text-primary);
}
.sv-settings-card__hint {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.sv-settings-card__body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
