<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import LeftCategoryNav from "./LeftCategoryNav.vue";
import PluginCardList from "./PluginCardList.vue";
import RightDetailPanel from "./RightDetailPanel.vue";
import HelpCircle from "../HelpCircle.vue";
import ExpertModelsPanel from "../ExpertModels/ExpertModelsPanel.vue";
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
  if (!llm) return "未设置";
  if (llm === "directory") return dirId ? `directory · ${dirId}` : "directory · (未指定插件)";
  return llm;
});

async function onEnableLocalLlamaBasic(): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  const pid = localLlamaPluginIdDraft.value.trim();
  if (!roleId || !pid) return;
  if (!localLlamaPluginInstalled.value) {
    showToast("error", `未扫描到目录插件：${pid}`);
    return;
  }
  const declaredPerms = ["process:spawn", "network:*"];
  const ok = window.confirm(
    `启用本地 Llama（当前会话）将授予插件以下权限：\n${declaredPerms.map((p) => `- ${p}`).join("\n")}\n\n并把 LLM 后端切到 directory：${pid}\n\n继续吗？`,
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
    showToast("success", `已启用本地 Llama：${pid}（当前会话）`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onDisableSessionLlmOverride(): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  if (!roleId) return;
  const ok = window.confirm("将清除当前会话的 LLM 后端覆盖，恢复角色包/默认设置。继续吗？");
  if (!ok) return;
  try {
    const info = await setSessionPluginBackend(roleId, "llm", null);
    roleStore.applyRoleInfo(info);
    showToast("success", "已清除当前会话的 LLM 覆盖。");
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
        aria-label="插件权限"
        @click.self="closePermModal"
      >
        <div class="pm2-modal" @click.stop>
          <div class="pm2-modal-h">插件权限：{{ permPluginId }}</div>
          <p v-if="tokenInfoLoading" class="pm2-muted">加载权限说明中…</p>
          <div class="pm2-modal-actions">
            <button
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="onGrantAllDeclared"
            >
              一键授予声明权限
            </button>
            <button
              v-if="missingPermsFor(permPluginId).length"
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="onGrantMissingDeclared"
              :title="`补齐缺失：${missingPermsFor(permPluginId).join('、')}`"
            >
              补齐缺失
            </button>
            <button
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="() => refreshPerms(permPluginId)"
            >
              刷新
            </button>
            <button type="button" class="pm2-btn pm2-btn--sm" @click="closePermModal">
              关闭
            </button>
          </div>

          <div v-if="declaredPermsSorted.length > 0" class="pm2-perms-declared">
            <div class="pm2-perms-subh">声明（来自索引/安装元数据）</div>
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
          <p v-else-if="permLoading" class="pm2-muted">加载中…</p>
          <p v-else-if="permEffective.length === 0" class="pm2-muted">
            暂无权限信息（可能为旧版本安装，或该插件未声明任何权限）。
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
                <span v-if="p.declared !== true" class="pm2-perms-tag">额外</span>
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
            关闭权限后，对应能力会被宿主拒绝。部分变更可能需要重启插件进程生效。
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
    <div class="pm2-legend" aria-label="状态说明">
      <span class="pm2-legend-item is-enabled">已启用：当前配置可直接生效</span>
      <span class="pm2-legend-item is-pending">还需配置：通常缺少目录插件 ID</span>
      <span class="pm2-legend-item is-disabled">已关闭：当前链路未启用</span>
    </div>

    <section class="pm2-slotdash" aria-label="快速插槽配置">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">把插件放到界面里</h3>
          <HelpCircle label="这块是干什么的？" inline>
            <p>你只需要两步：先选“插槽”（插件要显示在哪），再勾选要显示的插件。</p>
            <p>如果某插件没有在 manifest 里声明这个插槽，这里不会出现它。</p>
          </HelpCircle>
        </div>
        <button type="button" class="pm2-btn" @click="onSaveSlotDashboard">保存</button>
      </div>

      <div class="pm2-slotdash-row">
        <label class="pm2-slotdash-label">
          插槽
          <select v-model="pickedSlot" class="pm2-select">
            <option v-for="s in supportedSlots" :key="s" :value="s">
              {{ slotLabel(s) }}
            </option>
          </select>
        </label>
        <div class="pm2-slotdash-muted">
          已启用 {{ enabledInPickedSlot.length }} / {{ candidatesForPickedSlot.length }}
        </div>
      </div>

      <div v-if="!pickedSlot" class="pm2-muted">未检测到可用插槽。</div>
      <div v-else class="pm2-slotdash-grid">
        <div class="pm2-slotdash-col">
          <div class="pm2-slotdash-colh">选择要显示的插件</div>
          <div v-if="candidatesForPickedSlot.length === 0" class="pm2-muted">
            这个插槽暂无可用插件（没有插件声明该插槽）。
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
                缺权限（{{ missingPermsFor(id).length }}）
              </span>
              <button type="button" class="pm2-mini" @click="openPermModal(id)">权限</button>
              <button
                v-if="missingPermsFor(id).length"
                type="button"
                class="pm2-mini warn"
                @click="openPermModal(id)"
              >
                一键修复
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
                    ? '当前插件已停用，点击启用'
                    : '当前插件已启用，点击停用'
                "
              >
                {{ pluginStore.isPluginDisabled(id) ? "已停用" : "已启用" }}
              </button>
            </li>
          </ul>
        </div>
        <div class="pm2-slotdash-col">
          <div class="pm2-slotdash-colh">显示顺序（从上到下）</div>
          <div v-if="enabledInPickedSlot.length === 0" class="pm2-muted">
            还没选择任何插件。
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
                  上移
                </button>
                <button type="button" class="pm2-mini" @click="moveInPickedSlot(id, 'down')">
                  下移
                </button>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </section>

    <section class="pm2-slotdash" aria-label="从 Git 安装插件">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">从 Git 仓库安装插件</h3>
          <HelpCircle label="什么是从 Git 安装？" inline>
            <p>适合从 GitHub/自建 Git 仓库直接拉取插件。</p>
            <p>网盘/压缩包请放到投放目录，再去“插件市场 → 本地导入”安装。</p>
          </HelpCircle>
        </div>
        <button
          type="button"
          class="pm2-btn"
          :disabled="gitInstalling || !gitUrlDraft.trim()"
          @click="onInstallFromGit"
        >
          {{ gitInstalling ? "安装中…" : "安装" }}
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
        提示：请仅安装你信任的来源；安装后如遇“权限不足”，到插件权限里补授权即可。
      </p>
    </section>

    <section class="pm2-slotdash" aria-label="本地 Llama（基础）">
      <div class="pm2-slotdash-head">
        <div class="pm2-slotdash-title">
          <h3 class="pm2-h3">本地 Llama（基础）</h3>
          <HelpCircle label="为什么要在这里设置？" inline>
            <p>这里提供“一键启用”的最小路径：授予必要权限，并把当前会话的 LLM 后端切到 directory。</p>
            <p>更复杂的模型/日志/调参工作台保留在 V1。</p>
          </HelpCircle>
        </div>
        <button type="button" class="pm2-btn secondary" @click="onDisableSessionLlmOverride">
          清除会话覆盖
        </button>
      </div>
      <div class="pm2-slotdash-row">
        <label class="pm2-slotdash-label" style="flex: 1 1 360px">
          插件 ID
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
        <div class="pm2-slotdash-muted">状态：{{ localLlamaPluginInstalled ? "已扫描" : "未扫描" }}</div>
        <button
          type="button"
          class="pm2-btn"
          :disabled="!localLlamaPluginInstalled"
          @click="onEnableLocalLlamaBasic"
        >
          一键启用（当前会话）
        </button>
      </div>
      <p class="pm2-muted" style="margin: 8px 0 0">当前有效 LLM：{{ llmEffectiveLabel }}</p>
    </section>

    <ExpertModelsPanel @open-permissions="openPermModal($event.pluginId)" />

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
  flex: 1;
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
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 248px minmax(0, 1fr) 300px;
  grid-template-rows: minmax(0, 1fr);
  gap: 12px;
  align-items: stretch;
}
.pm2-grid > * {
  min-height: 0;
}
@media (max-width: 1080px) {
  .pm2-grid {
    grid-template-columns: 1fr;
    grid-template-rows: none;
    grid-auto-rows: auto;
    flex: 1 1 auto;
    min-height: 0;
    overflow: auto;
  }
  .pm2-slotdash-grid {
    grid-template-columns: 1fr;
  }
}
</style>
