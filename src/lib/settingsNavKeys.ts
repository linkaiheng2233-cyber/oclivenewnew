/**
 * 设置中心左侧导航的稳定 id（与 `handoff/SETTINGS_IA_TRUTH_TABLE.md` 一致）。
 * i18n：`settings.nav.items.<settingsNavLabelKey(id)>`。
 */
export const SETTINGS_NAV_CAT = {
  models: "settings.cat.models",
  plugins: "settings.cat.plugins",
  advanced: "settings.cat.advanced",
} as const;

export type SettingsNavCatId = (typeof SETTINGS_NAV_CAT)[keyof typeof SETTINGS_NAV_CAT];

export const SETTINGS_NAV = {
  generalOverview: "settings.general.overview",
  generalLanguage: "settings.general.language",
  shortcutsMain: "settings.shortcuts.main",
  modelsCloud: "settings.models.cloud",
  modelsOllama: "settings.models.ollama",
  modelsExpert: "settings.models.expert",
  pluginsDirectory: "settings.plugins.directory",
  pluginsHotkeys: "settings.plugins.hotkeys",
  pluginsLinkInstalled: "settings.plugins.linkInstalled",
  pluginsLinkSlots: "settings.plugins.linkSlots",
  pluginsLinkBackends: "settings.plugins.linkBackends",
  pluginsV2Hub: "settings.plugins.v2Hub",
  marketBrowse: "settings.market.browse",
  securityHost: "settings.security.host",
  advancedExperimental: "settings.advanced.experimental",
  advancedEmbed: "settings.advanced.embed",
  advancedMarketSources: "settings.advanced.marketSources",
  diagnosticsDebug: "settings.diagnostics.debug",
  diagnosticsAgent: "settings.diagnostics.agent",
} as const;

export type SettingsNavId = (typeof SETTINGS_NAV)[keyof typeof SETTINGS_NAV];

export const ALL_SETTINGS_NAV_IDS: readonly SettingsNavId[] = Object.values(SETTINGS_NAV);

export type SettingsNavAnyId = SettingsNavId | SettingsNavCatId;

/** 供 vue-i18n：`t('settings.nav.items.' + settingsNavLabelKey(id))` */
export function settingsNavLabelKey(id: SettingsNavAnyId): string {
  const strip = id.replace(/^settings\./, "");
  return strip
    .split(".")
    .map((seg, i) => (i === 0 ? seg : seg[0]!.toUpperCase() + seg.slice(1)))
    .join("");
}

export type SettingsTier = "L1" | "L2" | "L3" | "L4";

export type SettingsNavVisibility = "always" | "immersive";

export interface SettingsNavRow {
  id: SettingsNavAnyId;
  /** 左侧缩进层级：0 = 分组顶；1 = 子项 */
  depth: 0 | 1;
  /** 非可选行：分组标题，不切换右栏 */
  isGroupLabel?: boolean;
  tier?: SettingsTier;
  visibility: SettingsNavVisibility;
}

function rowVisible(immersive: boolean, visibility: SettingsNavVisibility): boolean {
  return visibility === "always" || (visibility === "immersive" && immersive);
}

/**
 * 按纯聊 / 沉浸裁剪侧栏行；分组标题仅在其下存在可见子项时出现。
 */
export function filterSettingsNavRows(
  immersive: boolean,
  rows: readonly SettingsNavRow[],
): SettingsNavRow[] {
  const vis = (r: SettingsNavRow) => rowVisible(immersive, r.visibility);
  const out: SettingsNavRow[] = [];
  for (let i = 0; i < rows.length; i += 1) {
    const r = rows[i]!;
    if (r.isGroupLabel) {
      let anyChild = false;
      for (let j = i + 1; j < rows.length; j += 1) {
        const n = rows[j]!;
        if (n.depth <= r.depth) break;
        if (!n.isGroupLabel && vis(n)) anyChild = true;
      }
      if (anyChild) out.push(r);
      continue;
    }
    if (vis(r)) out.push(r);
  }
  return out;
}

/** 侧栏行顺序即 IA 顺序 */
export const SETTINGS_NAV_ROWS: readonly SettingsNavRow[] = [
  { id: SETTINGS_NAV.generalOverview, depth: 0, tier: "L1", visibility: "always" },
  { id: SETTINGS_NAV.generalLanguage, depth: 1, tier: "L1", visibility: "always" },
  { id: SETTINGS_NAV.shortcutsMain, depth: 1, tier: "L1", visibility: "immersive" },
  { id: SETTINGS_NAV_CAT.models, depth: 0, isGroupLabel: true, visibility: "always" },
  { id: SETTINGS_NAV.modelsCloud, depth: 1, tier: "L4", visibility: "always" },
  { id: SETTINGS_NAV.modelsOllama, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.modelsExpert, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV_CAT.plugins, depth: 0, isGroupLabel: true, visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsDirectory, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsHotkeys, depth: 1, tier: "L2", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsLinkInstalled, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsLinkSlots, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsLinkBackends, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsV2Hub, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.marketBrowse, depth: 0, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.securityHost, depth: 0, tier: "L4", visibility: "always" },
  { id: SETTINGS_NAV_CAT.advanced, depth: 0, isGroupLabel: true, visibility: "immersive" },
  { id: SETTINGS_NAV.advancedExperimental, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.advancedEmbed, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.advancedMarketSources, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.diagnosticsDebug, depth: 0, tier: "L2", visibility: "immersive" },
  { id: SETTINGS_NAV.diagnosticsAgent, depth: 0, tier: "L4", visibility: "immersive" },
];

export function firstSelectableSettingsNavId(
  immersive: boolean,
  rows: readonly SettingsNavRow[] = SETTINGS_NAV_ROWS,
): SettingsNavId {
  const filtered = filterSettingsNavRows(immersive, rows);
  for (const r of filtered) {
    if (!r.isGroupLabel && ALL_SETTINGS_NAV_IDS.includes(r.id as SettingsNavId)) {
      return r.id as SettingsNavId;
    }
  }
  return SETTINGS_NAV.generalOverview;
}
