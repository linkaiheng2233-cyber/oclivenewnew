<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { open as openExternal } from "@tauri-apps/api/shell";
import { computed, defineAsyncComponent, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import PluginBackendSessionPanel from "../components/PluginBackendSessionPanel.vue";
import ExpertModelsRuntimeCard from "../components/ExpertModelsRuntimeCard.vue";
import InstalledPluginWorkspaceDetail from "../components/InstalledPluginWorkspaceDetail.vue";
import PluginScaffoldWizard from "../components/PluginScaffoldWizard.vue";
import PmSlotRow from "../components/PmSlotRow.vue";
import PluginSlotEmbed from "../components/PluginSlotEmbed.vue";
import HelpCircle from "../components/HelpCircle.vue";
import { useAppToast } from "../composables/useAppToast";
import {
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
} from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import {
  applyAuthorSuggestedPluginBackends,
  packPlugin,
  getPluginPermissionGrants,
  setPluginPermissionGrant,
  previewPluginZipPermissions,
  previewPluginDirPermissions,
  installPluginDir,
  listPermissionTokens,
  previewProfileFromPath,
  getCachedPluginReviewsIndex,
  syncPluginReviewsIndex,
  listLocalImportCandidates,
  readLocalImportText,
  previewLocalPluginArchive,
  installLocalPluginArchive,
  setSessionPluginBackend,
  setSessionPluginBackendsOverride,
  type PluginMarketEntryDto,
  type PluginReviewEntryDto,
  type PermissionTokenInfoDto,
  type ProfilePreviewDto,
} from "../utils/tauri-api";
import { getPluginMarketSourcesConfig } from "../utils/tauri-api";
import { OFFICIAL_UI_SLOTS } from "../lib/shellCapabilities";
import {
  buildReviewJsonTemplate,
  getRecentReviews,
  renderReviewLine,
  type ReviewPreview,
} from "../lib/pluginReviewsUi";
import { useExpertModelsStore } from "../stores/expertModelsStore";

const ExpertModelsPanel = defineAsyncComponent(() => import("../components/ExpertModels/ExpertModelsPanel.vue"));

const pluginStore = usePluginStore();
const roleStore = useRoleStore();
const expertModelsStore = useExpertModelsStore();
const { showToast } = useAppToast();
const { t } = useI18n();

const marketSourceSelected = ref("official");
const marketSources = ref<string[]>([]);
type MarketEntryTab = "plugin" | "module" | "profile";
const marketEntryTab = ref<MarketEntryTab>("plugin");

const permConsentVisible = ref(false);
const permConsentTitle = ref("");
const permConsentPerms = ref<string[]>([]);
const permConsentSelected = ref<Record<string, boolean>>({});
const permConsentTrustSummary = ref<string>("");
let permConsentResolver: ((v: string[] | null) => void) | null = null;

const permTokenInfoLoading = ref(false);
const permTokenInfoMap = ref<Map<string, PermissionTokenInfoDto>>(new Map());

const preflightVisible = ref(false);
const preflightTitle = ref("");
const preflightLines = ref<string[]>([]);
let preflightResolver: ((v: boolean) => void) | null = null;

const profilePreviewLoading = ref(false);
const profilePreview = ref<ProfilePreviewDto | null>(null);
const profileApplyLoading = ref(false);

const pluginReviewsLoading = ref(false);
const pluginReviewsIndex = ref<{ reviews: PluginReviewEntryDto[] } | null>(null);
const pluginReviewsErr = ref("");

const localImportsLoading = ref(false);
const localImportsErr = ref("");
const localImportsRootDir = ref("");
const LOCAL_IMPORTS_HIDDEN_KEY = "oclive.local_imports.hidden_v1";
const localImportsHidden = ref<Record<string, boolean>>({});
const localImports = ref<
  Array<{
    kind: string;
    path: string;
    fileName: string;
    relatedSignaturePath?: string | null;
    sizeBytes?: number | null;
    modifiedMs?: number | null;
  }>
>([]);

function loadLocalImportsHidden(): void {
  try {
    const raw = localStorage.getItem(LOCAL_IMPORTS_HIDDEN_KEY);
    const obj = raw ? (JSON.parse(raw) as Record<string, boolean>) : {};
    localImportsHidden.value = obj && typeof obj === "object" ? obj : {};
  } catch {
    localImportsHidden.value = {};
  }
}

function persistLocalImportsHidden(): void {
  try {
    localStorage.setItem(
      LOCAL_IMPORTS_HIDDEN_KEY,
      JSON.stringify(localImportsHidden.value ?? {}),
    );
  } catch {
    /* ignore */
  }
}

function hideLocalImport(path: string): void {
  const p = path.trim();
  if (!p) return;
  localImportsHidden.value = { ...(localImportsHidden.value ?? {}), [p]: true };
  persistLocalImportsHidden();
}

function unhideAllLocalImports(): void {
  localImportsHidden.value = {};
  persistLocalImportsHidden();
}

const SESSION_OVERRIDE_ROLLBACK_KEY = "oclive.session_override.rollback_v1";
type SessionOverrideSnapshot = {
  roleId: string;
  savedAt: string;
  source: "module" | "profile" | "manual";
  label: string;
  /** null 表示当时无覆盖（回滚会清空 override） */
  override: Record<string, unknown> | null;
};

function readRollbackSnapshot(roleId: string): SessionOverrideSnapshot | null {
  try {
    const raw = localStorage.getItem(`${SESSION_OVERRIDE_ROLLBACK_KEY}.${roleId}`);
    if (!raw) return null;
    const j = JSON.parse(raw) as SessionOverrideSnapshot;
    if (!j || typeof j !== "object") return null;
    if ((j.roleId ?? "").trim() !== roleId.trim()) return null;
    return j;
  } catch {
    return null;
  }
}

function writeRollbackSnapshot(s: SessionOverrideSnapshot): void {
  try {
    localStorage.setItem(`${SESSION_OVERRIDE_ROLLBACK_KEY}.${s.roleId}`, JSON.stringify(s));
  } catch {
    /* ignore */
  }
}

function saveCurrentSessionOverrideForRollback(
  source: "module" | "profile" | "manual",
  label: string,
): void {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  if (!roleId) return;
  const cur = roleStore.roleInfo.pluginBackendsSessionOverride as any;
  const snapshot: SessionOverrideSnapshot = {
    roleId,
    savedAt: new Date().toISOString(),
    source,
    label: label.trim() || "(unknown)",
    override: cur && typeof cur === "object" ? (cur as Record<string, unknown>) : null,
  };
  writeRollbackSnapshot(snapshot);
}

const localLlamaPluginIdDraft = ref<string>("com.oclive.llama.local");

const localLlamaPluginInstalled = computed(() => {
  const pid = localLlamaPluginIdDraft.value.trim();
  if (!pid) return false;
  return !!pluginStore.catalog?.some((p) => p.id === pid);
});

async function onEnableLocalLlamaDirectory(): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  const pid = localLlamaPluginIdDraft.value.trim();
  if (!roleId || !pid) return;
  if (!localLlamaPluginInstalled.value) {
    showToast("error", String(t("pluginManagerV1.llama.toastNotFound", { id: pid })));
    return;
  }
  const declaredPerms = ["process:spawn", "network:*"];
  const accepted = await requestPermissionConsentWithTrust(
    String(t("pluginManagerV1.llama.permConsentTitle")),
    declaredPerms,
    String(t("pluginManagerV1.llama.permConsentTrustSummary")),
  );
  if (!accepted) return;
  if (hasHighRiskPermission(accepted)) {
    const ok = window.confirm(
      String(t("pluginManagerV1.permissions.confirmHighRisk", { list: accepted.map((p) => `- ${p}`).join("\n") })),
    );
    if (!ok) return;
  }
  const planLines = [
    String(t("pluginManagerV1.llama.plan.writeSessionOverride")),
    `- llm = directory`,
    `- directory_plugins.llm = ${pid}`,
    String(t("pluginManagerV1.llama.plan.writePermGrants")),
    ...accepted.map((p) => `- ${p}`),
  ];
  const ok2 = await requestApplyPreflight(String(t("pluginManagerV1.llama.preflightTitle")), planLines);
  if (!ok2) return;
  saveCurrentSessionOverrideForRollback("manual", "local-llama");
  try {
    for (const perm of accepted) {
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
    showToast("success", String(t("pluginManagerV1.llama.toastEnabled", { id: pid })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const rollbackSnapshotForRole = computed(() => {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  if (!roleId) return null;
  return readRollbackSnapshot(roleId);
});

async function rollbackLastSessionOverride(): Promise<void> {
  const roleId = (roleStore.currentRoleId ?? "").trim();
  const snap = rollbackSnapshotForRole.value;
  if (!roleId || !snap) return;
  const ok = window.confirm(
    String(t("pluginManagerV1.sessionOverride.confirmRollback", { source: snap.source, label: snap.label, savedAt: snap.savedAt })),
  );
  if (!ok) return;
  try {
    const next = snap.override ?? {};
    const info = await setSessionPluginBackendsOverride(roleId, next);
    roleStore.applyRoleInfo(info);
    showToast("success", String(t("pluginManagerV1.sessionOverride.toastRolledBack")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const PLUGIN_REVIEWS_REPO_URL =
  "https://github.com/linkaiheng2233-cyber/awesome-oclive-plugin-reviews";
const PLUGIN_REVIEWS_CONTRIBUTING_URL = `${PLUGIN_REVIEWS_REPO_URL}/blob/main/CONTRIBUTING.md`;

async function copyReviewTemplate(params: {
  pluginId: string;
  pubkeyId?: string | null;
  version?: string | null;
}): Promise<void> {
  const text = buildReviewJsonTemplate(params);
  try {
    if (!navigator.clipboard?.writeText) throw new Error("clipboard API unavailable");
    await navigator.clipboard.writeText(text);
    showToast("success", String(t("pluginManagerV1.reviews.toastCopiedTemplate")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function refreshPluginReviewsIndex(): Promise<void> {
  pluginReviewsLoading.value = true;
  pluginReviewsErr.value = "";
  try {
    pluginReviewsIndex.value = await getCachedPluginReviewsIndex();
  } catch (e) {
    pluginReviewsIndex.value = null;
    pluginReviewsErr.value = e instanceof Error ? e.message : String(e);
  } finally {
    pluginReviewsLoading.value = false;
  }
}

async function syncPluginReviewsIndexNow(): Promise<void> {
  pluginReviewsLoading.value = true;
  pluginReviewsErr.value = "";
  try {
    pluginReviewsIndex.value = await syncPluginReviewsIndex(null);
  } catch (e) {
    pluginReviewsIndex.value = null;
    pluginReviewsErr.value = e instanceof Error ? e.message : String(e);
  } finally {
    pluginReviewsLoading.value = false;
  }
}

type RatingAgg = { avg: number; count: number };

const ratingAggByPluginId = computed(() => {
  const map = new Map<string, RatingAgg>();
  const reviews = pluginReviewsIndex.value?.reviews ?? [];
  const acc = new Map<string, { sum: number; count: number }>();
  for (const r of reviews) {
    const pid = (r.plugin_id ?? "").trim();
    if (!pid) continue;
    const rating = Number(r.rating);
    if (!Number.isFinite(rating)) continue;
    const rr = Math.max(1, Math.min(5, Math.round(rating)));
    const cur = acc.get(pid) ?? { sum: 0, count: 0 };
    cur.sum += rr;
    cur.count += 1;
    acc.set(pid, cur);
  }
  for (const [pid, x] of acc.entries()) {
    map.set(pid, { avg: x.sum / Math.max(1, x.count), count: x.count });
  }
  return map;
});

function reviewsAggKey(pluginId: string, pubkeyId?: string | null): string {
  const pid = pluginId.trim();
  const pk = (pubkeyId ?? "").trim();
  return pk ? `${pid}@@${pk}` : `${pid}@@*`;
}

const ratingAggByPluginIdPubkey = computed(() => {
  const map = new Map<string, RatingAgg>();
  const reviews = pluginReviewsIndex.value?.reviews ?? [];
  const acc = new Map<string, { sum: number; count: number }>();
  for (const r of reviews) {
    const pid = (r.plugin_id ?? "").trim();
    if (!pid) continue;
    const key = reviewsAggKey(pid, r.pubkey_id ?? null);
    const rating = Number(r.rating);
    if (!Number.isFinite(rating)) continue;
    const rr = Math.max(1, Math.min(5, Math.round(rating)));
    const cur = acc.get(key) ?? { sum: 0, count: 0 };
    cur.sum += rr;
    cur.count += 1;
    acc.set(key, cur);
  }
  for (const [k, x] of acc.entries()) {
    map.set(k, { avg: x.sum / Math.max(1, x.count), count: x.count });
  }
  return map;
});

function ratingTextForPluginId(pluginId: string): string {
  const a = ratingAggByPluginId.value.get(pluginId.trim());
  if (!a) return String(t("pluginManagerV1.reviews.none"));
  return String(t("pluginManagerV1.reviews.summary", { avg: a.avg.toFixed(1), count: a.count }));
}

function ratingTextForPluginPubkey(pluginId: string, pubkeyId: string): string {
  const key = reviewsAggKey(pluginId, pubkeyId);
  const a = ratingAggByPluginIdPubkey.value.get(key);
  if (!a) return String(t("pluginManagerV1.reviews.none"));
  return String(t("pluginManagerV1.reviews.summary", { avg: a.avg.toFixed(1), count: a.count }));
}

function ratingStars(avg: number): string {
  const n = Math.max(0, Math.min(5, Math.round(avg)));
  return "★★★★★".slice(0, n) + "☆☆☆☆☆".slice(0, 5 - n);
}

function ratingStarsForPluginId(pluginId: string): string {
  const a = ratingAggByPluginId.value.get(pluginId.trim());
  if (!a) return "☆☆☆☆☆";
  return ratingStars(a.avg);
}

function ratingStarsForPluginPubkey(pluginId: string, pubkeyId: string): string {
  const key = reviewsAggKey(pluginId, pubkeyId);
  const a = ratingAggByPluginIdPubkey.value.get(key);
  if (!a) return "☆☆☆☆☆";
  return ratingStars(a.avg);
}

async function openPluginReviewsContribution(): Promise<void> {
  try {
    await openExternal(PLUGIN_REVIEWS_CONTRIBUTING_URL);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function refreshPermissionTokenInfos(): Promise<void> {
  permTokenInfoLoading.value = true;
  try {
    const res = await listPermissionTokens();
    const map = new Map<string, PermissionTokenInfoDto>();
    for (const x of res.tokens ?? []) {
      if (!x?.token) continue;
      map.set(x.token, x);
    }
    permTokenInfoMap.value = map;
  } finally {
    permTokenInfoLoading.value = false;
  }
}

function permRiskOf(token: string): string | undefined {
  return permTokenInfoMap.value.get(token)?.risk;
}

function riskLabel(risk: string | undefined): string {
  if (risk === "high") return String(t("pluginManagerV1.permissions.risk.high"));
  if (risk === "medium") return String(t("pluginManagerV1.permissions.risk.medium"));
  if (risk === "low") return String(t("pluginManagerV1.permissions.risk.low"));
  return String(t("pluginManagerV1.permissions.risk.unknown"));
}

function riskClass(risk: string | undefined): string {
  if (risk === "high") return "risk-high";
  if (risk === "medium") return "risk-medium";
  if (risk === "low") return "risk-low";
  return "risk-unknown";
}

function calcPermRisk(perms: string[]) {
  const hasNetwork = perms.some((p) => p === "network:*" || p.startsWith("network:"));
  const hasFs = perms.some((p) => p.startsWith("filesystem:"));
  const hasShell = perms.some((p) => p.startsWith("shell:") || p === "process:spawn");
  const hasRpcInvoke = perms.includes("rpc:invoke");
  return { hasNetwork, hasFs, hasShell, hasRpcInvoke };
}

function hasHighRiskPermission(perms: string[]): boolean {
  // Prefer registry when available; fall back to heuristic for forward-compat tokens.
  if (permTokenInfoMap.value.size > 0) {
    return perms.some((p) => permRiskOf(p) === "high");
  }
  const { hasNetwork, hasFs, hasShell, hasRpcInvoke } = calcPermRisk(perms);
  return (hasNetwork && hasFs && hasShell) || hasRpcInvoke;
}

async function requestPermissionConsent(
  title: string,
  declaredPerms: string[],
): Promise<string[] | null> {
  const perms = [...declaredPerms].map((s) => s.trim()).filter(Boolean);
  if (perms.length === 0) return [];
  permConsentTitle.value = title;
  permConsentPerms.value = perms;
  permConsentTrustSummary.value = "";
  const next: Record<string, boolean> = {};
  for (const p of perms) next[p] = true;
  permConsentSelected.value = next;
  permConsentVisible.value = true;
  return await new Promise<string[] | null>((resolve) => {
    permConsentResolver = resolve;
  });
}

async function requestPermissionConsentWithTrust(
  title: string,
  declaredPerms: string[],
  trustSummary: string,
): Promise<string[] | null> {
  const res = await requestPermissionConsent(title, declaredPerms);
  permConsentTrustSummary.value = trustSummary.trim();
  return res;
}

async function requestApplyPreflight(
  title: string,
  lines: string[],
): Promise<boolean> {
  preflightTitle.value = title;
  preflightLines.value = lines.map((s) => s.trim()).filter(Boolean);
  preflightVisible.value = true;
  return await new Promise<boolean>((resolve) => {
    preflightResolver = resolve;
  });
}

function summarizeOverrideBackends(obj: Record<string, unknown> | null | undefined): string[] {
  const o = obj ?? {};
  const keys = Object.keys(o);
  if (keys.length === 0) return [];
  const out: string[] = [];
  const topKeys = keys
    .filter((k) => k !== "directory_plugins")
    .sort();
  for (const k of topKeys) {
    const v = (o as any)[k];
    if (v === null || v === undefined || String(v).trim() === "") continue;
    out.push(`${k} = ${String(v)}`);
  }
  const dp = (o as any).directory_plugins;
  if (dp && typeof dp === "object") {
    const dpk = Object.keys(dp).sort();
    for (const k of dpk) {
      const v = (dp as any)[k];
      if (v === null || v === undefined || String(v).trim() === "") continue;
      out.push(`directory_plugins.${k} = ${String(v)}`);
    }
  }
  return out;
}

async function onPickProfilePreview(): Promise<void> {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Profile", extensions: ["json"] }],
  });
  if (!selected || typeof selected !== "string") return;
  profilePreviewLoading.value = true;
  try {
    profilePreview.value = await previewProfileFromPath(selected);
    showToast(
      "success",
      String(t("pluginManagerV1.profile.toastLoaded", { name: profilePreview.value.name })),
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    profilePreviewLoading.value = false;
  }
}

function normalizeProfileSource(s: string | null | undefined): string {
  const t = (s ?? "").trim();
  return t ? t : "official";
}

async function syncMarketSource(source: string): Promise<void> {
  marketSourceSelected.value = source;
  try {
    await pluginStore.syncPluginMarket(source === "official" ? null : source);
    if (pluginStore.pluginMarketSnapshot?.warning) {
      showToast("info", pluginStore.pluginMarketSnapshot.warning);
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast(
      "error",
      String(t("pluginManagerV1.marketSync.toastFailed", { source, msg })),
    );
    throw e;
  }
}

async function applyProfileBackends(p: ProfilePreviewDto): Promise<void> {
  const roleId = roleStore.currentRoleId;
  if (!roleId?.trim()) return;
  const b = p.backends ?? null;
  if (!b) return;
  const pairs: Array<[string, string | null | undefined]> = [
    ["memory", b.memory],
    ["emotion", b.emotion],
    ["event", b.event],
    ["prompt", b.prompt],
    ["llm", b.llm],
    ["agent", b.agent],
    ["complex_emotion", b.complexEmotion],
  ];
  for (const [module, backend] of pairs) {
    if (backend === undefined) continue;
    await setSessionPluginBackend(roleId, module as any, backend ?? null);
  }
}

async function onApplyProfile(): Promise<void> {
  const p = profilePreview.value;
  if (!p) return;
  if (profileApplyLoading.value) return;

  profileApplyLoading.value = true;
  try {
    const all = p.plugins ?? [];
    if (all.length === 0) {
      showToast("info", String(t("pluginManagerV1.profile.toastNoPlugins")));
    } else {
      const sources = [...new Set(all.map((x) => normalizeProfileSource(x.source)))];
      for (const s of sources) {
        await syncMarketSource(s);
        for (const spec of all.filter((x) => normalizeProfileSource(x.source) === s)) {
          const pid = (spec.id ?? "").trim();
          if (!pid) continue;
          const row = pluginStore.pluginMarketSnapshot?.plugins?.find((r) => r.id === pid);
          if (!row) {
            showToast("error", String(t("pluginManagerV1.profile.toastMarketMissingPlugin", { id: pid, source: s })));
            continue;
          }
          if (spec.version?.trim()) {
            marketPickedVersion.value = {
              ...marketPickedVersion.value,
              [pid]: spec.version.trim(),
            };
            await onInstallMarketVersion(row);
          } else {
            await onInstallMarketEntry(row);
          }
        }
      }
    }
    await applyProfileBackends(p);
    showToast("success", String(t("pluginManagerV1.profile.toastApplied")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    profileApplyLoading.value = false;
  }
}

function marketEntryType(row: PluginMarketEntryDto): string {
  return (row as any).type ?? "plugin";
}

const marketRowsFiltered = computed(() => {
  const rows = pluginStore.pluginMarketSnapshot?.plugins ?? [];
  const tab = marketEntryTab.value;
  return rows.filter((r) => {
    const t = marketEntryType(r);
    if (tab === "plugin") return t === "plugin" || !t;
    if (tab === "module") return t === "module";
    if (tab === "profile") return t === "profile";
    return true;
  });
});

const marketPageSize = ref<number>(30);
const marketPage = ref<number>(1);

const marketTotalPages = computed(() => {
  const total = marketRowsFiltered.value.length;
  const size = Math.max(1, Math.floor(marketPageSize.value || 30));
  return Math.max(1, Math.ceil(total / size));
});

const marketRowsPaged = computed(() => {
  const size = Math.max(1, Math.floor(marketPageSize.value || 30));
  const totalPages = marketTotalPages.value;
  const page = Math.min(Math.max(1, Math.floor(marketPage.value || 1)), totalPages);
  const start = (page - 1) * size;
  return marketRowsFiltered.value.slice(start, start + size);
});

watch([marketEntryTab, marketSourceSelected], () => {
  marketPage.value = 1;
});

const moduleRowsAll = computed(() => {
  const rows = pluginStore.pluginMarketSnapshot?.plugins ?? [];
  return rows.filter((r) => marketEntryType(r) === "module");
});

const profileRowsAll = computed(() => {
  const rows = pluginStore.pluginMarketSnapshot?.plugins ?? [];
  return rows.filter((r) => marketEntryType(r) === "profile");
});

async function onApplyModuleEntry(row: PluginMarketEntryDto): Promise<void> {
  const mod = (row as any).module as
    | { plugins: { id: string; version?: string | null; source?: string | null }[]; backends?: Record<string, unknown> | null }
    | null
    | undefined;
  if (!mod) {
    showToast("error", String(t("pluginManagerV1.modules.toastMissingBody")));
    return;
  }
  const planLines: string[] = [];
  const deps = (mod.plugins ?? []).map((x) => x.id).filter(Boolean);
  const sources = [
    ...new Set((mod.plugins ?? []).map((x) => normalizeProfileSource(x.source ?? null))),
  ];
  planLines.push(String(t("pluginManagerV1.applyPlan.type.module")));
  planLines.push(String(t("pluginManagerV1.applyPlan.entry", { id: row.id })));
  if (sources.length) planLines.push(String(t("pluginManagerV1.applyPlan.willSyncSources", { list: sources.join("、") })));
  if (deps.length) planLines.push(String(t("pluginManagerV1.applyPlan.willInstallDeps", { list: deps.join("、") })));
  const changes = summarizeOverrideBackends(mod.backends ?? null);
  if (changes.length) {
    planLines.push(String(t("pluginManagerV1.applyPlan.willWriteSessionOverride")));
    for (const x of changes) planLines.push(`- ${x}`);
  }
  const ok = await requestApplyPreflight(
    String(t("pluginManagerV1.applyPlan.titleModule", { id: row.id })),
    planLines,
  );
  if (!ok) return;
  saveCurrentSessionOverrideForRollback("module", row.id);

  const list = mod.plugins ?? [];
  if (list.length === 0) {
    showToast("info", String(t("pluginManagerV1.modules.toastNoDeps")));
  }
  for (const spec of list) {
    const pid = (spec.id ?? "").trim();
    if (!pid) continue;
    const src = normalizeProfileSource(spec.source ?? null);
    await syncMarketSource(src);
    const prow = pluginStore.pluginMarketSnapshot?.plugins?.find((r) => r.id === pid);
    if (!prow) {
      showToast(
        "error",
        String(t("pluginManagerV1.applyPlan.depNotFound", { id: pid, source: src })),
      );
      continue;
    }
    if ((spec.version ?? "").trim()) {
      marketPickedVersion.value = {
        ...marketPickedVersion.value,
        [pid]: (spec.version ?? "").trim(),
      };
      await onInstallMarketVersion(prow);
    } else {
      await onInstallMarketEntry(prow);
    }
  }
  if (mod.backends && Object.keys(mod.backends).length > 0) {
    const roleId = roleStore.currentRoleId;
    if (!roleId?.trim()) return;
    const info = await setSessionPluginBackendsOverride(roleId, mod.backends);
    roleStore.applyRoleInfo(info);
  } else {
    // ensure UI reflects cleared override if module applies no backends
    // (no-op here; install flow may still change plugin state)
  }
  showToast("success", String(t("pluginManagerV1.modules.toastApplied", { id: row.id })));
}

async function onApplyProfileEntry(row: PluginMarketEntryDto): Promise<void> {
  const prof = (row as any).profile as
    | {
        plugins: { id: string; version?: string | null; source?: string | null }[];
        backends?: Record<string, unknown> | null;
        predeclaredPermissions?: string[] | null;
      }
    | null
    | undefined;
  if (!prof) {
    showToast("error", String(t("pluginManagerV1.profiles.toastMissingBody")));
    return;
  }
  const planLines: string[] = [];
  const deps = (prof.plugins ?? []).map((x) => x.id).filter(Boolean);
  const sources = [
    ...new Set((prof.plugins ?? []).map((x) => normalizeProfileSource(x.source ?? null))),
  ];
  planLines.push(String(t("pluginManagerV1.applyPlan.type.profile")));
  planLines.push(String(t("pluginManagerV1.applyPlan.entry", { id: row.id })));
  if (sources.length) planLines.push(String(t("pluginManagerV1.applyPlan.willSyncSources", { list: sources.join("、") })));
  if (deps.length) planLines.push(String(t("pluginManagerV1.applyPlan.willInstallDeps", { list: deps.join("、") })));
  const changes = summarizeOverrideBackends(prof.backends ?? null);
  if (changes.length) {
    planLines.push(String(t("pluginManagerV1.applyPlan.willWriteSessionOverride")));
    for (const x of changes) planLines.push(`- ${x}`);
  }
  const ok = await requestApplyPreflight(
    String(t("pluginManagerV1.applyPlan.titleProfile", { id: row.id })),
    planLines,
  );
  if (!ok) return;
  saveCurrentSessionOverrideForRollback("profile", row.id);

  // Profile 本身无代码：权限风险来自依赖插件；这里的 predeclaredPermissions 仅做提示。
  const pre = (prof.predeclaredPermissions ?? []).map((s) => String(s).trim()).filter(Boolean);
  if (pre.length > 0) {
    showToast(
      "info",
      String(t("pluginManagerV1.profiles.toastPredeclaredPerms", { list: pre.join("、") })),
    );
  }
  const list = prof.plugins ?? [];
  for (const spec of list) {
    const pid = (spec.id ?? "").trim();
    if (!pid) continue;
    const src = normalizeProfileSource(spec.source ?? null);
    await syncMarketSource(src);
    const prow = pluginStore.pluginMarketSnapshot?.plugins?.find((r) => r.id === pid);
    if (!prow) {
      showToast(
        "error",
        String(t("pluginManagerV1.applyPlan.depNotFound", { id: pid, source: src })),
      );
      continue;
    }
    if ((spec.version ?? "").trim()) {
      marketPickedVersion.value = {
        ...marketPickedVersion.value,
        [pid]: (spec.version ?? "").trim(),
      };
      await onInstallMarketVersion(prow);
    } else {
      await onInstallMarketEntry(prow);
    }
  }
  if (prof.backends && Object.keys(prof.backends).length > 0) {
    const roleId = roleStore.currentRoleId;
    if (!roleId?.trim()) return;
    const info = await setSessionPluginBackendsOverride(roleId, prof.backends);
    roleStore.applyRoleInfo(info);
  }
  showToast("success", String(t("pluginManagerV1.profiles.toastApplied", { id: row.id })));
}

function onPermConsentCancel() {
  permConsentVisible.value = false;
  const r = permConsentResolver;
  permConsentResolver = null;
  permConsentTrustSummary.value = "";
  r?.(null);
}

function onPermConsentConfirm() {
  const selected = Object.entries(permConsentSelected.value)
    .filter(([, v]) => v)
    .map(([k]) => k);
  permConsentVisible.value = false;
  const r = permConsentResolver;
  permConsentResolver = null;
  permConsentTrustSummary.value = "";
  r?.(selected);
}

function onPreflightCancel() {
  preflightVisible.value = false;
  const r = preflightResolver;
  preflightResolver = null;
  r?.(false);
}

function onPreflightConfirm() {
  preflightVisible.value = false;
  const r = preflightResolver;
  preflightResolver = null;
  r?.(true);
}

function setPermConsentAll(v: boolean) {
  const next: Record<string, boolean> = {};
  for (const p of permConsentPerms.value) next[p] = v;
  permConsentSelected.value = next;
}

async function loadMarketSourcesForPanel(): Promise<void> {
  try {
    const cfg = await getPluginMarketSourcesConfig();
    marketSources.value = (cfg.pluginIndexSources ?? []).filter((x) => !!x?.trim());
    // 非开发者模式时仅允许官方
    if (cfg.developerMode !== true) {
      marketSourceSelected.value = "official";
    } else if (
      marketSourceSelected.value !== "official" &&
      !marketSources.value.includes(marketSourceSelected.value)
    ) {
      marketSourceSelected.value = marketSources.value[0] ?? "official";
    }
  } catch {
    // ignore; fall back to official
    marketSources.value = [];
    marketSourceSelected.value = "official";
  }
}

watch(
  () => pluginStore.panelVisible,
  (vis) => {
    if (vis) {
      loadLocalImportsHidden();
      void pluginStore.loadCachedPluginMarket();
      void refreshPermissionTokenInfos();
      void refreshPluginReviewsIndex();
      void loadMarketSourcesForPanel();
      void refreshLocalImports();
    }
  },
);

async function refreshLocalImports(): Promise<void> {
  localImportsLoading.value = true;
  localImportsErr.value = "";
  try {
    const r = await listLocalImportCandidates();
    localImportsRootDir.value = r.rootDir ?? "";
    const hidden = localImportsHidden.value ?? {};
    localImports.value = (r.items ?? []).filter((x) => !hidden[x.path]);
  } catch (e) {
    localImportsRootDir.value = "";
    localImports.value = [];
    localImportsErr.value = e instanceof Error ? e.message : String(e);
  } finally {
    localImportsLoading.value = false;
  }
}

function localImportKindLabel(k: string): string {
  if (k === "role_pack") return String(t("pluginManagerV1.localImports.kindLabels.rolePack"));
  if (k === "plugin_archive") return String(t("pluginManagerV1.localImports.kindLabels.pluginArchive"));
  if (k === "plugin_dir") return String(t("pluginManagerV1.localImports.kindLabels.pluginDir"));
  if (k === "module_json") return String(t("pluginManagerV1.localImports.kindLabels.moduleEntry"));
  if (k === "profile_json") return String(t("pluginManagerV1.localImports.kindLabels.profileEntry"));
  return k;
}

function localImportsByKind(kind: string) {
  return localImports.value.filter((x) => x.kind === kind);
}

async function onImportRolePackFromLocal(path: string): Promise<void> {
  try {
    const peek = await peekRolePack(path);
    const ok = window.confirm(
      String(
        t("pluginManagerV1.localImports.confirmImportRolePack", {
          name: peek.name,
          id: peek.id,
          version: peek.version,
        }),
      ),
    );
    if (!ok) return;
    const roleId = await importRolePack(path, false);
    showToast("success", String(t("pluginManagerV1.localImports.toastRolePackImported", { id: roleId })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onImportRolePackFromLocalOverwrite(path: string): Promise<void> {
  try {
    const peek = await peekRolePack(path);
    const ok = window.confirm(
      String(
        t("pluginManagerV1.localImports.confirmOverwriteRolePack", {
          name: peek.name,
          id: peek.id,
          version: peek.version,
        }),
      ),
    );
    if (!ok) return;
    const roleId = await importRolePack(path, true);
    showToast("success", String(t("pluginManagerV1.localImports.toastRolePackOverwritten", { id: roleId })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallPluginArchiveFromLocal(zipPath: string): Promise<void> {
  try {
    const isOclivePlugin = zipPath.toLowerCase().endsWith(".oclive-plugin");
    if (isOclivePlugin) {
      const it = localImports.value.find((x) => x.path === zipPath);
      const prev = await previewLocalPluginArchive({
        archivePath: zipPath,
        signaturePath: it?.relatedSignaturePath ?? null,
      });
      const trust =
        String(
          t("pluginManagerV1.localImports.offlineTrustSummary", {
            signature: prev.signatureVerified
              ? String(t("pluginManagerV1.localImports.signature.verified"))
              : prev.signatureMessage
                ? String(t("pluginManagerV1.localImports.signature.message", { msg: prev.signatureMessage }))
                : String(t("pluginManagerV1.localImports.signature.unknown")),
          }),
        );
      const accepted = await requestPermissionConsentWithTrust(
        String(t("pluginManagerV1.localImports.permTitleOfflinePackage", { id: prev.pluginId })),
        prev.declaredPermissions,
        trust,
      );
      if (accepted === null) return;
      if (hasHighRiskPermission(accepted)) {
        const ok = window.confirm(
          String(
            t("pluginManagerV1.localImports.confirmHighRiskPerms", {
              list: accepted.join("\n"),
            }),
          ),
        );
        if (!ok) return;
      }
      const overwrite = window.confirm(
        String(t("pluginManagerV1.localImports.confirmOverwritePlugin", { id: prev.pluginId })),
      );
      const pid = await installLocalPluginArchive({
        archivePath: zipPath,
        signaturePath: it?.relatedSignaturePath ?? null,
        overwrite,
        acceptedPermissions: accepted,
      });
      showToast("success", String(t("pluginManagerV1.localImports.toastInstalled", { id: pid })));
      await pluginStore.refresh();
      return;
    }

    // zip sideload
    const prev = await previewPluginZipPermissions(zipPath);
    const trust = String(t("pluginManagerV1.localImports.sideloadTrustSummary"));
    const accepted = await requestPermissionConsentWithTrust(
      String(t("pluginManagerV1.localImports.permTitleZip", { id: prev.pluginId })),
      prev.permissions,
      trust,
    );
    if (accepted === null) return;
    if (hasHighRiskPermission(accepted)) {
      const ok = window.confirm(
        String(
          t("pluginManagerV1.localImports.confirmHighRiskPerms", {
            list: accepted.join("\n"),
          }),
        ),
      );
      if (!ok) return;
    }
    await extractPluginZip(zipPath, prev.pluginId, accepted);
    showToast("success", String(t("pluginManagerV1.localImports.toastInstalled", { id: prev.pluginId })));
    await pluginStore.refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallPluginDirFromLocal(dirPath: string): Promise<void> {
  try {
    const prev = await previewPluginDirPermissions(dirPath);
    const trust = String(t("pluginManagerV1.localImports.sideloadTrustSummary"));
    const accepted = await requestPermissionConsentWithTrust(
      String(t("pluginManagerV1.localImports.permTitleDir", { id: prev.pluginId })),
      prev.permissions,
      trust,
    );
    if (accepted === null) return;
    if (hasHighRiskPermission(accepted)) {
      const ok = window.confirm(
        String(
          t("pluginManagerV1.localImports.confirmHighRiskPerms", {
            list: accepted.join("\n"),
          }),
        ),
      );
      if (!ok) return;
    }
    await installPluginDir(dirPath, prev.pluginId, accepted);
    showToast("success", String(t("pluginManagerV1.localImports.toastInstalled", { id: prev.pluginId })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onPreviewLocalJson(path: string): Promise<void> {
  try {
    const text = await readLocalImportText(path);
    await navigator.clipboard.writeText(text);
    showToast("success", String(t("pluginManagerV1.localImports.toastJsonCopied")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

function parseLocalMarketEntryJson(text: string): PluginMarketEntryDto {
  let j: unknown;
  try {
    j = JSON.parse(text) as unknown;
  } catch (e) {
    throw new Error(
      String(t("pluginManagerV1.localImports.jsonParseFailed", { msg: e instanceof Error ? e.message : String(e) })),
    );
  }
  if (!j || typeof j !== "object") {
    throw new Error(String(t("pluginManagerV1.localImports.jsonMustBeObject")));
  }
  const o = j as any;
  const t = String(o.type ?? "").trim();
  if (t !== "module" && t !== "profile") {
    throw new Error(String(t("pluginManagerV1.localImports.entryTypeMustBeModuleOrProfile")));
  }
  const id = String(o.id ?? "").trim();
  const name = String(o.name ?? "").trim();
  const version = String(o.version ?? "").trim();
  if (!id || !name || !version) {
    throw new Error(String(t("pluginManagerV1.localImports.entryMissingIdNameVersion")));
  }
  if (t === "module") {
    if (!o.module || typeof o.module !== "object") {
      throw new Error(String(t("pluginManagerV1.localImports.moduleMustHaveModuleObject")));
    }
    if (!Array.isArray(o.module.plugins)) {
      throw new Error(String(t("pluginManagerV1.localImports.modulePluginsMustBeArray")));
    }
  }
  if (t === "profile") {
    if (!o.profile || typeof o.profile !== "object") {
      throw new Error(String(t("pluginManagerV1.localImports.profileMustHaveProfileObject")));
    }
    if (!Array.isArray(o.profile.plugins)) {
      throw new Error(String(t("pluginManagerV1.localImports.profilePluginsMustBeArray")));
    }
  }
  return o as PluginMarketEntryDto;
}

async function onApplyLocalModuleOrProfile(path: string): Promise<void> {
  try {
    const text = await readLocalImportText(path);
    const row = parseLocalMarketEntryJson(text);
    if (marketEntryType(row) === "module") {
      await onApplyModuleEntry(row);
    } else if (marketEntryType(row) === "profile") {
      await onApplyProfileEntry(row);
    } else {
      showToast("error", String(t("pluginManagerV1.localImports.toastOnlyModuleOrProfile")));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const batchMode = ref(false);
const batchSelected = ref<Record<string, boolean>>({});
const scaffoldWizardVisible = ref(false);
const pluginPackStatus = ref("");
/** 已安装区：侧栏当前选中（右侧单一配置 + 调试台） */
const selectedWorkspacePluginId = ref("");
const expertFacilitySectionRef = ref<HTMLElement | null>(null);

const selectedWorkspacePlugin = computed(() =>
  pluginStore.catalog.find((c) => c.id === selectedWorkspacePluginId.value) ?? null,
);

const supportedUiSlotsForShell = computed(() => pluginStore.supportedUiSlots ?? []);
const unsupportedOfficialUiSlots = computed(() => {
  const supported = new Set(supportedUiSlotsForShell.value);
  return OFFICIAL_UI_SLOTS.filter((s) => !supported.has(s));
});

function selectWorkspacePlugin(id: string): void {
  selectedWorkspacePluginId.value = id;
}

function clearBatchSelection(): void {
  batchSelected.value = {};
}

watch(batchMode, (v) => {
  if (!v) {
    clearBatchSelection();
  }
});

watch(
  () => pluginStore.catalog.map((c) => c.id),
  () => {
    const next: Record<string, boolean> = {};
    for (const p of pluginStore.catalog) {
      if (batchSelected.value[p.id]) {
        next[p.id] = true;
      }
    }
    batchSelected.value = next;

    const ids = pluginStore.catalog.map((c) => c.id);
    if (ids.length === 0) {
      selectedWorkspacePluginId.value = "";
      return;
    }
    if (
      !selectedWorkspacePluginId.value ||
      !ids.includes(selectedWorkspacePluginId.value)
    ) {
      selectedWorkspacePluginId.value = ids[0] ?? "";
    }
  },
  { immediate: true },
);

const batchSelectedCount = computed(
  () => Object.values(batchSelected.value).filter(Boolean).length,
);

const batchSelectedIds = computed(() =>
  Object.entries(batchSelected.value)
    .filter(([, v]) => v)
    .map(([k]) => k),
);

function setBatchSelected(id: string, v: boolean): void {
  batchSelected.value = { ...batchSelected.value, [id]: v };
}

async function onBatchEnable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    pluginStore.batchEnablePluginIds(ids);
    showToast("success", String(t("pluginManagerV1.batch.toastEnabled", { n: ids.length })));
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onBatchDisable() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  pluginStore.batchDisablePluginIds(ids);
  showToast("success", String(t("pluginManagerV1.batch.toastDisabled", { n: ids.length })));
  clearBatchSelection();
}

async function onBatchUpdate() {
  const ids = batchSelectedIds.value;
  if (ids.length === 0) {
    return;
  }
  try {
    await pluginStore.batchUpdatePluginsFromGitIndex(ids);
    showToast("success", String(t("pluginManagerV1.batch.toastGitUpdated")));
    clearBatchSelection();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onSyncMarketIndex() {
  try {
    await pluginStore.syncPluginMarket(
      marketSourceSelected.value === "official" ? null : marketSourceSelected.value,
    );
    // Sync public reviews index best-effort; do not block plugin index success.
    void syncPluginReviewsIndexNow();
    if (pluginStore.pluginMarketSnapshot?.warning) {
      showToast("info", pluginStore.pluginMarketSnapshot.warning);
    } else {
      showToast("success", String(t("pluginManagerV1.marketSync.toastOk")));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallMarketEntry(row: PluginMarketEntryDto) {
  if ((row.missingDependencies ?? []).length > 0) {
    showToast(
      "error",
      String(
        t("pluginManagerV1.marketInstall.toastMissingDeps", {
          list: row.missingDependencies.join("、"),
        }),
      ),
    );
    return;
  }
  const declaredPerms = (row.permissions ?? []).map((s) => s.trim()).filter(Boolean);
  const trust = [
    row.source
      ? String(t("pluginManagerV1.communityIndex.trustLine.source", { v: row.source }))
      : "",
    row.publisher
      ? String(t("pluginManagerV1.communityIndex.trustLine.publisher", { v: row.publisher }))
      : "",
    (row.publicKeys ?? []).length
      ? String(
          t("pluginManagerV1.communityIndex.trustLine.pubkeys", {
            v: (row.publicKeys ?? [])
              .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
              .join("，"),
          }),
        )
      : "",
  ]
    .filter(Boolean)
    .join("\n");
  const accepted = await requestPermissionConsentWithTrust(
    String(t("pluginManagerV1.marketInstall.permTitleInstall", { id: row.id })),
    declaredPerms,
    trust,
  );
  if (accepted == null) return;
  if (hasHighRiskPermission(accepted)) {
    const ok2 = window.confirm(String(t("pluginManagerV1.marketInstall.confirmHighRisk")));
    if (!ok2) return;
  }
  try {
    // 默认安装走索引内版本解析（git tag clone）；仅开发者模式才应允许自定义 gitUrl 覆盖
    await pluginStore.installFromPluginMarket(row.id, null, accepted);
    showToast("success", String(t("pluginManagerV1.marketInstall.toastInstalledRecommendedRestart", { id: row.id })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const marketPickedVersion = ref<Record<string, string>>({});

function marketVersionsForRow(row: PluginMarketEntryDto): string[] {
  const vs = (row.versions ?? []).map((x) => x.version).filter((x) => !!x?.trim());
  // 降序展示：优先 semver 解析失败时按字符串倒序
  return [...vs].sort((a, b) => (a === b ? 0 : a < b ? 1 : -1));
}

function marketPickedVersionForRow(row: PluginMarketEntryDto): string {
  const pid = row.id;
  const picked = marketPickedVersion.value[pid]?.trim();
  if (picked) return picked;
  const vs = marketVersionsForRow(row);
  return vs[0] ?? row.version;
}

async function onInstallMarketVersion(row: PluginMarketEntryDto) {
  const v = marketPickedVersionForRow(row);
  if (!v?.trim()) return;
  const declaredPerms = (row.permissions ?? []).map((s) => s.trim()).filter(Boolean);
  const trust = [
    row.source
      ? String(t("pluginManagerV1.communityIndex.trustLine.source", { v: row.source }))
      : "",
    row.publisher
      ? String(t("pluginManagerV1.communityIndex.trustLine.publisher", { v: row.publisher }))
      : "",
    (row.publicKeys ?? []).length
      ? String(
          t("pluginManagerV1.communityIndex.trustLine.pubkeys", {
            v: (row.publicKeys ?? [])
              .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
              .join("，"),
          }),
        )
      : "",
  ]
    .filter(Boolean)
    .join("\n");
  const accepted = await requestPermissionConsentWithTrust(
    String(t("pluginManagerV1.marketInstall.permTitleInstallVersion", { id: row.id, version: v })),
    declaredPerms,
    trust,
  );
  if (accepted == null) return;
  if (hasHighRiskPermission(accepted)) {
    const ok2 = window.confirm(String(t("pluginManagerV1.marketInstall.confirmHighRiskVersion", { version: v })));
    if (!ok2) return;
  }
  try {
    await pluginStore.installVersionFromPluginMarket(row.id, v, accepted);
    showToast(
      "success",
      String(
        row.installed
          ? t("pluginManagerV1.marketInstall.toastRolledBackOrSwitched", {
              id: row.id,
              version: v,
            })
          : t("pluginManagerV1.marketInstall.toastInstalledVersion", {
              id: row.id,
              version: v,
            }),
      ),
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdateMarketEntry(row: PluginMarketEntryDto) {
  try {
    await pluginStore.updateInstalledPluginFromGit(row.id);
    showToast("success", String(t("pluginManagerV1.marketInstall.toastUpdated", { id: row.id })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onGitPullWorkspacePlugin() {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? "";
  if (!pid) return;
  try {
    await pluginStore.updateInstalledPluginFromGit(pid);
    showToast("success", String(t("pluginManagerV1.installed.toastGitPulled")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

const toolbarOrder = computed(() => pluginStore.toolbarPluginsOrdered());
const settingsPanelOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PANEL),
);
const roleDetailOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_ROLE_DETAIL));
const sidebarOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_SIDEBAR));
const chatHeaderOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_CHAT_HEADER));
const settingsPluginsOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_PLUGINS),
);
const settingsAdvancedOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_SETTINGS_ADVANCED),
);
const overlayFloatingOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_OVERLAY_FLOATING),
);
const launcherPaletteOrder = computed(() =>
  pluginStore.pluginsOrderedForSlot(SLOT_LAUNCHER_PALETTE),
);
const debugDockOrder = computed(() => pluginStore.pluginsOrderedForSlot(SLOT_DEBUG_DOCK));

let dragSlot: { slot: string; index: number } | null = null;

function onDragSlotStart(slot: string, index: number) {
  dragSlot = { slot, index };
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

function onDropSlot(slot: string, index: number) {
  if (!dragSlot || dragSlot.slot !== slot) {
    dragSlot = null;
    return;
  }
  if (dragSlot.index === index) {
    dragSlot = null;
    return;
  }
  pluginStore.movePluginInSlotOrder(slot, dragSlot.index, index);
  dragSlot = null;
}

async function onSave() {
  try {
    await pluginStore.persist();
    showToast("success", String(t("pluginManagerV1.save.toastSaved")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onResetToPackDefault() {
  try {
    if (pluginStore.persistScope === "global") {
      pluginStore.setPersistScope("role");
    }
    await pluginStore.resetToRolePackDefault();
    showToast(
      "success",
      String(t("pluginManagerV1.ui.toasts.resetToPackDefaultOk")),
    );
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onApplyAuthorSuggestedBackends() {
  try {
    const info = await applyAuthorSuggestedPluginBackends(roleStore.currentRoleId);
    roleStore.applyRoleInfo(info);
    showToast("success", String(t("pluginManagerV1.author.toastAppliedSuggestedBackends")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onCheckUpdates() {
  try {
    await pluginStore.checkPluginUpdatesFromRegistry();
    if (pluginStore.error) {
      showToast("error", pluginStore.error);
    } else {
      showToast("success", String(t("pluginManagerV1.installed.toastCheckUpdatesDone")));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onUpdateFromZip(pluginId: string) {
  const path = await open({
    multiple: false,
    filters: [{ name: "Zip", extensions: ["zip"] }],
  });
  if (path === null || Array.isArray(path)) {
    return;
  }
  try {
    const preview = await previewPluginZipPermissions(path);
    if (preview.pluginId.trim() !== pluginId.trim()) {
      showToast(
        "error",
        String(t("pluginManagerV1.installed.toastZipIdMismatch", { zipId: preview.pluginId, targetId: pluginId })),
      );
      return;
    }
    const accepted = await requestPermissionConsentWithTrust(
      String(t("pluginManagerV1.installed.permTitleSideloadUpdate", { id: pluginId })),
      preview.permissions ?? [],
      String(t("pluginManagerV1.installed.sideloadSourceLocalZip")),
    );
    if (accepted == null) return;
    if (hasHighRiskPermission(accepted)) {
      const ok2 = window.confirm(
        String(t("pluginManagerV1.installed.confirmSideloadHighRiskUpdate")),
      );
      if (!ok2) return;
    }
    await pluginStore.installPluginFromLocalZip(pluginId, path, accepted);
    showToast("success", String(t("pluginManagerV1.installed.toastZipUpdated")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onPackSelectedPlugin(): Promise<void> {
  const pid = selectedWorkspacePlugin.value?.id?.trim() ?? "";
  if (!pid) {
    pluginPackStatus.value = String(t("pluginManagerV1.installed.packStatusPickFirst"));
    return;
  }
  try {
    const r = await packPlugin(pid);
    pluginPackStatus.value = String(t("pluginManagerV1.installed.packStatusDone", { path: r.archive_path }));
  } catch (e) {
    pluginPackStatus.value = e instanceof Error ? e.message : String(e);
  }
}

function requestClosePmPanel(): void {
  if (expertModelsStore.workbenchDraftDirty) {
    if (!window.confirm(String(t("expertModels.confirm.unsavedWorkbenchClose")))) return;
  }
  pluginStore.closePanel();
}

function closePmAndOpenMarket(): void {
  if (expertModelsStore.workbenchDraftDirty) {
    if (!window.confirm(String(t("expertModels.confirm.unsavedWorkbenchClose")))) return;
  }
  pluginStore.closePanel();
  void pluginStore.openMarketPanel();
}

function onExpertModelsOpenPermissions(payload: { pluginId: string }): void {
  const pid = payload.pluginId.trim();
  if (!pid) return;
  pluginStore.panelMainTab = "plugins";
  selectWorkspacePlugin(pid);
  showToast("info", String(t("pluginManagerV1.ui.expertModels.permNavToast")));
}

watch(
  () => pluginStore.expertModelsWorkbenchRequestEpoch,
  async (n, prev) => {
    if (n <= 0 || n === prev) return;
    await nextTick();
    if (pluginStore.panelMainTab !== "backends") return;
    expertFacilitySectionRef.value?.scrollIntoView({ behavior: "smooth", block: "nearest" });
  },
);
</script>

<template>
  <Teleport to="body">
    <div
      v-if="pluginStore.panelVisible"
      class="pm-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="String(t('pluginManagerV1.ui.dialogLabel'))"
      @click.self="requestClosePmPanel()"
    >
      <div
        v-if="preflightVisible"
        class="pm-modal-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="String(t('pluginManagerV1.ui.preflight.dialogLabel'))"
        @click.self="onPreflightCancel"
      >
        <div class="pm-modal" @click.stop>
          <div class="pm-modal-h">{{ preflightTitle }}</div>
          <p class="pm-hint">{{ t("pluginManagerV1.ui.preflight.hint") }}</p>
          <ul class="pm-preflight-list">
            <li v-for="(x, idx) in preflightLines" :key="`pl-${idx}`" class="pm-preflight-li">
              <span style="white-space: pre-wrap">{{ x }}</span>
            </li>
          </ul>
          <div class="pm-modal-actions pm-modal-actions--foot">
            <button type="button" class="pm-btn secondary" @click="onPreflightCancel">
              {{ t("common.cancel") }}
            </button>
            <button type="button" class="pm-btn primary" @click="onPreflightConfirm">
              {{ t("pluginManagerV1.ui.preflight.confirmAndContinue") }}
            </button>
          </div>
        </div>
      </div>
      <div
        v-if="permConsentVisible"
        class="pm-modal-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="String(t('pluginManagerV1.ui.permConsent.dialogLabel'))"
        @click.self="onPermConsentCancel"
      >
        <div class="pm-modal" @click.stop>
          <div class="pm-modal-h">{{ permConsentTitle }}</div>
          <p v-if="permConsentTrustSummary" class="pm-trust-summary">
            <span class="pm-trust-h">{{ t("pluginManagerV1.ui.permConsent.trustSummaryTitle") }}</span>
            <br />
            <span class="pm-trust-mono" style="white-space: pre-wrap">{{
              permConsentTrustSummary
            }}</span>
          </p>
          <p class="pm-hint">
            {{ t("pluginManagerV1.ui.permConsent.hint") }}
          </p>
          <p v-if="permTokenInfoLoading" class="pm-muted" style="margin: 6px 0 0">
            {{ t("pluginManagerV1.ui.permConsent.loadingTokenInfo") }}
          </p>
          <div class="pm-modal-actions">
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="setPermConsentAll(true)"
            >
              {{ t("pluginManagerV1.ui.permConsent.selectAll") }}
            </button>
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="setPermConsentAll(false)"
            >
              {{ t("pluginManagerV1.ui.permConsent.selectNone") }}
            </button>
          </div>
          <ul class="pm-perm-list">
            <li v-for="p in permConsentPerms" :key="p" class="pm-perm-li">
              <label class="pm-perm-row">
                <input
                  type="checkbox"
                  :checked="permConsentSelected[p] === true"
                  @change="
                    permConsentSelected = {
                      ...permConsentSelected,
                      [p]: ($event.target as HTMLInputElement).checked,
                    }
                  "
                />
                <span class="pm-perm-token">{{ p }}</span>
                <span
                  v-if="permTokenInfoMap.get(p)?.risk"
                  class="pm-perm-risk"
                  :class="riskClass(permTokenInfoMap.get(p)?.risk)"
                >
                  {{ riskLabel(permTokenInfoMap.get(p)?.risk) }}
                </span>
              </label>
              <div
                v-if="permTokenInfoMap.get(p)?.title || permTokenInfoMap.get(p)?.description"
                class="pm-perm-desc"
              >
                <div v-if="permTokenInfoMap.get(p)?.title" class="pm-perm-title">
                  {{ permTokenInfoMap.get(p)?.title }}
                </div>
                <div v-if="permTokenInfoMap.get(p)?.description" class="pm-muted">
                  {{ permTokenInfoMap.get(p)?.description }}
                </div>
              </div>
            </li>
          </ul>
          <div class="pm-modal-actions pm-modal-actions--foot">
            <button type="button" class="pm-btn secondary" @click="onPermConsentCancel">
              {{ t("common.cancel") }}
            </button>
            <button type="button" class="pm-btn" @click="onPermConsentConfirm">
              {{ t("pluginManagerV1.ui.permConsent.continueInstall") }}
            </button>
          </div>
        </div>
      </div>
      <div class="pm-dialog pm-dialog--studio" @click.stop>
        <header class="pm-head">
          <div class="pm-head-row">
            <h2 class="pm-title">{{ t("pluginManagerV1.ui.title") }}</h2>
            <span
              class="pm-studio-badge"
              :title="String(t('pluginManagerV1.ui.proModeBadgeTitle'))"
            >{{ t("pluginManagerV1.ui.proModeBadge") }}</span>
          </div>
          <p class="pm-sub">{{ t("pluginManagerV1.ui.subtitle") }}</p>
          <button type="button" class="pm-close" :aria-label="String(t('common.close'))" @click="requestClosePmPanel()">
            ×
          </button>
        </header>

        <div v-if="pluginStore.loading" class="pm-muted pm-dialog-pad">{{ t("pluginManagerV1.ui.loading") }}</div>
        <p v-else-if="pluginStore.error" class="pm-err pm-dialog-pad">{{ pluginStore.error }}</p>

        <template v-else>
          <div class="pm-tabs" role="tablist" :aria-label="String(t('pluginManagerV1.ui.tabsAria'))">
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'plugins' }"
              :aria-selected="pluginStore.panelMainTab === 'plugins'"
              @click="pluginStore.panelMainTab = 'plugins'"
            >
              {{ t("pluginManagerV1.ui.tabs.plugins") }}
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'backends' }"
              :aria-selected="pluginStore.panelMainTab === 'backends'"
              @click="pluginStore.panelMainTab = 'backends'"
            >
              {{ t("pluginManagerV1.ui.tabs.backends") }}
            </button>
            <button
              type="button"
              role="tab"
              class="pm-tab"
              :class="{ 'pm-tab--active': pluginStore.panelMainTab === 'slots' }"
              :aria-selected="pluginStore.panelMainTab === 'slots'"
              @click="pluginStore.panelMainTab = 'slots'"
            >
              {{ t("pluginManagerV1.ui.tabs.slots") }}
            </button>
          </div>

          <div class="pm-scroll">
          <div
            v-show="pluginStore.panelMainTab === 'plugins'"
            class="pm-tab-panel"
            role="tabpanel"
          >
          <section class="pm-section">
            <div class="pm-section-head">
              <h3 class="pm-h3">{{ t("pluginManagerV1.ui.market.title") }}</h3>
              <div class="pm-section-actions">
                <button type="button" class="pm-btn secondary pm-btn--sm" @click="closePmAndOpenMarket()">
                  {{ t("pluginManagerV1.ui.market.openMarket") }}
                </button>
              </div>
            </div>
            <p class="pm-hint">
              {{ t("pluginManagerV1.ui.market.hint") }}
            </p>
          </section>
          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.persistScope.title") }}</h3>
            <p class="pm-hint">
              {{ t("pluginManagerV1.ui.persistScope.hint") }}
            </p>
            <div class="pm-scope-row" role="group" :aria-label="String(t('pluginManagerV1.ui.persistScope.aria'))">
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'role'"
                  @change="pluginStore.setPersistScope('role')"
                />
                {{ t("pluginManagerV1.ui.persistScope.roleOnly") }}
              </label>
              <label class="pm-scope-label">
                <input
                  type="radio"
                  name="pm-persist-scope"
                  :checked="pluginStore.persistScope === 'global'"
                  @change="pluginStore.setPersistScope('global')"
                />
                {{ t("pluginManagerV1.ui.persistScope.globalDefault") }}
              </label>
            </div>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.shell.title") }}</h3>
            <p class="pm-hint">
              {{ t("pluginManagerV1.shell.hint") }}
            </p>
            <p class="pm-muted" v-if="supportedUiSlotsForShell.length === 0">
              {{ t("pluginManagerV1.shell.noSupportedSlotsHint") }}
            </p>
            <div v-else class="pm-shell-slots">
              <div class="pm-shell-slots-row">
                <span class="pm-muted">{{ t("pluginManagerV1.shell.supportedLabel") }}</span>
                <span class="pm-shell-chip" v-for="s in supportedUiSlotsForShell" :key="`sup-${s}`">
                  {{ s }}
                </span>
              </div>
              <div v-if="unsupportedOfficialUiSlots.length > 0" class="pm-shell-slots-row">
                <span class="pm-muted">{{ t("pluginManagerV1.shell.unsupportedOfficialLabel") }}</span>
                <span
                  class="pm-shell-chip pm-shell-chip--warn"
                  v-for="s in unsupportedOfficialUiSlots"
                  :key="`unsup-${s}`"
                >
                  {{ s }}
                </span>
              </div>
            </div>
          </section>

          <section class="pm-section">
            <div class="pm-section-head">
              <h3 class="pm-h3">{{ t("pluginManagerV1.profileSection.title") }}</h3>
              <div class="pm-section-actions">
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="profilePreviewLoading"
                  @click="onPickProfilePreview"
                >
                  {{
                    profilePreviewLoading
                      ? t("pluginManagerV1.profileSection.loadingPreview")
                      : t("pluginManagerV1.profileSection.pickFile")
                  }}
                </button>
                <button
                  v-if="profilePreview"
                  type="button"
                  class="pm-btn primary pm-btn--sm"
                  :disabled="profileApplyLoading"
                  @click="onApplyProfile"
                >
                  {{
                    profileApplyLoading
                      ? t("pluginManagerV1.profileSection.applying")
                      : t("pluginManagerV1.profileSection.apply")
                  }}
                </button>
              </div>
            </div>
            <p class="pm-hint">
              {{ t("pluginManagerV1.profileSection.hint") }}
            </p>
            <div v-if="profilePreview" class="pm-profile-preview">
              <p class="pm-muted">
                <strong>{{ profilePreview.name }}</strong>
                <span class="pm-muted"> · {{ profilePreview.id }} · v{{ profilePreview.version }}</span>
              </p>
              <p v-if="(profilePreview.marketSources ?? []).length" class="pm-muted">
                {{ t("pluginManagerV1.profileSection.marketSourcesLabel") }}：{{ profilePreview.marketSources.join("、") }}
              </p>
              <p class="pm-muted">
                {{ t("pluginManagerV1.profileSection.developerModeLabel") }}：{{
                  profilePreview.developerMode
                    ? t("pluginManagerV1.profileSection.devModeOn")
                    : t("pluginManagerV1.profileSection.devModeOff")
                }}
              </p>
              <p v-if="(profilePreview.plugins ?? []).length" class="pm-muted">
                {{ t("pluginManagerV1.profileSection.pluginsLabel") }}：{{ profilePreview.plugins.map((x) => x.id).join("、") }}
              </p>
              <p v-if="profilePreview.backends" class="pm-muted">
                {{ t("pluginManagerV1.profileSection.backendsLabel") }}：{{
                  Object.entries(profilePreview.backends)
                    .filter(([, v]) => !!(v ?? "").toString().trim())
                    .map(([k, v]) => `${k}=${v}`)
                    .join("，")
                }}
              </p>
            </div>
            <p v-else class="pm-muted">{{ t("pluginManagerV1.profileSection.empty") }}</p>
          </section>

          <section
            v-if="roleStore.roleInfo.authorPack?.suggested_plugin_backends"
            class="pm-section"
          >
            <h3 class="pm-h3">{{ t("pluginManagerV1.authorSuggestedBackends.title") }}</h3>
            <p class="pm-hint">
              {{ t("pluginManagerV1.authorSuggestedBackends.hint") }}
            </p>
            <button
              type="button"
              class="pm-btn secondary pm-btn--sm"
              @click="onApplyAuthorSuggestedBackends"
            >
              {{ t("pluginManagerV1.authorSuggestedBackends.apply") }}
            </button>
          </section>

          <section v-if="roleStore.roleInfo.authorPack" class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.authorPack.title") }}</h3>
            <p v-if="roleStore.roleInfo.authorPack.summary" class="pm-author-summary">
              {{ roleStore.roleInfo.authorPack.summary }}
            </p>
            <ul
              v-if="(roleStore.roleInfo.authorPack.recommended_plugins ?? []).length"
              class="pm-rec-list"
            >
              <li
                v-for="(rp, idx) in roleStore.roleInfo.authorPack.recommended_plugins"
                :key="`${rp.id}-${idx}`"
              >
                <strong>{{ rp.id }}</strong>
                <span v-if="rp.version_range" class="pm-muted"> · {{ rp.version_range }}</span>
                <span v-if="rp.optional" class="pm-muted">{{ t("pluginManagerV1.authorPack.optional") }}</span>
              </li>
            </ul>
            <p v-else class="pm-muted">{{ t("pluginManagerV1.authorPack.empty") }}</p>
          </section>

          <section v-if="false" id="pm-community-index" class="pm-section">
            <div class="pm-section-head">
              <h3 class="pm-h3">{{ t("pluginManagerV1.communityIndex.title") }}</h3>
              <div class="pm-section-actions">
                <div
                  class="pm-market-tabs"
                  role="tablist"
                  :aria-label="String(t('pluginManagerV1.communityIndex.tabs.aria'))"
                >
                  <button
                    type="button"
                    class="pm-tab pm-tab--sm"
                    :class="{ 'pm-tab--active': marketEntryTab === 'plugin' }"
                    role="tab"
                    :aria-selected="marketEntryTab === 'plugin'"
                    @click="marketEntryTab = 'plugin'"
                  >
                    {{ t("pluginManagerV1.communityIndex.tabs.plugin") }}
                  </button>
                  <button
                    type="button"
                    class="pm-tab pm-tab--sm"
                    :class="{ 'pm-tab--active': marketEntryTab === 'module' }"
                    role="tab"
                    :aria-selected="marketEntryTab === 'module'"
                    @click="marketEntryTab = 'module'"
                  >
                    {{ t("pluginManagerV1.communityIndex.tabs.module") }}
                  </button>
                  <button
                    type="button"
                    class="pm-tab pm-tab--sm"
                    :class="{ 'pm-tab--active': marketEntryTab === 'profile' }"
                    role="tab"
                    :aria-selected="marketEntryTab === 'profile'"
                    @click="marketEntryTab = 'profile'"
                  >
                    Profile
                  </button>
                </div>
                <select
                  class="pm-select pm-select--sm"
                  :value="marketSourceSelected"
                  @change="marketSourceSelected = ($event.target as HTMLSelectElement).value"
                >
                  <option value="official">{{ t("pluginManagerV1.communityIndex.source.official") }}</option>
                  <option
                    v-for="s in marketSources"
                    :key="s"
                    :value="s"
                  >
                    {{ t("pluginManagerV1.communityIndex.source.thirdParty", { s }) }}
                  </option>
                </select>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginMarketSyncing"
                  @click="onSyncMarketIndex"
                >
                  {{
                    pluginStore.pluginMarketSyncing
                      ? t("pluginManagerV1.communityIndex.syncing")
                      : t("pluginManagerV1.communityIndex.sync")
                  }}
                </button>
              </div>
            </div>
            <p v-if="pluginStore.pluginMarketError" class="pm-err">{{ pluginStore.pluginMarketError }}</p>
            <p
              v-else-if="pluginStore.pluginMarketSnapshot?.warning"
              class="pm-hint"
            >
              {{ pluginStore.pluginMarketSnapshot.warning }}
            </p>
            <p v-if="pluginStore.pluginMarketSnapshot?.offlineMode" class="pm-hint">
              {{ t("pluginManagerV1.communityIndex.offlineHint") }}
            </p>
            <details class="pm-local-imports">
              <summary class="pm-local-imports-sum">{{ t("pluginManagerV1.localImports.sectionTitle") }}</summary>
              <div class="pm-local-imports-body">
                <p class="pm-muted">
                  <span v-html="t('pluginManagerV1.localImports.sectionHintHtml')"></span>
                </p>
                <p v-if="localImportsRootDir" class="pm-muted">
                  {{ t("pluginManagerV1.localImports.rootLabel") }}：<code>{{ localImportsRootDir }}</code>
                </p>
                <ul class="pm-muted">
                  <li><code>roles/</code>：{{ t("pluginManagerV1.localImports.paths.roles") }}</li>
                  <li><code>plugins/plugin/</code>：{{ t("pluginManagerV1.localImports.paths.pluginsPlugin") }}</li>
                  <li><code>plugins/module/</code>：{{ t("pluginManagerV1.localImports.paths.pluginsModule") }}</li>
                  <li><code>profiles/</code>：{{ t("pluginManagerV1.localImports.paths.profiles") }}</li>
                </ul>
                <div class="pm-row">
                  <button
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    :disabled="localImportsLoading"
                    @click="refreshLocalImports"
                  >
                    {{
                      localImportsLoading
                        ? t("pluginManagerV1.localImports.scanning")
                        : t("pluginManagerV1.localImports.scan")
                    }}
                  </button>
                  <button type="button" class="pm-btn secondary pm-btn--sm" @click="unhideAllLocalImports">
                    {{ t("pluginManagerV1.localImports.showAll") }}
                  </button>
                  <span v-if="localImportsErr" class="pm-err"> {{ localImportsErr }} </span>
                </div>

                <div v-if="localImports.length === 0" class="pm-muted">{{ t("pluginManagerV1.localImports.empty") }}</div>
                <div v-else class="pm-local-imports-grid">
                  <div class="pm-local-imports-col">
                    <h4>{{ t("pluginManagerV1.localImports.cols.rolePacks") }}</h4>
                    <ul class="pm-local-imports-list">
                      <li v-for="it in localImportsByKind('role_pack')" :key="it.path">
                        <code>{{ it.fileName }}</code>
                        <button type="button" class="pm-link" @click="onImportRolePackFromLocal(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.import") }}
                        </button>
                        <button
                          type="button"
                          class="pm-link"
                          :title="String(t('pluginManagerV1.localImports.actions.overwriteImportTitle'))"
                          @click="onImportRolePackFromLocalOverwrite(it.path)"
                        >
                          {{ t("pluginManagerV1.localImports.actions.overwriteImport") }}
                        </button>
                      </li>
                    </ul>
                  </div>

                  <div class="pm-local-imports-col">
                    <h4>{{ t("pluginManagerV1.localImports.cols.plugins") }}</h4>
                    <ul class="pm-local-imports-list">
                      <li v-for="it in localImportsByKind('plugin_archive')" :key="it.path">
                        <code>{{ it.fileName }}</code>
                        <button type="button" class="pm-link" @click="onInstallPluginArchiveFromLocal(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.install") }}
                        </button>
                        <button type="button" class="pm-link" @click="hideLocalImport(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.hide") }}
                        </button>
                      </li>
                      <li v-for="it in localImportsByKind('plugin_dir')" :key="it.path">
                        <code>{{ it.fileName }}</code>
                        <button type="button" class="pm-link" @click="onInstallPluginDirFromLocal(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.install") }}
                        </button>
                        <button type="button" class="pm-link" @click="hideLocalImport(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.hide") }}
                        </button>
                      </li>
                    </ul>
                  </div>

                  <div class="pm-local-imports-col">
                    <h4>{{ t("pluginManagerV1.localImports.cols.moduleProfile") }}</h4>
                    <ul class="pm-local-imports-list">
                      <li v-for="it in localImportsByKind('module_json')" :key="it.path">
                        <code>{{ it.fileName }}</code>
                        <button type="button" class="pm-link" @click="onApplyLocalModuleOrProfile(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.apply") }}
                        </button>
                        <button type="button" class="pm-link" @click="onPreviewLocalJson(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.copyJson") }}
                        </button>
                        <button type="button" class="pm-link" @click="hideLocalImport(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.hide") }}
                        </button>
                      </li>
                      <li v-for="it in localImportsByKind('profile_json')" :key="it.path">
                        <code>{{ it.fileName }}</code>
                        <button type="button" class="pm-link" @click="onApplyLocalModuleOrProfile(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.apply") }}
                        </button>
                        <button type="button" class="pm-link" @click="onPreviewLocalJson(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.copyJson") }}
                        </button>
                        <button type="button" class="pm-link" @click="hideLocalImport(it.path)">
                          {{ t("pluginManagerV1.localImports.actions.hide") }}
                        </button>
                      </li>
                    </ul>
                  </div>
                </div>
              </div>
            </details>
            <p
              v-if="marketSourceSelected !== 'official'"
              class="pm-err"
            >
              {{ t("pluginManagerV1.communityIndex.thirdPartyWarning") }}
            </p>
            <p
              v-if="
                !pluginStore.pluginMarketSnapshot?.plugins?.length &&
                !pluginStore.pluginMarketError
              "
              class="pm-muted"
            >
              {{ t("pluginManagerV1.communityIndex.emptyHint") }}
            </p>
            <div
              v-else-if="marketRowsFiltered.length > 0"
              class="pm-market-pager"
              role="toolbar"
              :aria-label="String(t('pluginManagerV1.communityIndex.pager.aria'))"
            >
              <span class="pm-muted">
                {{
                  t("pluginManagerV1.communityIndex.pager.summary", {
                    total: marketRowsFiltered.length,
                    page: marketPage,
                    pages: marketTotalPages,
                  })
                }}
              </span>
              <label class="pm-muted">
                {{ t("pluginManagerV1.communityIndex.pager.perPage") }}
                <select
                  v-model.number="marketPageSize"
                  class="pm-select pm-select--sm"
                  :aria-label="String(t('pluginManagerV1.communityIndex.pager.perPageAria'))"
                >
                  <option :value="15">15</option>
                  <option :value="30">30</option>
                  <option :value="60">60</option>
                </select>
              </label>
              <button
                type="button"
                class="pm-btn secondary pm-btn--sm"
                :disabled="marketPage <= 1"
                @click="marketPage = Math.max(1, marketPage - 1)"
              >
                {{ t("pluginManagerV1.communityIndex.pager.prev") }}
              </button>
              <button
                type="button"
                class="pm-btn secondary pm-btn--sm"
                :disabled="marketPage >= marketTotalPages"
                @click="marketPage = Math.min(marketTotalPages, marketPage + 1)"
              >
                {{ t("pluginManagerV1.communityIndex.pager.next") }}
              </button>
            </div>
            <ul v-if="marketRowsPaged.length > 0" class="pm-market-list">
              <li
                v-for="row in marketRowsPaged"
                :key="row.id"
                class="pm-market-li"
              >
                <div class="pm-market-main">
                  <strong>{{ row.id }}</strong>
                  <span
                    class="pm-source-badge"
                    :class="(row.source ?? '') === 'official' ? 'official' : 'third'"
                    :title="
                      (row.source ?? '') === 'official'
                        ? String(t('pluginManagerV1.communityIndex.sourceBadge.officialTitle'))
                        : String(t('pluginManagerV1.communityIndex.sourceBadge.thirdTitle'))
                    "
                  >
                    {{
                      (row.source ?? "") === "official"
                        ? t("pluginManagerV1.communityIndex.sourceBadge.official")
                        : t("pluginManagerV1.communityIndex.sourceBadge.third")
                    }}
                  </span>
                  <span
                    v-if="marketEntryType(row) !== 'plugin'"
                    class="pm-entry-type-badge"
                    :class="marketEntryType(row)"
                    :title="
                      marketEntryType(row) === 'module'
                        ? String(t('pluginManagerV1.communityIndex.entryTypeBadge.moduleTitle'))
                        : String(t('pluginManagerV1.communityIndex.entryTypeBadge.profileTitle'))
                    "
                  >
                    {{
                      marketEntryType(row) === "module"
                        ? t("pluginManagerV1.communityIndex.entryTypeBadge.module")
                        : t("pluginManagerV1.communityIndex.entryTypeBadge.profile")
                    }}
                  </span>
                  <span class="pm-muted"> · {{ row.name }} · v{{ row.version }}</span>
                  <p v-if="row.source || row.publisher" class="pm-market-trust">
                    <span v-if="row.source" class="pm-muted"
                      >{{ t("pluginManagerV1.communityIndex.trust.source") }}：{{ row.source }}</span
                    >
                    <span v-if="row.publisher" class="pm-muted"
                      > · {{ t("pluginManagerV1.communityIndex.trust.publisher") }}：{{ row.publisher }}</span
                    >
                    <span
                      v-if="(row.publicKeys ?? []).length"
                      class="pm-muted"
                      :title="String(t('pluginManagerV1.communityIndex.trust.pubkeysTitle'))"
                    >
                      · {{ t("pluginManagerV1.communityIndex.trust.pubkeys") }}：{{
                        (row.publicKeys ?? [])
                          .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
                          .join("，")
                      }}
                    </span>
                  </p>
                  <p class="pm-market-rating">
                    <span
                      class="pm-rating-stars"
                      :title="
                        String(
                          t('pluginManagerV1.communityIndex.reviews.overallTitle', {
                            rating: ratingTextForPluginId(row.id),
                          }),
                        )
                      "
                    >
                      {{ ratingStarsForPluginId(row.id) }}
                    </span>
                    <span class="pm-muted"> · {{ ratingTextForPluginId(row.id) }}</span>
                    <template v-if="(row.publicKeys ?? []).length">
                      <span class="pm-muted"> · {{ t("pluginManagerV1.communityIndex.reviews.pubkeyDimension") }}</span>
                      <span
                        v-for="k in row.publicKeys ?? []"
                        :key="`rv-${row.id}-${k.pubkeyId}`"
                        class="pm-pubkey-rating"
                        :title="`pubkeyId=${k.pubkeyId}${k.status ? ` (${k.status})` : ''}`"
                      >
                        <span class="pm-muted">{{ k.pubkeyId }}</span>
                        <span class="pm-rating-stars">{{
                          ratingStarsForPluginPubkey(row.id, k.pubkeyId)
                        }}</span>
                        <span class="pm-muted">({{
                          ratingTextForPluginPubkey(row.id, k.pubkeyId)
                        }})</span>
                        <button
                          type="button"
                          class="pm-link pm-link--tiny"
                          :disabled="pluginReviewsLoading"
                          :title="String(t('pluginManagerV1.communityIndex.reviews.copyPubkeyTemplateTitle'))"
                          @click="
                            copyReviewTemplate({
                              pluginId: row.id,
                              pubkeyId: k.pubkeyId,
                              version: marketPickedVersionForRow(row) ?? null,
                            })
                          "
                        >
                          {{ t("common.copy") }}
                        </button>
                      </span>
                    </template>
                    <button
                      type="button"
                      class="pm-link"
                      :disabled="pluginReviewsLoading"
                      @click="openPluginReviewsContribution"
                    >
                      {{ t("pluginManagerV1.communityIndex.reviews.goContribute") }}
                    </button>
                    <button
                      type="button"
                      class="pm-link"
                      :disabled="pluginReviewsLoading"
                      @click="
                        copyReviewTemplate({
                          pluginId: row.id,
                          pubkeyId: row.publicKeys?.[0]?.pubkeyId ?? null,
                          version: marketPickedVersionForRow(row) ?? null,
                        })
                      "
                      :title="String(t('pluginManagerV1.communityIndex.reviews.copyOverallTemplateTitle'))"
                    >
                      {{ t("pluginManagerV1.communityIndex.reviews.copyTemplate") }}
                    </button>
                    <button
                      type="button"
                      class="pm-link"
                      :disabled="pluginReviewsLoading"
                      @click="syncPluginReviewsIndexNow"
                    >
                      {{ t("pluginManagerV1.communityIndex.reviews.refresh") }}
                    </button>
                    <span v-if="pluginReviewsErr" class="pm-err"> · {{ pluginReviewsErr }}</span>
                  </p>
                  <div
                    v-if="
                      getRecentReviews(pluginReviewsIndex?.reviews ?? [], {
                        pluginId: row.id,
                        limit: 3,
                      }).length
                    "
                    class="pm-market-reviews"
                  >
                    <p class="pm-market-reviews-h">{{ t("pluginManagerV1.communityIndex.reviews.recent") }}</p>
                    <ul class="pm-market-reviews-list">
                      <li
                        v-for="r in getRecentReviews(pluginReviewsIndex?.reviews ?? [], {
                          pluginId: row.id,
                          limit: 3,
                        })"
                        :key="`rr-${row.id}-${r.id}`"
                      >
                        <span class="pm-market-review-line" :title="r.created_at">{{
                          renderReviewLine(r)
                        }}</span>
                      </li>
                    </ul>
                  </div>
                  <p v-if="row.description" class="pm-market-desc">{{ row.description }}</p>
                  <details v-if="marketEntryType(row) === 'module' && (row as any).module" class="pm-market-details">
                    <summary class="pm-market-details-sum">{{ t("pluginManagerV1.communityIndex.details.viewModule") }}</summary>
                    <div class="pm-market-details-body">
                      <p class="pm-muted" v-if="(((row as any).module.plugins ?? []) as any[]).length">
                        {{ t("pluginManagerV1.communityIndex.details.deps") }}：{{
                          ((row as any).module.plugins ?? []).map((x: any) => x.id).join("、")
                        }}
                      </p>
                      <div v-if="summarizeOverrideBackends(((row as any).module.backends ?? null) as any).length">
                        <p class="pm-muted">{{ t("pluginManagerV1.communityIndex.details.backends") }}</p>
                        <ul class="pm-kv-list">
                          <li
                            v-for="(x, idx) in summarizeOverrideBackends(((row as any).module.backends ?? null) as any)"
                            :key="`mb-${idx}`"
                            class="pm-kv-li"
                          >
                            {{ x }}
                          </li>
                        </ul>
                      </div>
                      <p v-else class="pm-muted">{{ t("pluginManagerV1.communityIndex.details.noBackends") }}</p>
                    </div>
                  </details>
                  <details v-else-if="marketEntryType(row) === 'profile' && (row as any).profile" class="pm-market-details">
                    <summary class="pm-market-details-sum">{{ t("pluginManagerV1.communityIndex.details.viewProfile") }}</summary>
                    <div class="pm-market-details-body">
                      <p class="pm-muted" v-if="(((row as any).profile.plugins ?? []) as any[]).length">
                        {{ t("pluginManagerV1.communityIndex.details.deps") }}：{{
                          ((row as any).profile.plugins ?? []).map((x: any) => x.id).join("、")
                        }}
                      </p>
                      <p class="pm-muted" v-if="(((row as any).profile.predeclaredPermissions ?? []) as any[]).length">
                        {{ t("pluginManagerV1.communityIndex.details.predeclaredPerms") }}：{{
                          ((row as any).profile.predeclaredPermissions ?? []).join("、")
                        }}
                      </p>
                      <div v-if="summarizeOverrideBackends(((row as any).profile.backends ?? null) as any).length">
                        <p class="pm-muted">{{ t("pluginManagerV1.communityIndex.details.backends") }}</p>
                        <ul class="pm-kv-list">
                          <li
                            v-for="(x, idx) in summarizeOverrideBackends(((row as any).profile.backends ?? null) as any)"
                            :key="`pb-${idx}`"
                            class="pm-kv-li"
                          >
                            {{ x }}
                          </li>
                        </ul>
                      </div>
                      <p v-else class="pm-muted">{{ t("pluginManagerV1.communityIndex.details.noBackends") }}</p>
                    </div>
                  </details>
                  <p
                    v-if="(row.missingDependencies ?? []).length"
                    class="pm-err pm-market-deps"
                  >
                    {{ t("pluginManagerV1.communityIndex.missingDeps") }}：{{ row.missingDependencies.join("、") }}
                  </p>
                </div>
                <div class="pm-market-actions">
                  <button
                    v-if="marketEntryType(row) === 'module'"
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    @click="onApplyModuleEntry(row)"
                  >
                    {{ t("pluginManagerV1.communityIndex.applyModule") }}
                  </button>
                  <button
                    v-else-if="marketEntryType(row) === 'profile'"
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    @click="onApplyProfileEntry(row)"
                  >
                    {{ t("pluginManagerV1.communityIndex.applyProfile") }}
                  </button>
                  <div
                    v-else-if="(row.versions ?? []).length > 0"
                    class="pm-market-versions"
                  >
                    <select
                      class="pm-select pm-select--sm"
                      :value="marketPickedVersionForRow(row)"
                      @change="
                        (e) =>
                          (marketPickedVersion = {
                            ...marketPickedVersion,
                            [row.id]: (e.target as HTMLSelectElement).value,
                          })
                      "
                    >
                      <option
                        v-for="v in marketVersionsForRow(row)"
                        :key="`${row.id}-${v}`"
                        :value="v"
                      >
                        v{{ v }}
                      </option>
                    </select>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onInstallMarketVersion(row)"
                    >
                      {{ row.installed ? t("pluginManagerV1.ui.marketVersions.rollbackOrSwitch") : t("pluginManagerV1.ui.marketVersions.installThisVersion") }}
                    </button>
                  </div>
                  <button
                    v-else-if="!row.installed"
                    type="button"
                    class="pm-btn secondary pm-btn--sm"
                    @click="onInstallMarketEntry(row)"
                  >
                    {{ t("pluginManagerV1.ui.actions.install") }}
                  </button>
                  <template v-else>
                    <span v-if="row.hasUpdate" class="pm-badge">{{ t("pluginManagerV1.ui.marketVersions.updatable") }}</span>
                    <span v-else class="pm-muted">{{ t("pluginManagerV1.ui.marketVersions.installed") }}</span>
                    <button
                      v-if="row.hasUpdate"
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onUpdateMarketEntry(row)"
                    >
                      {{ t("pluginManagerV1.ui.actions.update") }}
                    </button>
                  </template>
                </div>
              </li>
            </ul>
          </section>

          <section class="pm-section pm-section--catalog">
            <div class="pm-section-head">
              <div class="pm-h3-row">
                <h3 class="pm-h3">{{ t("pluginManagerV1.ui.installed.title") }}</h3>
                <HelpCircle :label="String(t('pluginManagerV1.ui.installed.helpLabel'))">
                  <p>{{ t("pluginManagerV1.ui.installed.helpLine1") }}</p>
                  <p>{{ t("pluginManagerV1.ui.installed.helpLine2") }}</p>
                </HelpCircle>
              </div>
              <div class="pm-section-actions">
                <label class="pm-batch-toggle chk">
                  <input v-model="batchMode" type="checkbox" />
                  {{ t("pluginManagerV1.ui.installed.batchSelect") }}
                </label>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  @click="scaffoldWizardVisible = true"
                >
                  {{ t("pluginManagerV1.ui.installed.newPlugin") }}
                </button>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="!selectedWorkspacePlugin"
                  @click="onPackSelectedPlugin"
                >
                  {{ t("pluginManagerV1.ui.installed.packCurrent") }}
                </button>
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="pluginStore.pluginUpdatesCheckLoading"
                  @click="onCheckUpdates"
                >
                  {{ t("pluginManagerV1.ui.installed.checkUpdates") }}
                </button>
              </div>
            </div>
            <p v-if="pluginPackStatus" class="pm-hint">{{ pluginPackStatus }}</p>
            <div class="pm-row pm-primary-actions" role="toolbar" :aria-label="String(t('pluginManagerV1.ui.installed.primaryActionsAria'))">
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchEnable">
                {{ t("pluginManagerV1.ui.installed.enableSelected") }}
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchDisable">
                {{ t("pluginManagerV1.ui.installed.disableSelected") }}
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchUpdate">
                {{ t("pluginManagerV1.ui.installed.updateSelectedFromGit") }}
              </button>
              <HelpCircle :label="String(t('pluginManagerV1.ui.installed.primaryHelpLabel'))" inline>
                <p>{{ t("pluginManagerV1.ui.installed.primaryHelpLine1") }}</p>
                <p>{{ t("pluginManagerV1.ui.installed.primaryHelpLine2") }}</p>
                <p>{{ t("pluginManagerV1.ui.installed.primaryHelpLine3") }}</p>
              </HelpCircle>
            </div>
            <div
              v-if="batchMode && batchSelectedCount > 0"
              class="pm-batch-bar"
              role="toolbar"
              :aria-label="String(t('pluginManagerV1.ui.installed.batchActionsAria'))"
            >
              <span class="pm-batch-count">{{ t("pluginManagerV1.ui.installed.selectedCount", { n: batchSelectedCount }) }}</span>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchEnable">
                {{ t("pluginManagerV1.ui.actions.enable") }}
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchDisable">
                {{ t("pluginManagerV1.ui.actions.disable") }}
              </button>
              <button type="button" class="pm-btn secondary pm-btn--sm" @click="onBatchUpdate">
                {{ t("pluginManagerV1.ui.actions.updateFromGit") }}
              </button>
            </div>
            <p v-if="!pluginStore.catalog.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.installed.noDirectoryPluginsFound") }}
            </p>

            <div v-else class="pm-wb" :aria-label="String(t('pluginManagerV1.ui.installed.workspaceAria'))">
              <aside class="pm-wb-sidebar">
                <div class="pm-wb-sidebar-head">
                  <span class="pm-wb-sidebar-title">{{ t("pluginManagerV1.ui.installed.sidebarTitle") }}</span>
                  <span class="pm-wb-sidebar-count">{{ pluginStore.catalog.length }}</span>
                </div>
                <ul class="pm-wb-list" role="listbox" :aria-label="String(t('pluginManagerV1.ui.installed.catalogAria'))">
                  <li v-for="p in pluginStore.catalog" :key="p.id" class="pm-wb-li">
                    <label v-if="batchMode" class="pm-wb-batch chk" @click.stop>
                      <input
                        type="checkbox"
                        :checked="!!batchSelected[p.id]"
                        @change="
                          setBatchSelected(
                            p.id,
                            ($event.target as HTMLInputElement).checked,
                          )
                        "
                      />
                    </label>
                    <button
                      type="button"
                      class="pm-wb-item"
                      :class="{ 'pm-wb-item--active': p.id === selectedWorkspacePluginId }"
                      role="option"
                      :aria-selected="p.id === selectedWorkspacePluginId"
                      @click="selectWorkspacePlugin(p.id)"
                    >
                      <span class="pm-wb-item-id">{{ p.id }}</span>
                      <span class="pm-wb-item-row2">
                        <span class="pm-wb-item-ver">v{{ p.version }}</span>
                        <span class="pm-wb-chip">{{ p.isShell ? t("pluginManagerV1.ui.installed.chip.shell") : t("pluginManagerV1.ui.installed.chip.directory") }}</span>
                        <span
                          v-if="pluginStore.pluginUpdateById[p.id]?.hasUpdate"
                          class="pm-wb-pill"
                        >{{ t("pluginManagerV1.ui.actions.update") }}</span>
                      </span>
                    </button>
                  </li>
                </ul>
              </aside>

              <main v-if="selectedWorkspacePlugin" class="pm-wb-main">
                <div class="pm-wb-main-head">
                  <div class="pm-wb-main-titles">
                    <h4 class="pm-wb-main-h">{{ selectedWorkspacePlugin.id }}</h4>
                    <span class="pm-wb-main-sub">
                      {{ t("pluginManagerV1.ui.installed.mainSub") }}
                    </span>
                  </div>
                  <div class="pm-wb-main-actions">
                    <span
                      v-if="
                        pluginStore.pluginUpdateById[selectedWorkspacePlugin.id]?.hasUpdate
                      "
                      class="pm-badge"
                    >{{ t("pluginManagerV1.ui.installed.hasUpdateBadge") }}</span>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      @click="onGitPullWorkspacePlugin"
                    >
                      {{ t("pluginManagerV1.ui.installed.gitPull") }}
                    </button>
                    <button
                      type="button"
                      class="pm-btn secondary pm-btn--sm"
                      :disabled="
                        pluginStore.extractingPluginId === selectedWorkspacePlugin.id
                      "
                      @click="onUpdateFromZip(selectedWorkspacePlugin.id)"
                    >
                      {{ t("pluginManagerV1.ui.installed.updateFromZip") }}
                    </button>
                  </div>
                </div>
                <div class="pm-wb-main-body">
                  <InstalledPluginWorkspaceDetail
                    :entry="selectedWorkspacePlugin"
                    :batch-mode="batchMode"
                    :batch-selected="!!batchSelected[selectedWorkspacePlugin.id]"
                    @update:batch-selected="
                      setBatchSelected(selectedWorkspacePlugin.id, $event)
                    "
                  />
                </div>
              </main>
            </div>
          </section>
          </div>

          <div
            v-if="false"
            class="pm-tab-panel"
            role="tabpanel"
          >
            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginManagerV1.noCodeModules.title") }}</h3>
              <p class="pm-hint">
                {{ t("pluginManagerV1.noCodeModules.hint") }}
              </p>
              <div class="pm-row">
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="localImportsLoading"
                  @click="refreshLocalImports"
                >
                  {{
                    localImportsLoading
                      ? t("pluginManagerV1.noCodeModules.scanning")
                      : t("pluginManagerV1.noCodeModules.scanLocal")
                  }}
                </button>
                <button
                  v-if="rollbackSnapshotForRole"
                  type="button"
                  class="pm-btn danger pm-btn--sm"
                  :title="
                    String(
                      t('pluginManagerV1.noCodeModules.rollbackTitle', {
                        label: rollbackSnapshotForRole.label,
                        savedAt: rollbackSnapshotForRole.savedAt,
                      }),
                    )
                  "
                  @click="rollbackLastSessionOverride"
                >
                  {{ t("pluginManagerV1.noCodeModules.rollbackLast") }}
                </button>
                <span v-if="localImportsErr" class="pm-err"> {{ localImportsErr }} </span>
              </div>
            </section>

            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginManagerV1.noCodeModules.localTitle") }}</h3>
              <p v-if="localImportsByKind('module_json').length === 0" class="pm-muted">
                <span v-html="t('pluginManagerV1.noCodeModules.localEmptyHtml', { dir: `${localImportsRootDir}/plugins/module` })"></span>
              </p>
              <ul v-else class="pm-market-list">
                <li
                  v-for="it in localImportsByKind('module_json')"
                  :key="`lm-${it.path}`"
                  class="pm-market-li"
                >
                  <div class="pm-market-main">
                    <strong>{{ it.fileName }}</strong>
                    <span class="pm-muted"> · {{ localImportKindLabel(it.kind) }}</span>
                    <div class="pm-market-actions">
                      <button type="button" class="pm-btn" @click="onApplyLocalModuleOrProfile(it.path)">
                        {{ t("pluginManagerV1.noCodeModules.applyModule") }}
                      </button>
                      <button type="button" class="pm-btn secondary" @click="onPreviewLocalJson(it.path)">
                        {{ t("pluginManagerV1.noCodeModules.copyJson") }}
                      </button>
                    </div>
                  </div>
                </li>
              </ul>
            </section>

            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginManagerV1.noCodeModules.marketTitle") }}</h3>
              <p v-if="moduleRowsAll.length === 0" class="pm-muted">
                {{ t("pluginManagerV1.noCodeModules.marketEmpty") }}
              </p>
              <ul v-else class="pm-market-list">
                <li
                  v-for="row in moduleRowsAll"
                  :key="`mm-${row.id}`"
                  class="pm-market-li"
                >
                  <div class="pm-market-main">
                    <strong>{{ row.id }}</strong>
                    <span class="pm-entry-type-badge module">{{ t("pluginManagerV1.communityIndex.entryTypeBadge.module") }}</span>
                    <span class="pm-muted"> · {{ row.name }} · v{{ row.version }}</span>
                    <p v-if="row.description" class="pm-market-desc">{{ row.description }}</p>
                    <div class="pm-market-actions">
                      <button type="button" class="pm-btn" @click="onApplyModuleEntry(row)">
                        {{ t("pluginManagerV1.noCodeModules.applyModule") }}
                      </button>
                    </div>
                  </div>
                </li>
              </ul>
            </section>
          </div>

          <div
            v-if="false"
            class="pm-tab-panel"
            role="tabpanel"
          >
            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginManagerV1.noCodeProfiles.title") }}</h3>
              <p class="pm-hint">
                {{ t("pluginManagerV1.noCodeProfiles.hint") }}
              </p>
              <div class="pm-row">
                <button
                  type="button"
                  class="pm-btn secondary pm-btn--sm"
                  :disabled="localImportsLoading"
                  @click="refreshLocalImports"
                >
                  {{
                    localImportsLoading
                      ? t("pluginManagerV1.noCodeProfiles.scanning")
                      : t("pluginManagerV1.noCodeProfiles.scanLocal")
                  }}
                </button>
                <button
                  v-if="rollbackSnapshotForRole"
                  type="button"
                  class="pm-btn danger pm-btn--sm"
                  :title="
                    String(
                      t('pluginManagerV1.noCodeProfiles.rollbackTitle', {
                        label: rollbackSnapshotForRole.label,
                        savedAt: rollbackSnapshotForRole.savedAt,
                      }),
                    )
                  "
                  @click="rollbackLastSessionOverride"
                >
                  {{ t("pluginManagerV1.noCodeProfiles.rollbackLast") }}
                </button>
                <span v-if="localImportsErr" class="pm-err"> {{ localImportsErr }} </span>
              </div>
            </section>

            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginManagerV1.noCodeProfiles.localTitle") }}</h3>
              <p v-if="localImportsByKind('profile_json').length === 0" class="pm-muted">
                <span v-html="t('pluginManagerV1.noCodeProfiles.localEmptyHtml', { dir: `${localImportsRootDir}/profiles` })"></span>
              </p>
              <ul v-else class="pm-market-list">
                <li
                  v-for="it in localImportsByKind('profile_json')"
                  :key="`lp-${it.path}`"
                  class="pm-market-li"
                >
                  <div class="pm-market-main">
                    <strong>{{ it.fileName }}</strong>
                    <span class="pm-muted"> · {{ localImportKindLabel(it.kind) }}</span>
                    <div class="pm-market-actions">
                      <button type="button" class="pm-btn" @click="onApplyLocalModuleOrProfile(it.path)">
                        {{ t("pluginManagerV1.noCodeProfiles.applyProfile") }}
                      </button>
                      <button type="button" class="pm-btn secondary" @click="onPreviewLocalJson(it.path)">
                        {{ t("pluginManagerV1.noCodeProfiles.copyJson") }}
                      </button>
                    </div>
                  </div>
                </li>
              </ul>
            </section>

            <section class="pm-section">
              <h3 class="pm-h3">{{ t("pluginManagerV1.noCodeProfiles.marketTitle") }}</h3>
              <p v-if="profileRowsAll.length === 0" class="pm-muted">
                {{ t("pluginManagerV1.noCodeProfiles.marketEmpty") }}
              </p>
              <ul v-else class="pm-market-list">
                <li
                  v-for="row in profileRowsAll"
                  :key="`mp-${row.id}`"
                  class="pm-market-li"
                >
                  <div class="pm-market-main">
                    <strong>{{ row.id }}</strong>
                    <span class="pm-entry-type-badge profile">Profile</span>
                    <span class="pm-muted"> · {{ row.name }} · v{{ row.version }}</span>
                    <p v-if="row.description" class="pm-market-desc">{{ row.description }}</p>
                    <div class="pm-market-actions">
                      <button type="button" class="pm-btn" @click="onApplyProfileEntry(row)">
                        {{ t("pluginManagerV1.noCodeProfiles.applyProfile") }}
                      </button>
                    </div>
                  </div>
                </li>
              </ul>
            </section>
          </div>

          <div
            v-show="pluginStore.panelMainTab === 'backends'"
            class="pm-tab-panel pm-tab-panel--backends"
            role="tabpanel"
          >
            <section class="pm-section pm-backends-block">
              <h3 class="pm-h3">{{ t("pluginManagerV1.ui.localLlama.title") }}</h3>
              <p class="pm-hint">
                {{ t("pluginManagerV1.ui.localLlama.hint") }}
              </p>
              <div class="pm-row">
                <label class="pm-label">
                  {{ t("pluginManagerV1.ui.localLlama.pluginIdLabel") }}
                  <input
                    v-model="localLlamaPluginIdDraft"
                    class="pm-input"
                    type="text"
                    autocomplete="off"
                    placeholder="com.oclive.llama.local"
                  />
                </label>
                <span class="pm-muted">
                  {{ t("pluginManagerV1.ui.localLlama.statusLabel") }}：{{
                    localLlamaPluginInstalled
                      ? t("pluginManagerV1.ui.localLlama.status.scanned")
                      : t("pluginManagerV1.ui.localLlama.status.notScanned")
                  }}
                </span>
                <button
                  type="button"
                  class="pm-btn"
                  :disabled="!localLlamaPluginInstalled"
                  @click="onEnableLocalLlamaDirectory"
                >
                  {{ t("pluginManagerV1.ui.localLlama.enableOneClick") }}
                </button>
                <button
                  v-if="rollbackSnapshotForRole"
                  type="button"
                  class="pm-btn danger pm-btn--sm"
                  :title="
                    String(
                      t('pluginManagerV1.ui.localLlama.rollbackTitle', {
                        label: rollbackSnapshotForRole.label,
                        savedAt: rollbackSnapshotForRole.savedAt,
                      }),
                    )
                  "
                  @click="rollbackLastSessionOverride"
                >
                  {{ t("pluginManagerV1.ui.localLlama.rollbackLastOverride") }}
                </button>
              </div>
            </section>

            <section class="pm-section pm-backends-block pm-backends-block--session">
              <PluginBackendSessionPanel />
            </section>

            <section class="pm-section pm-backends-block pm-backends-block--bridge">
              <h3 class="pm-h3">{{ t("pluginManagerV1.ui.expertModels.runtimeTitle") }}</h3>
              <p class="pm-hint">{{ t("pluginManagerV1.ui.expertModels.runtimeHint") }}</p>
              <ExpertModelsRuntimeCard layout="pmSection" />
            </section>

            <section
              ref="expertFacilitySectionRef"
              class="pm-section pm-backends-block pm-backends-block--facility"
              data-pm-expert-facility
            >
              <h3 class="pm-h3">{{ t("pluginManagerV1.ui.expertModels.facilityTitle") }}</h3>
              <p class="pm-hint">{{ t("pluginManagerV1.ui.expertModels.facilityHint") }}</p>
              <div class="pm-expert-facility-scroll">
                <Suspense>
                  <ExpertModelsPanel @open-permissions="onExpertModelsOpenPermissions" />
                  <template #fallback>
                    <p class="pm-muted">{{ t("common.loading") }}</p>
                  </template>
                </Suspense>
              </div>
            </section>
          </div>

          <div
            v-show="pluginStore.panelMainTab === 'slots'"
            class="pm-tab-panel pm-tab-panel--slots"
            role="tabpanel"
          >
          <section class="pm-section pm-embed-slot">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.previewTitle") }}</h3>
            <p class="pm-hint">
              {{ t("pluginManagerV1.ui.slots.previewHint") }}
            </p>
            <div class="pm-embed-preview" aria-hidden="true">
              <PluginSlotEmbed
                slot-name="settings.plugins"
                :bootstrap-epoch="pluginStore.bootstrapEpoch"
              />
            </div>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.settingsPluginsTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.settingsPluginsHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.settingsPluginsAria'))">
              <li
                v-for="(id, i) in settingsPluginsOrder"
                :key="`spl-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_PLUGINS, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_PLUGINS, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_PLUGINS" />
              </li>
            </ol>
            <p v-if="!settingsPluginsOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "settings.plugins" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.chatToolbarTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.chatToolbarHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.chatToolbarAria'))">
              <li
                v-for="(id, i) in toolbarOrder"
                :key="id"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_CHAT_TOOLBAR, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_CHAT_TOOLBAR, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_CHAT_TOOLBAR" />
              </li>
            </ol>
            <p v-if="!toolbarOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "chat_toolbar" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.settingsPanelTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.settingsPanelHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.settingsPanelAria'))">
              <li
                v-for="(id, i) in settingsPanelOrder"
                :key="`sp-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_PANEL, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_PANEL, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_PANEL" />
              </li>
            </ol>
            <p v-if="!settingsPanelOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "settings.panel" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.roleDetailTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.roleDetailHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.roleDetailAria'))">
              <li
                v-for="(id, i) in roleDetailOrder"
                :key="`rd-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_ROLE_DETAIL, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_ROLE_DETAIL, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_ROLE_DETAIL" />
              </li>
            </ol>
            <p v-if="!roleDetailOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "role.detail" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.sidebarTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.sidebarHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.sidebarAria'))">
              <li
                v-for="(id, i) in sidebarOrder"
                :key="`sb-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SIDEBAR, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SIDEBAR, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SIDEBAR" />
              </li>
            </ol>
            <p v-if="!sidebarOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "sidebar" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.chatHeaderTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.chatHeaderHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.chatHeaderAria'))">
              <li
                v-for="(id, i) in chatHeaderOrder"
                :key="`ch-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_CHAT_HEADER, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_CHAT_HEADER, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_CHAT_HEADER" />
              </li>
            </ol>
            <p v-if="!chatHeaderOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "chat.header" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.settingsAdvancedTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.settingsAdvancedHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.settingsAdvancedAria'))">
              <li
                v-for="(id, i) in settingsAdvancedOrder"
                :key="`sa-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_SETTINGS_ADVANCED, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_SETTINGS_ADVANCED, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_SETTINGS_ADVANCED" />
              </li>
            </ol>
            <p v-if="!settingsAdvancedOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "settings.advanced" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.overlayFloatingTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.overlayFloatingHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.overlayFloatingAria'))">
              <li
                v-for="(id, i) in overlayFloatingOrder"
                :key="`of-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_OVERLAY_FLOATING, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_OVERLAY_FLOATING, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_OVERLAY_FLOATING" />
              </li>
            </ol>
            <p v-if="!overlayFloatingOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "overlay.floating" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.launcherPaletteTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.launcherPaletteHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.launcherPaletteAria'))">
              <li
                v-for="(id, i) in launcherPaletteOrder"
                :key="`lp-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_LAUNCHER_PALETTE, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_LAUNCHER_PALETTE, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_LAUNCHER_PALETTE" />
              </li>
            </ol>
            <p v-if="!launcherPaletteOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "launcher.palette" }) }}
            </p>
          </section>

          <section class="pm-section">
            <h3 class="pm-h3">{{ t("pluginManagerV1.ui.slots.debugDockTitle") }}</h3>
            <p class="pm-hint">{{ t("pluginManagerV1.ui.slots.debugDockHint") }}</p>
            <ol class="pm-order" :aria-label="String(t('pluginManagerV1.ui.slots.debugDockAria'))">
              <li
                v-for="(id, i) in debugDockOrder"
                :key="`dd-${id}`"
                class="pm-order-item pm-order-item--row"
                draggable="true"
                @dragstart="onDragSlotStart(SLOT_DEBUG_DOCK, i)"
                @dragover="onDragOver"
                @drop="onDropSlot(SLOT_DEBUG_DOCK, i)"
              >
                <span class="pm-grip" aria-hidden="true">⋮⋮</span>
                <span class="pm-order-id">{{ id }}</span>
                <PmSlotRow :plugin-id="id" :slot-key="SLOT_DEBUG_DOCK" />
              </li>
            </ol>
            <p v-if="!debugDockOrder.length" class="pm-muted">
              {{ t("pluginManagerV1.ui.slots.empty", { slot: "debug.dock" }) }}
            </p>
          </section>
          </div>
          </div>

          <footer class="pm-foot">
            <button type="button" class="pm-btn secondary" @click="requestClosePmPanel()">
              {{ t("common.close") }}
            </button>
            <button type="button" class="pm-btn secondary" @click="onResetToPackDefault">
              {{ t("pluginManagerV1.ui.footer.resetToPackDefault") }}
            </button>
            <button type="button" class="pm-btn primary" @click="onSave">{{ t("common.save") }}</button>
          </footer>
        </template>
      </div>
    </div>
    <PluginScaffoldWizard
      :visible="scaffoldWizardVisible"
      @close="scaffoldWizardVisible = false"
      @created="
        scaffoldWizardVisible = false;
        void pluginStore.refresh();
      "
    />
  </Teleport>
</template>

<style scoped>
.pm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10050;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 45%, transparent));
}
.pm-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10080;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 55%, transparent));
}
.pm-modal {
  width: min(520px, 100%);
  max-height: min(86vh, 720px);
  overflow: auto;
  padding: 14px 16px 12px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-modal-h {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 8px;
}
.pm-trust-summary {
  margin: 0 0 10px;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-trust-h {
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-trust-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-modal-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin: 8px 0;
}
.pm-modal-actions--foot {
  justify-content: flex-end;
  margin-top: 12px;
}
.pm-perm-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm-preflight-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm-preflight-li {
  margin: 0;
  padding: 8px 10px;
  border: 1px solid var(--border-light);
  border-radius: 10px;
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-perm-li {
  margin: 0;
}
.pm-perm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.pm-perm-token {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-perm-title {
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-perm-desc {
  margin-left: 22px;
  margin-top: 2px;
}
.pm-perm-risk {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
}
.pm-perm-risk.risk-high {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
}
.pm-perm-risk.risk-medium {
  color: var(--warn-700, #b9770e);
  border-color: color-mix(in srgb, var(--warn-700, #b9770e) 40%, var(--border-light));
}
.pm-perm-risk.risk-low {
  color: var(--success-700, #1e7e34);
  border-color: color-mix(in srgb, var(--success-700, #1e7e34) 40%, var(--border-light));
}
.pm-dialog {
  position: relative;
  width: min(680px, 100%);
  max-height: min(88vh, 760px);
  overflow: auto;
  padding: 16px 18px 14px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-dialog--studio {
  width: min(1080px, 100%);
  max-height: min(92vh, 900px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm-dialog-pad {
  padding: 12px 18px;
}
.pm-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 18px 22px 14px;
}
.pm-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  flex-shrink: 0;
  padding: 0 22px 14px;
  margin: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-tab {
  flex: 1 1 auto;
  min-width: 0;
  padding: 10px 14px;
  border: 1px solid transparent;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
  transition: var(--control-transition, border-color 0.18s ease, background 0.18s ease);
}
.pm-tab:hover {
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm-tab--active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
  font-weight: 600;
}
.pm-tab--sm {
  flex: 0 0 auto;
  padding: 8px 12px;
  font-size: 13px;
}
.pm-market-tabs {
  display: flex;
  gap: 6px;
  align-items: center;
}
.pm-tab-panel {
  min-height: 0;
}
.pm-tab-panel--backends {
  display: flex;
  flex-direction: column;
  gap: 0;
}
.pm-backends-block {
  margin-bottom: 18px;
  padding-bottom: 18px;
  border-bottom: 1px solid var(--border-light);
}
.pm-backends-block:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}
.pm-backends-block--facility {
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.pm-expert-facility-scroll {
  flex: 1;
  min-height: 260px;
  max-height: min(62vh, 720px);
  overflow: auto;
  margin-top: 8px;
  padding: 10px 12px;
  border-radius: var(--radius-card, 10px);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary, var(--bg-elevated));
}
.pm-embed-preview {
  pointer-events: none;
  user-select: none;
  opacity: 0.97;
  border-radius: var(--radius-card);
  overflow: hidden;
  border: 1px dashed color-mix(in srgb, var(--border-light) 85%, transparent);
}
.pm-head {
  flex-shrink: 0;
  padding: 20px 48px 14px 22px;
  margin: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-head-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.pm-title {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
}
.pm-studio-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--border-light);
  color: var(--text-accent);
  background: color-mix(in srgb, var(--accent) 12%, var(--bg-elevated));
}
.pm-sub {
  margin: 8px 0 0;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}
.pm-kbd {
  display: inline-block;
  padding: 2px 6px;
  margin: 0 2px;
  font-size: 11px;
  font-family: ui-monospace, Menlo, Consolas, monospace;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
}
.pm-close {
  position: absolute;
  top: 12px;
  right: 12px;
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
.pm-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.pm-section {
  margin-bottom: 18px;
  padding: 16px 18px;
  border-radius: 14px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.pm-section--catalog {
  padding: 12px 14px 14px;
  border-radius: var(--radius-card);
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  box-shadow: var(--shadow-sm);
}
.pm-market-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 240px;
  overflow: auto;
}
.pm-market-li {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 13px;
}
.pm-market-main {
  flex: 1 1 200px;
  min-width: 0;
}
.pm-market-desc {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-local-imports {
  margin: 8px 0 0;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  background: var(--bg-elevated);
}
.pm-local-imports-sum {
  cursor: pointer;
  padding: 8px 10px;
  font-size: 13px;
}
.pm-local-imports-body {
  padding: 0 10px 10px;
}
.pm-local-imports-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-top: 8px;
}
.pm-local-imports-col h4 {
  margin: 0 0 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-local-imports-list {
  margin: 0;
  padding-left: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}
@media (max-width: 900px) {
  .pm-local-imports-grid {
    grid-template-columns: 1fr;
  }
}
.pm-market-trust {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm-market-rating {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}
.pm-rating-stars {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
  letter-spacing: 0.5px;
}
.pm-link {
  border: none;
  background: none;
  padding: 0;
  font: inherit;
  color: var(--accent, #6b8cff);
  text-decoration: underline;
  cursor: pointer;
}
.pm-link:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  text-decoration: none;
}
.pm-link--tiny {
  font-size: 11px;
  text-decoration: none;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  padding: 1px 6px;
}
.pm-link--tiny:disabled {
  border-color: transparent;
}
.pm-pubkey-rating {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
}
.pm-source-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  font-size: 11px;
  font-weight: 600;
  vertical-align: middle;
}
.pm-source-badge.official {
  color: var(--success-700, #1e7e34);
  border-color: color-mix(in srgb, var(--success-700, #1e7e34) 40%, var(--border-light));
  background: color-mix(in srgb, var(--success-700, #1e7e34) 8%, var(--bg-primary));
}
.pm-source-badge.third {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
  background: color-mix(in srgb, var(--danger-600, #c0392b) 8%, var(--bg-primary));
}
.pm-market-reviews {
  margin: 4px 0 0;
  padding: 6px 8px;
  border-radius: 10px;
  border: 1px dashed var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
}
.pm-market-reviews-h {
  margin: 0 0 4px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-market-reviews-list {
  margin: 0;
  padding-left: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-market-review-line {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: inline-block;
  max-width: 860px;
  vertical-align: bottom;
}
.pm-market-deps {
  margin: 6px 0 0;
  font-size: 12px;
}
.pm-market-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm-section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}
.pm-section-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}
.pm-batch-toggle {
  font-size: 12px;
  user-select: none;
}
.pm-batch-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px dashed var(--border-light);
  background: var(--bg-elevated);
  font-size: 12px;
}
.pm-batch-count {
  margin-right: 4px;
  color: var(--text-secondary);
}
.pm-primary-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin: 10px 0 14px;
  padding: 10px 12px;
  border-radius: 12px;
  border: 1px dashed var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 55%, transparent);
}
.chk {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}
.pm-h3 {
  margin: 0;
  font-size: 19px;
  line-height: 1.35;
}
.pm-h3-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pm-help {
  position: relative;
}
.pm-help--inline {
  align-self: center;
}
.pm-help-btn {
  list-style: none;
  width: 22px;
  height: 22px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 13px;
  font-weight: 800;
  color: var(--text-secondary);
  background: var(--bg-primary);
  user-select: none;
}
.pm-help-btn::-webkit-details-marker {
  display: none;
}
.pm-help-pop {
  position: absolute;
  z-index: 4;
  top: 28px;
  right: 0;
  min-width: 280px;
  max-width: 380px;
  padding: 10px 12px;
  border-radius: 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-md);
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.55;
}
.pm-help-pop p {
  margin: 0 0 8px;
}
.pm-help-pop p:last-child {
  margin-bottom: 0;
}
.pm-shell-slots {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}
.pm-shell-slots-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  font-size: 12px;
}
.pm-shell-chip {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-shell-chip--warn {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
}

.pm-profile-preview {
  padding: 10px 12px;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  background: var(--bg-elevated);
}

.pm-entry-type-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 6px;
  font-size: 11px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
}
.pm-entry-type-badge.module {
  border-color: color-mix(in srgb, var(--border-light) 70%, #4f46e5);
}
.pm-entry-type-badge.profile {
  border-color: color-mix(in srgb, var(--border-light) 70%, #16a34a);
}
.pm-market-details {
  margin-top: 8px;
}
.pm-market-details-sum {
  cursor: pointer;
  user-select: none;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm-market-details-body {
  margin-top: 6px;
}
.pm-kv-list {
  list-style: none;
  padding: 0;
  margin: 6px 0 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm-kv-li {
  padding: 6px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 11px;
  color: var(--text-secondary);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm-json {
  margin: 6px 0 0;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 11px;
  color: var(--text-secondary);
  overflow: auto;
  max-height: 240px;
}

/* 已安装区：侧栏目录 + 右侧单一配置与调试台 */
.pm-wb {
  display: grid;
  grid-template-columns: minmax(200px, 260px) minmax(0, 1fr);
  gap: 0;
  min-height: min(520px, 58vh);
  max-height: min(62vh, 640px);
  margin-top: 4px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-card);
  overflow: hidden;
  background: var(--bg-primary);
}
.pm-wb-sidebar {
  border-right: 1px solid var(--border-light);
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.pm-wb-sidebar-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-light);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}
.pm-wb-sidebar-title {
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
.pm-wb-sidebar-count {
  font-variant-numeric: tabular-nums;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  font-size: 11px;
}
.pm-wb-list {
  list-style: none;
  margin: 0;
  padding: 4px 0;
  overflow: auto;
  flex: 1;
  min-height: 0;
}
.pm-wb-li {
  display: flex;
  align-items: stretch;
  border-bottom: 1px solid
    color-mix(in srgb, var(--border-light) 70%, transparent);
}
.pm-wb-batch {
  display: flex;
  align-items: center;
  padding: 0 8px;
  flex-shrink: 0;
}
.pm-wb-item {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 2px;
  padding: 8px 10px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font: inherit;
  color: var(--text-primary);
  transition: background 0.15s ease;
}
.pm-wb-item:hover {
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}
.pm-wb-item--active {
  background: var(--bg-elevated);
  box-shadow: inset 3px 0 0 0 var(--accent);
}
.pm-wb-item-id {
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  font-weight: 600;
  word-break: break-all;
}
.pm-wb-item-row2 {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}
.pm-wb-item-ver {
  font-size: 11px;
  color: var(--text-secondary);
}
.pm-wb-chip {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
}
.pm-wb-pill {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-primary));
  color: var(--text-accent);
}
.pm-wb-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}
.pm-wb-main-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-wb-main-titles {
  min-width: 0;
}
.pm-wb-main-h {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  font-family: ui-monospace, Menlo, Consolas, monospace;
}
.pm-wb-main-sub {
  display: block;
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-secondary);
}
.pm-wb-main-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm-wb-main-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.pm-wb-debug {
  border-top: 1px dashed var(--border-light);
  padding-top: 10px;
}
.pm-wb-debug-h {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}
@media (max-width: 720px) {
  .pm-wb {
    grid-template-columns: 1fr;
    max-height: none;
  }
  .pm-wb-sidebar {
    border-right: none;
    border-bottom: 1px solid var(--border-light);
    max-height: 200px;
  }
}

.pm-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-elevated));
  color: var(--text-primary);
}
.pm-btn--sm {
  padding: 8px 12px;
  font-size: 13px;
}
.pm-hint {
  margin: 0 0 12px;
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-secondary);
}
.pm-order {
  margin: 0;
  padding-left: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pm-order-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  font-size: 13px;
  border: 1px dashed var(--border-light);
  border-radius: var(--radius-btn);
  cursor: grab;
  background: var(--bg-elevated);
}
.pm-order-item--row {
  flex-wrap: wrap;
}
.pm-order-id {
  flex: 1;
  min-width: 0;
  word-break: break-all;
}
.pm-embed-slot code {
  font-size: 11px;
}
.pm-grip {
  color: var(--text-secondary);
  font-size: 12px;
  user-select: none;
}
.pm-scope-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-top: 6px;
}
.pm-scope-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
}
.pm-author-summary {
  margin: 0 0 8px;
  font-size: 13px;
  line-height: 1.45;
}
.pm-rec-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  line-height: 1.5;
}
.pm-muted {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}
.pm-err {
  color: var(--error);
  font-size: 13px;
}
.pm-foot {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  margin: 0;
  padding: 12px 18px;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.pm-btn {
  padding: 10px 16px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
}
.pm-btn.secondary {
  background: transparent;
}
.pm-btn.primary {
  background: var(--accent);
  color: var(--bg-elevated);
  border-color: color-mix(in srgb, var(--accent) 85%, var(--text-primary) 15%);
}
.pm-btn.primary:hover {
  filter: brightness(1.05);
}
</style>
