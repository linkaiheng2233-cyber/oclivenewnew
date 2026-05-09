<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import LeftCategoryNav from "./LeftCategoryNav.vue";
import PluginCardList from "./PluginCardList.vue";
import RightDetailPanel from "./RightDetailPanel.vue";
import HelpCircle from "../HelpCircle.vue";
import { usePluginManagerV2 } from "../../composables/usePluginManagerV2";
import { usePluginTerm } from "../../composables/usePluginTerm";
import {
  ALL_EMBEDDED_SLOT_NAMES,
  SLOT_CHAT_HEADER,
  SLOT_CHAT_TOOLBAR,
  SLOT_DEBUG_DOCK,
  SLOT_LAUNCHER_PALETTE,
  SLOT_OVERLAY_FLOATING,
  SLOT_ROLE_DETAIL,
  SLOT_SETTINGS_ADVANCED,
  SLOT_SETTINGS_PANEL,
  SLOT_SETTINGS_PLUGINS,
  SLOT_SIDEBAR,
  usePluginStore,
} from "../../stores/pluginStore";
import { useAppToast } from "../../composables/useAppToast";
import { useRoleStore } from "../../stores/roleStore";
import {
  getPluginPermissionGrants,
  installPluginFromGit,
  listPermissionTokens,
  setPluginPermissionGrant,
  setSessionPluginBackend,
  type PermissionTokenInfoDto,
  type PluginPermissionGrantDto,
} from "../../utils/tauri-api";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openV1: [];
  openV1Backends: [];
}>();

const {
  searchKeyword,
  selectedCategory,
  selectedCardId,
  categories,
  filteredCards,
  selectedCard,
  applyCardChange,
} = usePluginManagerV2();
const { term } = usePluginTerm();
const { showToast } = useAppToast();
const { t } = useI18n();
const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const busy = ref(false);
const rightCollapsed = ref(false);

const slotLabel = (slot: string): string => {
  if (slot === SLOT_SETTINGS_PANEL) return String(t("pluginManagerV2.slots.settingsPanel"));
  if (slot === SLOT_SETTINGS_PLUGINS) return String(t("pluginManagerV2.slots.settingsPlugins"));
  if (slot === SLOT_SETTINGS_ADVANCED) return String(t("pluginManagerV2.slots.settingsAdvanced"));
  if (slot === SLOT_SIDEBAR) return String(t("pluginManagerV2.slots.sidebar"));
  if (slot === SLOT_ROLE_DETAIL) return String(t("pluginManagerV2.slots.roleDetail"));
  if (slot === SLOT_CHAT_HEADER) return String(t("pluginManagerV2.slots.chatHeader"));
  if (slot === SLOT_CHAT_TOOLBAR) return String(t("pluginManagerV2.slots.chatToolbar"));
  if (slot === SLOT_OVERLAY_FLOATING) return String(t("pluginManagerV2.slots.overlayFloating"));
  if (slot === SLOT_LAUNCHER_PALETTE) return String(t("pluginManagerV2.slots.launcherPalette"));
  if (slot === SLOT_DEBUG_DOCK) return String(t("pluginManagerV2.slots.debugDock"));
  return slot;
};

const supportedSlots = computed(() => {
  const s = pluginStore.supportedUiSlots ?? [];
  if (s.length > 0) return s;
  return [...ALL_EMBEDDED_SLOT_NAMES];
});

const pickedSlot = ref<string>("");
watch(
  supportedSlots,
  (list) => {
    if (pickedSlot.value && list.includes(pickedSlot.value)) return;
    pickedSlot.value = list[0] ?? "";
  },
  { immediate: true },
);

const candidatesForPickedSlot = computed(() => {
  const slot = pickedSlot.value.trim();
  if (!slot) return [];
  return pluginStore.pluginsOrderedForSlot(slot);
});

const enabledInPickedSlot = computed(() => {
  const slot = pickedSlot.value.trim();
  if (!slot) return [];
  return candidatesForPickedSlot.value.filter(
    (id) => !pluginStore.isSlotContributionDisabled(slot, id),
  );
});

const missingPermsFor = (pluginId: string): string[] => {
  const entry = pluginStore.catalog.find((c) => c.id === pluginId);
  const declared = (entry?.installMeta?.declaredPermissions ?? []).map((x) => (x ?? "").trim()).filter(Boolean);
  const granted = (entry?.installMeta?.grantedPermissions ?? []).map((x) => (x ?? "").trim()).filter(Boolean);
  const grantedSet = new Set(granted);
  return declared.filter((p) => !grantedSet.has(p));
};

const permModalOpen = ref(false);
const permPluginId = ref<string>("");
const permLoading = ref(false);
const permError = ref<string | null>(null);
const permGrants = ref<PluginPermissionGrantDto[]>([]);
const tokenInfoLoading = ref(false);
const tokenInfoMap = ref<Map<string, PermissionTokenInfoDto>>(new Map());

const permSorted = computed(() =>
  [...(permGrants.value ?? [])].sort((a, b) =>
    a.permission === b.permission ? 0 : a.permission < b.permission ? -1 : 1,
  ),
);

const declaredPermsSorted = computed(() => {
  const pid = permPluginId.value.trim();
  const entry = pluginStore.catalog.find((c) => c.id === pid);
  const raw = entry?.installMeta?.declaredPermissions ?? [];
  return [...raw]
    .map((s) => (s ?? "").trim())
    .filter(Boolean)
    .sort((a, b) => (a === b ? 0 : a < b ? -1 : 1));
});

const permEffective = computed(() => {
  const declared = declaredPermsSorted.value;
  const g = permSorted.value ?? [];
  const enabledMap = new Map<string, boolean>();
  for (const x of g) {
    enabledMap.set(x.permission, x.enabled === true);
  }
  const tokens = new Set<string>();
  for (const p of declared) tokens.add(p);
  for (const x of g) tokens.add(x.permission);
  const all = [...tokens].sort((a, b) => (a === b ? 0 : a < b ? -1 : 1));
  return all.map((permission) => ({
    permission,
    enabled: enabledMap.get(permission) === true,
    declared: declared.includes(permission),
    info: tokenInfoMap.value.get(permission),
  }));
});

const riskLabel = (risk: string | undefined): string => {
  if (risk === "high") return String(t("pluginManagerV2.permissions.risk.high"));
  if (risk === "medium") return String(t("pluginManagerV2.permissions.risk.medium"));
  if (risk === "low") return String(t("pluginManagerV2.permissions.risk.low"));
  return String(t("pluginManagerV2.permissions.risk.unknown"));
};

const riskClass = (risk: string | undefined): string => {
  if (risk === "high") return "risk-high";
  if (risk === "medium") return "risk-medium";
  if (risk === "low") return "risk-low";
  return "risk-unknown";
};

async function refreshTokenInfos(): Promise<void> {
  tokenInfoLoading.value = true;
  try {
    const res = await listPermissionTokens();
    const map = new Map<string, PermissionTokenInfoDto>();
    for (const x of res.tokens ?? []) {
      if (!x?.token) continue;
      map.set(x.token, x);
    }
    tokenInfoMap.value = map;
  } finally {
    tokenInfoLoading.value = false;
  }
}

async function refreshPerms(pid: string): Promise<void> {
  const pluginId = pid.trim();
  if (!pluginId) return;
  permLoading.value = true;
  permError.value = null;
  try {
    const res = await getPluginPermissionGrants(pluginId);
    permGrants.value = res.grants ?? [];
  } catch (e) {
    permError.value = e instanceof Error ? e.message : String(e);
  } finally {
    permLoading.value = false;
  }
}

async function openPermModal(pid: string): Promise<void> {
  permPluginId.value = pid.trim();
  permModalOpen.value = true;
  if (tokenInfoMap.value.size === 0) void refreshTokenInfos();
  void refreshPerms(pid);
}

function closePermModal(): void {
  permModalOpen.value = false;
  permPluginId.value = "";
  permError.value = null;
  permGrants.value = [];
}

async function onTogglePermission(permission: string, enabled: boolean) {
  const pid = permPluginId.value.trim();
  if (!pid || !permission.trim()) return;
  permLoading.value = true;
  try {
    await setPluginPermissionGrant(pid, permission, enabled);
    await refreshPerms(pid);
    await pluginStore.refresh();
    showToast("success", String(t("pluginManagerV2.permissions.toastUpdated")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    permLoading.value = false;
  }
}

async function onGrantAllDeclared(): Promise<void> {
  const pid = permPluginId.value.trim();
  if (!pid) return;
  const declared = declaredPermsSorted.value;
  if (declared.length === 0) {
    showToast("info", String(t("pluginManagerV2.permissions.toastNoDeclared")));
    return;
  }
  const ok = window.confirm(String(t("pluginManagerV2.permissions.confirmGrantAll", { n: declared.length })));
  if (!ok) return;
  permLoading.value = true;
  try {
    for (const p of declared) {
      await setPluginPermissionGrant(pid, p, true);
    }
    await refreshPerms(pid);
    await pluginStore.refresh();
    showToast("success", String(t("pluginManagerV2.permissions.toastGrantedAll")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    permLoading.value = false;
  }
}

async function onGrantMissingDeclared(): Promise<void> {
  const pid = permPluginId.value.trim();
  if (!pid) return;
  const missing = missingPermsFor(pid);
  if (missing.length === 0) {
    showToast("success", String(t("pluginManagerV2.permissions.toastNoMissing")));
    return;
  }
  const ok = window.confirm(
    String(
      t("pluginManagerV2.permissions.confirmGrantMissing", {
        n: missing.length,
        list: missing.map((p) => `- ${p}`).join("\n"),
      }),
    ),
  );
  if (!ok) return;
  permLoading.value = true;
  try {
    for (const p of missing) {
      await setPluginPermissionGrant(pid, p, true);
    }
    await refreshPerms(pid);
    await pluginStore.refresh();
    showToast("success", String(t("pluginManagerV2.permissions.toastGrantedMissing")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    permLoading.value = false;
  }
}

function toggleSlotContribution(pluginId: string, enabled: boolean) {
  const slot = pickedSlot.value.trim();
  if (!slot) return;
  pluginStore.setSlotContributionDisabled(slot, pluginId, !enabled);
}

function moveInPickedSlot(pluginId: string, dir: "up" | "down") {
  const slot = pickedSlot.value.trim();
  if (!slot) return;
  const ids = pluginStore.pluginsOrderedForSlot(slot);
  const from = ids.indexOf(pluginId);
  if (from < 0) return;
  const to = dir === "up" ? from - 1 : from + 1;
  pluginStore.movePluginInSlotOrder(slot, from, to);
}

async function onSaveSlotDashboard(): Promise<void> {
  try {
    await pluginStore.persist();
    showToast("success", String(t("pluginManagerV2.slotDashboard.toastSaved")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const gitUrlDraft = ref<string>("");
const gitInstalling = ref(false);

async function onInstallFromGit(): Promise<void> {
  const gitUrl = gitUrlDraft.value.trim();
  if (!gitUrl) return;
  const ok = window.confirm(String(t("pluginManagerV2.gitInstall.confirm", { url: gitUrl })));
  if (!ok) return;
  gitInstalling.value = true;
  try {
    const r = await installPluginFromGit(gitUrl);
    showToast("success", String(t("pluginManagerV2.gitInstall.toastInstalled", { id: r.installedPluginId })));
    gitUrlDraft.value = "";
    await pluginStore.refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    gitInstalling.value = false;
  }
}

const localLlamaPluginIdDraft = ref<string>("com.oclive.llama.local");
const localLlamaSuggestedPlugins = computed(() => {
  const out = pluginStore.catalog
    .filter((p) => !p.isShell)
    .map((p) => {
      const provides = (p.provides ?? []).map((x) => String(x).toLowerCase());
      const score =
        (p.id.toLowerCase().includes("llama") ? 50 : 0) +
        (p.id.toLowerCase().includes("llm") ? 20 : 0) +
        (provides.some((x) => x.includes("llm")) ? 30 : 0) +
        (provides.some((x) => x.includes("directory")) ? 5 : 0);
      return { id: p.id, provides: p.provides ?? [], score };
    })
    .sort((a, b) => b.score - a.score || a.id.localeCompare(b.id));
  return out.slice(0, 30);
});
const localLlamaPluginInstalled = computed(() => {
  const pid = localLlamaPluginIdDraft.value.trim();
  if (!pid) return false;
  return !!pluginStore.catalog?.some((p) => p.id === pid);
});

const llmEffectiveLabel = computed(() => {
  const eff = roleStore.roleInfo.pluginBackendsEffective as any;
  const llm = String(eff?.llm ?? "").trim();
  const dp = eff?.directory_plugins as any;
  const dirId = String(dp?.llm ?? "").trim();
  if (!llm) return String(t("pluginManagerV2.localLlama.effective.notSet"));
  if (llm === "directory")
    return dirId
      ? String(t("pluginManagerV2.localLlama.effective.directoryWithId", { id: dirId }))
      : String(t("pluginManagerV2.localLlama.effective.directoryNoId"));
  return llm;
});

async function onEnableLocalLlamaBasic(): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  const pid = localLlamaPluginIdDraft.value.trim();
  if (!roleId || !pid) return;
  if (!localLlamaPluginInstalled.value) {
    showToast("error", String(t("pluginManagerV2.localLlama.toastNotScanned", { id: pid })));
    return;
  }
  const declaredPerms = ["process:spawn", "network:*"];
  const ok = window.confirm(
    String(
      t("pluginManagerV2.localLlama.confirmEnable", {
        list: declaredPerms.map((p) => `- ${p}`).join("\n"),
        id: pid,
      }),
    ),
  );
  if (!ok) return;
  try {
    for (const perm of declaredPerms) {
      await setPluginPermissionGrant(pid, perm, true);
    }
    const info = await setSessionPluginBackend(
      roleId,
      "llm",
      "directory",
      undefined,
      undefined,
      pid,
    );
    roleStore.applyRoleInfo(info);
    showToast("success", String(t("pluginManagerV2.localLlama.toastEnabled", { id: pid })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onDisableSessionLlmOverride(): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  if (!roleId) return;
  const ok = window.confirm(String(t("pluginManagerV2.localLlama.confirmClearSessionOverride")));
  if (!ok) return;
  try {
    const info = await setSessionPluginBackend(roleId, "llm", null);
    roleStore.applyRoleInfo(info);
    showToast("success", String(t("pluginManagerV2.localLlama.toastClearedOverride")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

onMounted(async () => {
  if (pluginStore.catalog.length > 0) return;
  try {
    await pluginStore.refresh();
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
});

watch(
  () => props.visible,
  (v) => {
    if (v) rightCollapsed.value = false;
  },
);

async function onApply(payload: Record<string, unknown>) {
  if (!selectedCard.value) return;
  busy.value = true;
  try {
    const msg = await applyCardChange(selectedCard.value, payload);
    showToast("success", msg);
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="pm2-root">
    <Teleport to="body">
      <div
        v-if="permModalOpen"
        class="pm2-modal-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="String(t('pluginManagerV2.permissions.dialogAria'))"
        @click.self="closePermModal"
      >
        <div class="pm2-modal" @click.stop>
          <div class="pm2-modal-h">{{ t("pluginManagerV2.permissions.title", { id: permPluginId }) }}</div>
          <p v-if="tokenInfoLoading" class="pm2-muted">{{ t("pluginManagerV2.permissions.loadingTokenInfo") }}</p>
          <div class="pm2-modal-actions">
            <button
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="onGrantAllDeclared"
            >
              {{ t("pluginManagerV2.permissions.grantAllDeclared") }}
            </button>
            <button
              v-if="missingPermsFor(permPluginId).length"
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="onGrantMissingDeclared"
              :title="
                t('pluginManagerV2.permissions.grantMissingTitle', {
                  missing: missingPermsFor(permPluginId).join('、'),
                })
              "
            >
              {{ t("pluginManagerV2.permissions.grantMissing") }}
            </button>
            <button
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="() => refreshPerms(permPluginId)"
            >
              {{ t("common.refresh") }}
            </button>
            <button type="button" class="pm2-btn pm2-btn--sm" @click="closePermModal">
              {{ t("common.close") }}
            </button>
          </div>

          <div v-if="declaredPermsSorted.length > 0" class="pm2-perms-declared">
            <div class="pm2-perms-subh">{{ t("pluginManagerV2.permissions.declaredTitle") }}</div>
            <ul class="pm2-perms-list">
              <li v-for="p in declaredPermsSorted" :key="p" class="pm2-perms-li">
                <span class="pm2-perms-token">{{ p }}</span>
                <span v-if="tokenInfoMap.get(p)?.title" class="pm2-muted">
                  · {{ tokenInfoMap.get(p)?.title }}
                </span>
              </li>
            </ul>
          </div>

          <p v-if="permError" class="pm2-err">{{ permError }}</p>
          <p v-else-if="permLoading" class="pm2-muted">{{ t("pluginManagerV2.permissions.loading") }}</p>
          <p v-else-if="permEffective.length === 0" class="pm2-muted">
            {{ t("pluginManagerV2.permissions.noPermInfo") }}
          </p>
          <ul v-else class="pm2-perms-list">
            <li v-for="p in permEffective" :key="p.permission" class="pm2-perms-li">
              <label class="pm2-perms-row">
                <input
                  type="checkbox"
                  :disabled="permLoading"
                  :checked="p.enabled === true"
                  @change="
                    onTogglePermission(
                      p.permission,
                      ($event.target as HTMLInputElement).checked,
                    )
                  "
                />
                <span class="pm2-perms-token">{{ p.permission }}</span>
                <span v-if="p.declared !== true" class="pm2-perms-tag">{{ t("pluginManagerV2.permissions.extraTag") }}</span>
                <span
                  v-if="p.info?.risk"
                  class="pm2-perms-risk"
                  :class="riskClass(p.info?.risk)"
                >
                  {{ riskLabel(p.info?.risk) }}
                </span>
              </label>
              <div v-if="p.info?.title || p.info?.description" class="pm2-perms-desc">
                <div v-if="p.info?.title" class="pm2-perms-title">{{ p.info?.title }}</div>
                <div v-if="p.info?.description" class="pm2-muted">
                  {{ p.info?.description }}
                </div>
              </div>
            </li>
          </ul>
          <p class="pm2-muted" style="margin: 8px 0 0">
            {{ t("pluginManagerV2.permissions.hint") }}
          </p>
        </div>
      </div>
    </Teleport>

    <header class="pm2-head">
      <div>
        <h2 class="pm2-title">{{ term("title.v2") }}</h2>
        <p class="pm2-sub">{{ term("subtitle.v2") }}</p>
      </div>
      <div class="pm2-actions">
        <button type="button" class="pm2-btn secondary" @click="emit('openV1')">
          {{ term("action.open_v1") }}
        </button>
        <button type="button" class="pm2-btn" @click="emit('close')">{{ term("action.close") }}</button>
      </div>
    </header>
    <div class="pm2-legend" :aria-label="String(t('pluginManagerV2.legend.aria'))">
      <span class="pm2-legend-item is-enabled">{{ t("pluginManagerV2.legend.enabled") }}</span>
      <span class="pm2-legend-item is-pending">{{ t("pluginManagerV2.legend.pending") }}</span>
      <span class="pm2-legend-item is-disabled">{{ t("pluginManagerV2.legend.disabled") }}</span>
    </div>

    <section class="pm2-slotdash" :aria-label="String(t('pluginManagerV2.slotDashboard.aria'))">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">{{ t("pluginManagerV2.slotDashboard.title") }}</h3>
          <HelpCircle :label="String(t('pluginManagerV2.slotDashboard.helpLabel'))" inline>
            <p>{{ t("pluginManagerV2.slotDashboard.helpLine1") }}</p>
            <p>{{ t("pluginManagerV2.slotDashboard.helpLine2") }}</p>
          </HelpCircle>
        </div>
        <button type="button" class="pm2-btn" @click="onSaveSlotDashboard">{{ t("common.save") }}</button>
      </div>

      <div class="pm2-slotdash-row">
        <label class="pm2-slotdash-label">
          {{ t("pluginManagerV2.slotDashboard.slotLabel") }}
          <select v-model="pickedSlot" class="pm2-select">
            <option v-for="s in supportedSlots" :key="s" :value="s">
              {{ slotLabel(s) }}
            </option>
          </select>
        </label>
        <div class="pm2-slotdash-muted">
          {{ t("pluginManagerV2.slotDashboard.enabledCount", { enabled: enabledInPickedSlot.length, total: candidatesForPickedSlot.length }) }}
        </div>
      </div>

      <div v-if="!pickedSlot" class="pm2-muted">{{ t("pluginManagerV2.slotDashboard.noSlots") }}</div>
      <div v-else class="pm2-slotdash-grid">
        <div class="pm2-slotdash-col">
          <div class="pm2-slotdash-colh">{{ t("pluginManagerV2.slotDashboard.pickPluginsTitle") }}</div>
          <div v-if="candidatesForPickedSlot.length === 0" class="pm2-muted">
            {{ t("pluginManagerV2.slotDashboard.noPluginsForSlot") }}
          </div>
          <ul v-else class="pm2-slotdash-list">
            <li
              v-for="id in candidatesForPickedSlot"
              :key="`${pickedSlot}-${id}`"
              class="pm2-slotdash-li"
            >
              <label class="pm2-slotdash-item">
                <input
                  type="checkbox"
                  :checked="!pluginStore.isSlotContributionDisabled(pickedSlot, id)"
                  @change="
                    toggleSlotContribution(
                      id,
                      ($event.target as HTMLInputElement).checked,
                    )
                  "
                />
                <span class="pm2-slotdash-id">{{ id }}</span>
              </label>
              <span v-if="missingPermsFor(id).length" class="pm2-warn-pill" :title="missingPermsFor(id).join('、')">
                {{ t("pluginManagerV2.slotDashboard.missingPerms", { n: missingPermsFor(id).length }) }}
              </span>
              <button type="button" class="pm2-mini" @click="openPermModal(id)">{{ t("pluginManagerV2.slotDashboard.permsBtn") }}</button>
              <button
                v-if="missingPermsFor(id).length"
                type="button"
                class="pm2-mini warn"
                @click="openPermModal(id)"
              >
                {{ t("pluginManagerV2.slotDashboard.fixPermsBtn") }}
              </button>
              <button
                type="button"
                class="pm2-mini"
                :class="{ warn: pluginStore.isPluginDisabled(id) }"
                @click="
                  pluginStore.setPluginDisabled(id, !pluginStore.isPluginDisabled(id))
                "
                :title="
                  pluginStore.isPluginDisabled(id)
                    ? t('pluginManagerV2.slotDashboard.toggleEnableTitle.enable')
                    : t('pluginManagerV2.slotDashboard.toggleEnableTitle.disable')
                "
              >
                {{ pluginStore.isPluginDisabled(id) ? t("pluginManagerV2.slotDashboard.disabled") : t("pluginManagerV2.slotDashboard.enabled") }}
              </button>
            </li>
          </ul>
        </div>
        <div class="pm2-slotdash-col">
          <div class="pm2-slotdash-colh">{{ t("pluginManagerV2.slotDashboard.orderTitle") }}</div>
          <div v-if="enabledInPickedSlot.length === 0" class="pm2-muted">
            {{ t("pluginManagerV2.slotDashboard.noPickedPlugins") }}
          </div>
          <ul v-else class="pm2-slotdash-order">
            <li
              v-for="id in enabledInPickedSlot"
              :key="`ord-${pickedSlot}-${id}`"
              class="pm2-slotdash-ordli"
            >
              <span class="pm2-slotdash-id">{{ id }}</span>
              <div class="pm2-slotdash-ordbtns">
                <button type="button" class="pm2-mini" @click="moveInPickedSlot(id, 'up')">
                  {{ t("pluginManagerV2.slotDashboard.moveUp") }}
                </button>
                <button type="button" class="pm2-mini" @click="moveInPickedSlot(id, 'down')">
                  {{ t("pluginManagerV2.slotDashboard.moveDown") }}
                </button>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </section>

    <section class="pm2-slotdash" :aria-label="String(t('pluginManagerV2.gitSection.aria'))">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">{{ t("pluginManagerV2.gitSection.title") }}</h3>
          <HelpCircle :label="String(t('pluginManagerV2.gitSection.helpLabel'))" inline>
            <p>{{ t("pluginManagerV2.gitSection.helpLine1") }}</p>
            <p>{{ t("pluginManagerV2.gitSection.helpLine2") }}</p>
          </HelpCircle>
        </div>
        <button
          type="button"
          class="pm2-btn"
          :disabled="gitInstalling || !gitUrlDraft.trim()"
          @click="onInstallFromGit"
        >
          {{ gitInstalling ? t("pluginManagerV2.gitSection.installing") : t("pluginManagerV2.gitSection.install") }}
        </button>
      </div>
      <div class="pm2-slotdash-row">
        <label class="pm2-slotdash-label" style="flex: 1 1 420px">
          Git URL
          <input
            v-model="gitUrlDraft"
            class="pm2-input"
            type="text"
            autocomplete="off"
            placeholder="https://github.com/owner/repo.git"
          />
        </label>
      </div>
      <p class="pm2-muted" style="margin: 8px 0 0">
        {{ t("pluginManagerV2.gitSection.hint") }}
      </p>
    </section>

    <section class="pm2-slotdash" :aria-label="String(t('pluginManagerV2.localLlamaSection.aria'))">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">{{ t("pluginManagerV2.localLlamaSection.title") }}</h3>
          <HelpCircle :label="String(t('pluginManagerV2.localLlamaSection.helpLabel'))" inline>
            <p>{{ t("pluginManagerV2.localLlamaSection.helpLine1") }}</p>
            <p>{{ t("pluginManagerV2.localLlamaSection.helpLine2") }}</p>
          </HelpCircle>
        </div>
        <button type="button" class="pm2-btn secondary" @click="onDisableSessionLlmOverride">
          {{ t("pluginManagerV2.localLlamaSection.clearOverride") }}
        </button>
      </div>
      <div class="pm2-slotdash-row">
        <label class="pm2-slotdash-label" style="flex: 1 1 360px">
          {{ t("pluginManagerV2.localLlamaSection.pluginIdLabel") }}
          <input
            v-model="localLlamaPluginIdDraft"
            class="pm2-input"
            type="text"
            autocomplete="off"
            list="pm2-llama-suggestions"
            placeholder="com.oclive.llama.local"
          />
        </label>
        <datalist id="pm2-llama-suggestions">
          <option
            v-for="p in localLlamaSuggestedPlugins"
            :key="p.id"
            :value="p.id"
          >
            {{ p.id }}{{ p.provides.length ? ` · provides: ${p.provides.join(", ")}` : "" }}
          </option>
        </datalist>
        <div class="pm2-slotdash-muted">
          {{ t("pluginManagerV2.localLlamaSection.statusLabel") }}：{{
            localLlamaPluginInstalled
              ? t("pluginManagerV2.localLlamaSection.status.scanned")
              : t("pluginManagerV2.localLlamaSection.status.notScanned")
          }}
        </div>
        <button
          type="button"
          class="pm2-btn"
          :disabled="!localLlamaPluginInstalled"
          @click="onEnableLocalLlamaBasic"
        >
          {{ t("pluginManagerV2.localLlamaSection.enableForSession") }}
        </button>
      </div>
      <p class="pm2-muted" style="margin: 8px 0 0">{{ t("pluginManagerV2.localLlamaSection.effectiveLabel") }}：{{ llmEffectiveLabel }}</p>
    </section>

    <section class="pm2-slotdash pm2-expert-classic" :aria-label="String(t('pluginManagerV2.expertModelsClassic.aria'))">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">{{ t("pluginManagerV2.expertModelsClassic.title") }}</h3>
          <HelpCircle :label="String(t('pluginManagerV2.expertModelsClassic.helpLabel'))" inline>
            <p>{{ t("pluginManagerV2.expertModelsClassic.helpLine1") }}</p>
            <p>{{ t("pluginManagerV2.expertModelsClassic.helpLine2") }}</p>
          </HelpCircle>
        </div>
        <button type="button" class="pm2-btn" @click="emit('openV1Backends')">
          {{ t("pluginManagerV2.expertModelsClassic.openClassic") }}
        </button>
      </div>
      <p class="pm2-muted" style="margin: 0">{{ t("pluginManagerV2.expertModelsClassic.hint") }}</p>
    </section>

    <div class="pm2-grid">
      <LeftCategoryNav v-model="selectedCategory" :categories="categories" />
      <PluginCardList
        :items="filteredCards"
        :selected-id="selectedCardId"
        :keyword="searchKeyword"
        @update:keyword="searchKeyword = $event"
        @select="selectedCardId = $event"
      />
      <RightDetailPanel
        :item="selectedCard"
        :collapsed="rightCollapsed"
        :busy="busy"
        @toggle="rightCollapsed = !rightCollapsed"
        @apply="onApply"
      />
    </div>
  </div>
</template>

<style scoped>
.pm2-root {
  flex: 0 0 auto;
  width: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.pm2-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}
.pm2-title {
  margin: 0 0 6px;
  font-size: 18px;
}
.pm2-sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm2-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.pm2-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px;
  border: 1px dashed var(--border-light);
  border-radius: 8px;
  background: var(--bg-elevated);
}
.pm2-legend-item {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 1.2;
}
.pm2-legend-item.is-enabled {
  background: color-mix(in srgb, #16a34a 16%, var(--bg-primary));
  color: color-mix(in srgb, #166534 80%, var(--text-primary));
}
.pm2-legend-item.is-pending {
  background: color-mix(in srgb, #f59e0b 20%, var(--bg-primary));
  color: color-mix(in srgb, #92400e 85%, var(--text-primary));
}
.pm2-legend-item.is-disabled {
  background: color-mix(in srgb, #64748b 18%, var(--bg-primary));
  color: var(--text-secondary);
}
.pm2-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.pm2-btn.secondary {
  background: transparent;
}
.pm2-select {
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
}
.pm2-input {
  width: min(720px, 100%);
  padding: 7px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}
.pm2-h3 {
  margin: 0;
  font-size: 16px;
}
.pm2-slotdash {
  padding: 12px 12px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.pm2-slotdash-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}
.pm2-slotdash-title {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pm2-slotdash-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 10px;
  flex-wrap: wrap;
}
.pm2-slotdash-label {
  display: flex;
  gap: 8px;
  align-items: center;
  color: var(--text-secondary);
  font-size: 13px;
}
.pm2-slotdash-muted {
  color: var(--text-secondary);
  font-size: 12px;
}
.pm2-slotdash-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
  margin-top: 10px;
}
.pm2-slotdash-col {
  min-width: 0;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  background: var(--bg-elevated);
  padding: 10px 10px;
}
.pm2-slotdash-colh {
  font-size: 13px;
  font-weight: 700;
  margin-bottom: 8px;
}
.pm2-slotdash-list,
.pm2-slotdash-order {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm2-slotdash-li,
.pm2-slotdash-ordli {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.pm2-slotdash-item {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.pm2-slotdash-id {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 520px;
}
.pm2-mini {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
}
.pm2-mini.warn {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
}
.pm2-warn-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--danger-600, #c0392b) 35%, var(--border-light));
  background: color-mix(in srgb, var(--danger-600, #c0392b) 10%, var(--bg-primary));
  color: var(--danger-600, #c0392b);
  font-size: 12px;
  font-weight: 700;
}
.pm2-slotdash-ordbtns {
  display: flex;
  gap: 6px;
}
.pm2-grid {
  flex: 0 0 auto;
  width: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: 248px minmax(0, 1fr) minmax(220px, 300px);
  grid-template-rows: none;
  gap: 12px;
  align-items: start;
}
.pm2-grid > * {
  min-height: 0;
}
@media (max-width: 1080px) {
  .pm2-grid {
    grid-template-columns: 1fr;
    grid-auto-rows: auto;
  }
  .pm2-slotdash-grid {
    grid-template-columns: 1fr;
  }
}
</style>
