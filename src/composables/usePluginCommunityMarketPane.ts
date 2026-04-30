/**
 * V2「插件市场」页与 V1「社区索引」共用的一套逻辑（索引源、分页、安装、模块/Profile、评价摘要）。
 * 权限确认弹窗状态独立于 PluginManagerPanel，避免与 V1 面板互相抢占。
 */
import { open as openExternal } from "@tauri-apps/api/shell";
import { computed, onMounted, ref, watch } from "vue";
import {
  buildReviewJsonTemplate,
  getRecentReviews,
  renderReviewLine,
} from "../lib/pluginReviewsUi";
import { useAppToast } from "./useAppToast";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import {
  getCachedPluginReviewsIndex,
  listPermissionTokens,
  setSessionPluginBackendsOverride,
  syncPluginReviewsIndex,
  type PermissionTokenInfoDto,
  type PluginMarketEntryDto,
  type PluginReviewEntryDto,
} from "../utils/tauri-api";
import { getPluginMarketSourcesConfig } from "../utils/tauri-api";
import { i18n } from "../i18n";

function t(key: string, params?: Record<string, unknown>): string {
  return String(i18n.global.t(key as any, params as any));
}

const PLUGIN_REVIEWS_REPO_URL =
  "https://github.com/linkaiheng2233-cyber/awesome-oclive-plugin-reviews";
const PLUGIN_REVIEWS_CONTRIBUTING_URL = `${PLUGIN_REVIEWS_REPO_URL}/blob/main/CONTRIBUTING.md`;

const SESSION_OVERRIDE_ROLLBACK_KEY = "oclive.session_override.rollback_v1";
type SessionOverrideSnapshot = {
  roleId: string;
  savedAt: string;
  source: "module" | "profile" | "manual";
  label: string;
  override: Record<string, unknown> | null;
};

function writeRollbackSnapshot(s: SessionOverrideSnapshot): void {
  try {
    localStorage.setItem(`${SESSION_OVERRIDE_ROLLBACK_KEY}.${s.roleId}`, JSON.stringify(s));
  } catch {
    /* ignore */
  }
}

export type MarketEntryTab = "plugin" | "module" | "profile";

export function usePluginCommunityMarketPane(options?: { loadOnMount?: boolean }) {
  const pluginStore = usePluginStore();
  const roleStore = useRoleStore();
  const { showToast } = useAppToast();

  const marketSourceSelected = ref("official");
  const marketSources = ref<string[]>([]);
  const marketEntryTab = ref<MarketEntryTab>("plugin");

  const permConsentVisible = ref(false);
  const permConsentTitle = ref("");
  const permConsentPerms = ref<string[]>([]);
  const permConsentSelected = ref<Record<string, boolean>>({});
  const permConsentTrustSummary = ref("");
  let permConsentResolver: ((v: string[] | null) => void) | null = null;

  const permTokenInfoLoading = ref(false);
  const permTokenInfoMap = ref<Map<string, PermissionTokenInfoDto>>(new Map());

  const preflightVisible = ref(false);
  const preflightTitle = ref("");
  const preflightLines = ref<string[]>([]);
  let preflightResolver: ((v: boolean) => void) | null = null;

  const pluginReviewsLoading = ref(false);
  const pluginReviewsIndex = ref<{ reviews: PluginReviewEntryDto[] } | null>(null);
  const pluginReviewsErr = ref("");

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
    if (!a) return t("pluginManagerV1.reviews.none");
    return t("pluginManagerV1.reviews.summary", { avg: a.avg.toFixed(1), count: a.count });
  }

  function ratingTextForPluginPubkey(pluginId: string, pubkeyId: string): string {
    const key = reviewsAggKey(pluginId, pubkeyId);
    const a = ratingAggByPluginIdPubkey.value.get(key);
    if (!a) return t("pluginManagerV1.reviews.none");
    return t("pluginManagerV1.reviews.summary", { avg: a.avg.toFixed(1), count: a.count });
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
    if (risk === "high") return t("pluginManagerV1.ipwd.risk.high");
    if (risk === "medium") return t("pluginManagerV1.ipwd.risk.medium");
    if (risk === "low") return t("pluginManagerV1.ipwd.risk.low");
    return t("pluginManagerV1.ipwd.risk.unknown");
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
    permConsentTrustSummary.value = trustSummary.trim();
    return await requestPermissionConsent(title, declaredPerms);
  }

  async function requestApplyPreflight(title: string, lines: string[]): Promise<boolean> {
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
    const topKeys = keys.filter((k) => k !== "directory_plugins").sort();
    for (const k of topKeys) {
      const v = (o as Record<string, unknown>)[k];
      if (v === null || v === undefined || String(v).trim() === "") continue;
      out.push(`${k} = ${String(v)}`);
    }
    const dp = (o as { directory_plugins?: Record<string, unknown> }).directory_plugins;
    if (dp && typeof dp === "object") {
      const dpk = Object.keys(dp).sort();
      for (const k of dpk) {
        const v = dp[k];
        if (v === null || v === undefined || String(v).trim() === "") continue;
        out.push(`directory_plugins.${k} = ${String(v)}`);
      }
    }
    return out;
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

  function normalizeProfileSource(s: string | null | undefined): string {
    const t = (s ?? "").trim();
    return t ? t : "official";
  }

  async function loadMarketSourcesForPanel(): Promise<void> {
    try {
      const cfg = await getPluginMarketSourcesConfig();
      marketSources.value = (cfg.pluginIndexSources ?? []).filter((x) => !!x?.trim());
      if (cfg.developerMode !== true) {
        marketSourceSelected.value = "official";
      } else if (
        marketSourceSelected.value !== "official" &&
        !marketSources.value.includes(marketSourceSelected.value)
      ) {
        marketSourceSelected.value = marketSources.value[0] ?? "official";
      }
    } catch {
      marketSources.value = [];
      marketSourceSelected.value = "official";
    }
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
        `同步索引失败（source=${source}）：${msg}\n\n建议：检查网络，或稍后重试；第三方源请确认开发者模式已开启。`,
      );
      throw e;
    }
  }

  function saveCurrentSessionOverrideForRollback(
    source: "module" | "profile" | "manual",
    label: string,
  ): void {
    const roleId = (roleStore.currentRoleId ?? "").trim();
    if (!roleId) return;
    const cur = roleStore.roleInfo.pluginBackendsSessionOverride as unknown;
    const snapshot: SessionOverrideSnapshot = {
      roleId,
      savedAt: new Date().toISOString(),
      source,
      label: label.trim() || "(unknown)",
      override: cur && typeof cur === "object" ? (cur as Record<string, unknown>) : null,
    };
    writeRollbackSnapshot(snapshot);
  }

  const marketPickedVersion = ref<Record<string, string>>({});

  function marketEntryType(row: PluginMarketEntryDto): string {
    return (row as { type?: string }).type ?? "plugin";
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

  function marketVersionsForRow(row: PluginMarketEntryDto): string[] {
    const vs = (row.versions ?? []).map((x) => x.version).filter((x) => !!x?.trim());
    return [...vs].sort((a, b) => (a === b ? 0 : a < b ? 1 : -1));
  }

  function marketPickedVersionForRow(row: PluginMarketEntryDto): string {
    const pid = row.id;
    const picked = marketPickedVersion.value[pid]?.trim();
    if (picked) return picked;
    const vs = marketVersionsForRow(row);
    return vs[0] ?? row.version;
  }

  async function onSyncMarketIndex() {
    try {
      await pluginStore.syncPluginMarket(
        marketSourceSelected.value === "official" ? null : marketSourceSelected.value,
      );
      void syncPluginReviewsIndexNow();
      if (pluginStore.pluginMarketSnapshot?.warning) {
        showToast("info", pluginStore.pluginMarketSnapshot.warning);
      } else {
        showToast("success", "索引已同步。");
      }
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    }
  }

  async function onInstallMarketEntry(row: PluginMarketEntryDto) {
    if ((row.missingDependencies ?? []).length > 0) {
      showToast("error", `依赖未满足，无法安装：${row.missingDependencies.join("、")}`);
      return;
    }
    const declaredPerms = (row.permissions ?? []).map((s) => s.trim()).filter(Boolean);
    const trust = [
      row.source ? `来源：${row.source}` : "",
      row.publisher ? `发布者：${row.publisher}` : "",
      (row.publicKeys ?? []).length
        ? `公钥：${(row.publicKeys ?? [])
            .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
            .join("，")}`
        : "",
    ]
      .filter(Boolean)
      .join("\n");
    const accepted = await requestPermissionConsentWithTrust(
      `安装 ${row.id}`,
      declaredPerms,
      trust,
    );
    if (accepted == null) return;
    if (hasHighRiskPermission(accepted)) {
      const ok2 = window.confirm(
        `你已勾选高风险权限。\n\n建议仅安装你信任的来源。\n\n请再次确认：是否继续安装？`,
      );
      if (!ok2) return;
    }
    try {
      await pluginStore.installFromPluginMarket(row.id, null, accepted);
      showToast("success", `已安装 ${row.id}，建议保存配置并视需要重启应用。`);
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    }
  }

  async function onInstallMarketVersion(row: PluginMarketEntryDto) {
    const v = marketPickedVersionForRow(row);
    if (!v?.trim()) return;
    const declaredPerms = (row.permissions ?? []).map((s) => s.trim()).filter(Boolean);
    const trust = [
      row.source ? `来源：${row.source}` : "",
      row.publisher ? `发布者：${row.publisher}` : "",
      (row.publicKeys ?? []).length
        ? `公钥：${(row.publicKeys ?? [])
            .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
            .join("，")}`
        : "",
    ]
      .filter(Boolean)
      .join("\n");
    const accepted = await requestPermissionConsentWithTrust(
      `安装 ${row.id} v${v}`,
      declaredPerms,
      trust,
    );
    if (accepted == null) return;
    if (hasHighRiskPermission(accepted)) {
      const ok2 = window.confirm(
        `你已勾选高风险权限。\n\n建议仅安装你信任的来源。\n\n请再次确认：是否继续安装 v${v}？`,
      );
      if (!ok2) return;
    }
    try {
      await pluginStore.installVersionFromPluginMarket(row.id, v, accepted);
      showToast(
        "success",
        row.installed ? `已回滚/切换 ${row.id} → v${v}` : `已安装 ${row.id} v${v}`,
      );
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    }
  }

  async function onUpdateMarketEntry(row: PluginMarketEntryDto) {
    try {
      await pluginStore.updateInstalledPluginFromGit(row.id);
      showToast("success", `已更新 ${row.id}（git pull --ff-only）。`);
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    }
  }

  async function onApplyModuleEntry(row: PluginMarketEntryDto): Promise<void> {
    const mod = (row as { module?: unknown }).module as
      | {
          plugins: { id: string; version?: string | null; source?: string | null }[];
          backends?: Record<string, unknown> | null;
        }
      | null
      | undefined;
    if (!mod) {
      showToast("error", "该条目未提供 module 声明体。");
      return;
    }
    const planLines: string[] = [];
    const deps = (mod.plugins ?? []).map((x) => x.id).filter(Boolean);
    const sources = [
      ...new Set((mod.plugins ?? []).map((x) => normalizeProfileSource(x.source ?? null))),
    ];
    planLines.push(`类型：模块（无代码）`);
    planLines.push(`条目：${row.id}`);
    if (sources.length) planLines.push(`将同步索引源：${sources.join("、")}`);
    if (deps.length) planLines.push(`将安装依赖插件：${deps.join("、")}`);
    const changes = summarizeOverrideBackends(mod.backends ?? null);
    if (changes.length) {
      planLines.push(`将写入后端覆盖（会话级）：`);
      for (const x of changes) planLines.push(`- ${x}`);
    }
    const ok = await requestApplyPreflight(`应用模块：${row.id}`, planLines);
    if (!ok) return;
    saveCurrentSessionOverrideForRollback("module", row.id);

    const list = mod.plugins ?? [];
    if (list.length === 0) {
      showToast("info", "该模块未声明依赖插件。");
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
          `索引未找到依赖插件：${pid}（source=${src}）\n\n建议：先确认该插件确实存在于该源，或切换到正确的源再同步。`,
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
    }
    showToast("success", `模块已应用：${row.id}（插槽位置可在「插槽顺序」里调整）`);
  }

  async function onApplyProfileEntry(row: PluginMarketEntryDto): Promise<void> {
    const prof = (row as { profile?: unknown }).profile as
      | {
          plugins: { id: string; version?: string | null; source?: string | null }[];
          backends?: Record<string, unknown> | null;
          predeclaredPermissions?: string[] | null;
        }
      | null
      | undefined;
    if (!prof) {
      showToast("error", "该条目未提供 profile 声明体。");
      return;
    }
    const planLines: string[] = [];
    const deps = (prof.plugins ?? []).map((x) => x.id).filter(Boolean);
    const sources = [
      ...new Set((prof.plugins ?? []).map((x) => normalizeProfileSource(x.source ?? null))),
    ];
    planLines.push(`类型：Profile（无代码）`);
    planLines.push(`条目：${row.id}`);
    if (sources.length) planLines.push(`将同步索引源：${sources.join("、")}`);
    if (deps.length) planLines.push(`将安装依赖插件：${deps.join("、")}`);
    const changes = summarizeOverrideBackends(prof.backends ?? null);
    if (changes.length) {
      planLines.push(`将写入后端覆盖（会话级）：`);
      for (const x of changes) planLines.push(`- ${x}`);
    }
    const ok = await requestApplyPreflight(`应用 Profile：${row.id}`, planLines);
    if (!ok) return;
    saveCurrentSessionOverrideForRollback("profile", row.id);

    const pre = (prof.predeclaredPermissions ?? []).map((s) => String(s).trim()).filter(Boolean);
    if (pre.length > 0) {
      showToast("info", `该 Profile 预声明权限：${pre.join("、")}`);
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
          `索引未找到依赖插件：${pid}（source=${src}）\n\n建议：先确认该插件确实存在于该源，或切换到正确的源再同步。`,
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
    showToast("success", `Profile 已应用：${row.id}（插槽位置可在「插槽顺序」里调整）`);
  }

  async function copyReviewTemplate(params: {
    pluginId: string;
    pubkeyId?: string | null;
    version?: string | null;
  }): Promise<void> {
    const text = buildReviewJsonTemplate(params);
    try {
      if (!navigator.clipboard?.writeText) throw new Error("clipboard API unavailable");
      await navigator.clipboard.writeText(text);
      showToast("success", "已复制评价模板（JSON）。");
    } catch (e) {
      showToast("error", e instanceof Error ? e.message : String(e));
    }
  }

  async function bootstrapMarketData(): Promise<void> {
    await Promise.all([
      pluginStore.loadCachedPluginMarket(),
      refreshPluginReviewsIndex(),
      loadMarketSourcesForPanel(),
      refreshPermissionTokenInfos(),
    ]);
  }

  if (options?.loadOnMount !== false) {
    onMounted(() => {
      void bootstrapMarketData();
    });
  }

  return {
    pluginStore,
    marketSourceSelected,
    marketSources,
    marketEntryTab,
    permConsentVisible,
    permConsentTitle,
    permConsentPerms,
    permConsentSelected,
    permConsentTrustSummary,
    permTokenInfoLoading,
    permTokenInfoMap,
    preflightVisible,
    preflightTitle,
    preflightLines,
    pluginReviewsLoading,
    pluginReviewsIndex,
    pluginReviewsErr,
    marketRowsFiltered,
    marketPageSize,
    marketPage,
    marketTotalPages,
    marketRowsPaged,
    marketPickedVersion,
    marketEntryType,
    marketVersionsForRow,
    marketPickedVersionForRow,
    summarizeOverrideBackends,
    ratingTextForPluginId,
    ratingTextForPluginPubkey,
    ratingStarsForPluginId,
    ratingStarsForPluginPubkey,
    getRecentReviews,
    renderReviewLine,
    riskLabel,
    riskClass,
    onPermConsentCancel,
    onPermConsentConfirm,
    onPreflightCancel,
    onPreflightConfirm,
    setPermConsentAll,
    onSyncMarketIndex,
    syncPluginReviewsIndexNow,
    openPluginReviewsContribution,
    copyReviewTemplate,
    onInstallMarketEntry,
    onInstallMarketVersion,
    onUpdateMarketEntry,
    onApplyModuleEntry,
    onApplyProfileEntry,
    bootstrapMarketData,
  };
}
