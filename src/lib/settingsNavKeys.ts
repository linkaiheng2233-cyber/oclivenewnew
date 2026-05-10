/**
 * 设置中心左侧导航的稳定 id（与 `handoff/SETTINGS_IA_TRUTH_TABLE.md` 一致）。
 * i18n：`settings.nav.items.<settingsNavLabelKey(id)>`。
 */
export const SETTINGS_NAV_CAT = {
  behavior: "settings.cat.behavior",
  models: "settings.cat.models",
  data: "settings.cat.data",
  plugins: "settings.cat.plugins",
  advanced: "settings.cat.advanced",
  system: "settings.cat.system",
} as const;

export type SettingsNavCatId = (typeof SETTINGS_NAV_CAT)[keyof typeof SETTINGS_NAV_CAT];

export const SETTINGS_NAV = {
  generalOverview: "settings.general.overview",
  generalLanguage: "settings.general.language",
  shortcutsManage: "settings.shortcuts.manage",
  generalDefaultModel: "settings.general.defaultModel",
  modelsCloud: "settings.models.cloud",
  modelsOllama: "settings.models.ollama",
  dataRoles: "settings.data.roles",
  dataExpertModels: "settings.data.expertModels",
  dataExpertWorkbench: "settings.data.expertWorkbench",
  pluginsDirectory: "settings.plugins.directory",
  pluginsLinkInstalled: "settings.plugins.linkInstalled",
  pluginsLinkSlots: "settings.plugins.linkSlots",
  pluginsLinkBackends: "settings.plugins.linkBackends",
  pluginsV2Hub: "settings.plugins.v2Hub",
  marketBrowse: "settings.market.browse",
  marketBrowseV2: "settings.market.browseV2",
  securityHost: "settings.security.host",
  advancedExperimental: "settings.advanced.experimental",
  advancedEmbed: "settings.advanced.embed",
  systemDeveloper: "settings.system.developer",
  diagnosticsDebug: "settings.diagnostics.debug",
  diagnosticsAgent: "settings.diagnostics.agent",
} as const;

export type SettingsNavId = (typeof SETTINGS_NAV)[keyof typeof SETTINGS_NAV];

export const ALL_SETTINGS_NAV_IDS: readonly SettingsNavId[] = Object.values(SETTINGS_NAV);

/**
 * 沉浸模式下「设置中心开发者总闸」关闭时从侧栏隐藏的项（V2 / 高阶；V1 仍保留在设置中）。
 * 与 `uiStore.settingsDeveloperMaster` 一致，不再与插件市场索引的 `developerMode` 混用。
 */
export const SETTINGS_DEVELOPER_GATED_NAV_IDS: readonly SettingsNavId[] = [
  SETTINGS_NAV.dataExpertModels,
  SETTINGS_NAV.dataExpertWorkbench,
  SETTINGS_NAV.pluginsV2Hub,
  SETTINGS_NAV.advancedExperimental,
  SETTINGS_NAV.systemDeveloper,
  SETTINGS_NAV.diagnosticsAgent,
  SETTINGS_NAV.marketBrowseV2,
];

/** 别名：与 `SETTINGS_DEVELOPER_GATED_NAV_IDS` 相同，语义更直白。 */
export const SETTINGS_CENTER_V2_ONLY_NAV_IDS = SETTINGS_DEVELOPER_GATED_NAV_IDS;

export function isDeveloperGatedNavId(id: SettingsNavId): boolean {
  return (SETTINGS_DEVELOPER_GATED_NAV_IDS as readonly string[]).includes(id);
}

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
  { id: SETTINGS_NAV.shortcutsManage, depth: 1, tier: "L2", visibility: "immersive" },
  { id: SETTINGS_NAV_CAT.behavior, depth: 0, isGroupLabel: true, visibility: "always" },
  { id: SETTINGS_NAV.generalDefaultModel, depth: 1, tier: "L2", visibility: "always" },
  { id: SETTINGS_NAV_CAT.models, depth: 0, isGroupLabel: true, visibility: "always" },
  { id: SETTINGS_NAV.modelsCloud, depth: 1, tier: "L4", visibility: "always" },
  { id: SETTINGS_NAV.modelsOllama, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV_CAT.data, depth: 0, isGroupLabel: true, visibility: "immersive" },
  { id: SETTINGS_NAV.dataRoles, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.dataExpertModels, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.dataExpertWorkbench, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV_CAT.plugins, depth: 0, isGroupLabel: true, visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsDirectory, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsLinkInstalled, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsLinkSlots, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsLinkBackends, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.pluginsV2Hub, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.marketBrowse, depth: 0, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV.marketBrowseV2, depth: 0, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.securityHost, depth: 0, tier: "L4", visibility: "always" },
  { id: SETTINGS_NAV_CAT.advanced, depth: 0, isGroupLabel: true, visibility: "immersive" },
  { id: SETTINGS_NAV.advancedExperimental, depth: 1, tier: "L4", visibility: "immersive" },
  { id: SETTINGS_NAV.advancedEmbed, depth: 1, tier: "L3", visibility: "immersive" },
  { id: SETTINGS_NAV_CAT.system, depth: 0, isGroupLabel: true, visibility: "immersive" },
  { id: SETTINGS_NAV.systemDeveloper, depth: 1, tier: "L4", visibility: "immersive" },
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

/**
 * 设置右栏「内容分级」与 `SettingsView` + `SettingsTierSection` 一致（L4 默认折叠，展开需确认）。
 *
 * | 侧栏 id | 右栏分区（自上而下） |
 * |---------|----------------------|
 * | generalOverview | L1 概览与纯聊边界 → L4 全局「恢复默认宿主偏好」 |
 * | generalLanguage | L1 语言 |
 * | shortcutsManage | 快捷键管理（说明 + 全局绑定编辑） |
 * | generalDefaultModel | L2 默认对话模型（与撰写区同步） |
 * | modelsCloud | L4 云端信任 / QuickSetup / 打开后端 |
 * | modelsOllama | L3 说明 → L4 打开本机模型 |
 * | dataExpertModels | L3 专家模型设置（生效图、工作台、恢复包默认） |
 * | dataRoles | L3 角色与数据（切换 / 包目录 / 编写器说明） |
 * | pluginsDirectory | L3 目录插件设置插槽 |
 * | pluginsLink* / pluginsV2Hub / marketBrowse | L3 说明 → L4 本页嵌入插件管理 / 市场 |
 * | securityHost / advancedExperimental | L4 安全与实验 |
 * | advancedEmbed | L3 settings.advanced 扩展区 |
 * | systemDeveloper | L4 市场开发者模式与第三方索引源（侧栏项受设置中心开发者总闸） |
 * | diagnosticsDebug | L2 说明 → L4 本页嵌入调试 |
 * | diagnosticsAgent | L3 说明 → L4 嵌入后端 Agent 调试 |
 */
