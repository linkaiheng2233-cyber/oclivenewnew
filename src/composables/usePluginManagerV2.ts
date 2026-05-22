import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { patchSlotRegistryBackend } from "../lib/archGraphSlotBackend";
import {
  SLOT_BACKEND_OPTIONS,
  SLOT_TYPE_LABEL_KEYS,
  sortedSlotRegistryEntries,
  type SlotRegistryEntry,
} from "../lib/slotRegistry";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import {
  clearSessionSlotOverride,
  saveRoleSlotRegistry,
  setRemoteLifeEnabled,
  setSessionSlotOverride,
} from "../utils/tauri-api";

export type V2TypeKey = "builtin" | "remote" | "directory";

export interface PluginV2CardItem {
  id: string;
  slotKey: string;
  title: string;
  /** slot_registry `type` */
  module: string;
  moduleLabel: string;
  type: V2TypeKey;
  status: "enabled" | "disabled" | "needs_config";
  sessionOverridden: boolean;
  packBackend: string;
  effectiveBackend: string;
  description: string;
  uiTemplate: "slot-registry" | "switch-toggle" | "endpoint-config";
  schema: Record<string, unknown>;
}

export interface PluginV2CategoryItem {
  id: string;
  label: string;
  count: number;
}

function normalizeType(backend: string): V2TypeKey {
  if (backend === "remote") return "remote";
  if (backend === "directory") return "directory";
  return "builtin";
}

function moduleLabelFor(
  t: (key: string) => string,
  te: (key: string) => boolean,
  slotType: string,
): string {
  const key = `pluginTerms.module.${slotType}`;
  return te(key) ? t(key) : slotType;
}

function buildRegistryCard(
  slotKey: string,
  entry: SlotRegistryEntry,
  packEntry: SlotRegistryEntry | undefined,
  sessionOverridden: boolean,
  directoryOptions: { value: string; label: string }[],
  t: (key: string, values?: Record<string, unknown>) => string,
  te: (key: string) => boolean,
): PluginV2CardItem {
  const type = normalizeType(entry.backend);
  const primaryPlugin = entry.plugin?.trim() ?? "";
  const needsConfig = type === "directory" && !primaryPlugin;
  const labelKey = SLOT_TYPE_LABEL_KEYS[entry.type];
  const options = (SLOT_BACKEND_OPTIONS[entry.type] ?? ["builtin"]).map((v) => ({
    value: v,
    label: v,
  }));

  return {
    id: slotKey,
    slotKey,
    title: entry.label?.trim() || slotKey,
    module: entry.type,
    moduleLabel: moduleLabelFor(t, te, entry.type),
    type,
    status: needsConfig ? "needs_config" : "enabled",
    sessionOverridden,
    packBackend: packEntry?.backend ?? entry.backend,
    effectiveBackend: entry.backend,
    description: labelKey ? t(labelKey) : entry.type,
    uiTemplate: "slot-registry",
    schema: {
      slotKey,
      backend: entry.backend,
      packBackend: packEntry?.backend ?? entry.backend,
      sessionOverridden,
      options,
      directoryOptions,
      directoryId: primaryPlugin,
    },
  };
}

export function usePluginManagerV2() {
  const { t, te, locale } = useI18n();
  const roleStore = useRoleStore();
  const pluginStore = usePluginStore();

  const searchKeyword = ref("");
  const selectedCategory = ref("all");
  const selectedCardId = ref("");

  const directoryOptions = computed(() =>
    pluginStore.catalog.map((c) => ({ value: c.id, label: c.id })),
  );

  const cards = computed<PluginV2CardItem[]>(() => {
    void locale.value;
    const effective = roleStore.roleInfo.slotRegistryEffective;
    const pack = roleStore.roleInfo.slotRegistryPack;
    const overridden = new Set(roleStore.roleInfo.slotSessionOverriddenKeys);

    if (effective && Object.keys(effective).length > 0) {
      const rows = sortedSlotRegistryEntries(effective).map(([slotKey, entry]) =>
        buildRegistryCard(
          slotKey,
          entry,
          pack?.[slotKey],
          overridden.has(slotKey),
          directoryOptions.value,
          t,
          te,
        ),
      );
      if (roleStore.roleInfo.remoteLifeEnabled !== undefined) {
        rows.push({
          id: "complex-remote-life",
          slotKey: "",
          title: t("pluginManager.cards.complexSwitch.title"),
          module: "complex_emotion",
          moduleLabel: moduleLabelFor(t, te, "complex_emotion"),
          type: "remote",
          status: roleStore.roleInfo.remoteLifeEnabled ? "enabled" : "disabled",
          sessionOverridden: false,
          packBackend: "",
          effectiveBackend: roleStore.roleInfo.remoteLifeEnabled ? "remote" : "builtin",
          description: t("pluginManager.cards.complexSwitch.description"),
          uiTemplate: "switch-toggle",
          schema: {
            checked: roleStore.roleInfo.remoteLifeEnabled,
            label: t("pluginManager.cards.complexSwitch.label"),
            hint: t("pluginManager.cards.complexSwitch.hint"),
          },
        });
      }
      return rows;
    }

    return [];
  });

  const categories = computed<PluginV2CategoryItem[]>(() => {
    void locale.value;
    const rows = cards.value;
    const countBy = (fn: (x: PluginV2CardItem) => boolean) =>
      rows.filter((x) => fn(x)).length;
    const types = [...new Set(rows.map((x) => x.module))];
    const base: PluginV2CategoryItem[] = [
      { id: "all", label: t("pluginTerms.category.all"), count: rows.length },
    ];
    for (const mod of types) {
      if (!mod) continue;
      base.push({
        id: `module:${mod}`,
        label: moduleLabelFor(t, te, mod),
        count: countBy((x) => x.module === mod),
      });
    }
    return base;
  });

  const filteredCards = computed(() => {
    const keyword = searchKeyword.value.trim().toLowerCase();
    return cards.value.filter((item) => {
      if (selectedCategory.value !== "all") {
        const [, val] = selectedCategory.value.split(":");
        if (item.module !== val) return false;
      }
      if (!keyword) return true;
      const hay = `${item.title} ${item.description} ${item.moduleLabel} ${item.slotKey}`.toLowerCase();
      return hay.includes(keyword);
    });
  });

  const selectedCard = computed(() =>
    filteredCards.value.find((x) => x.id === selectedCardId.value) ?? null,
  );

  watch(
    filteredCards,
    (rows) => {
      if (rows.length === 0) {
        selectedCardId.value = "";
        return;
      }
      if (!rows.some((x) => x.id === selectedCardId.value)) {
        selectedCardId.value = rows[0].id;
      }
    },
    { immediate: true },
  );

  async function applyCardChange(
    item: PluginV2CardItem,
    payload: Record<string, unknown>,
  ): Promise<string> {
    if (item.uiTemplate === "switch-toggle") {
      const info = await setRemoteLifeEnabled(
        roleStore.currentRoleId,
        Boolean(payload.enabled),
      );
      roleStore.applyRoleInfo(info);
      return t("pluginManager.apply.remoteLifeUpdated");
    }

    if (item.uiTemplate === "slot-registry") {
      const slotKey = String(payload.slotKey ?? item.slotKey);
      const backend = String(payload.backend ?? "");
      const applyMode = payload.applyMode as "session" | "pack";
      const directoryId = payload.directoryId as string | null | undefined;

      if (applyMode === "pack") {
        const pack = roleStore.roleInfo.slotRegistryPack;
        if (!pack?.[slotKey]) {
          throw new Error(t("pluginWorkbench.graph.connectUnknownPort"));
        }
        let next = patchSlotRegistryBackend(pack, slotKey, backend);
        if (backend === "directory" && directoryId !== undefined) {
          next = {
            ...next,
            [slotKey]: { ...next[slotKey], plugin: directoryId, plugins: directoryId ? [directoryId] : [] },
          };
        }
        let info = await saveRoleSlotRegistry(roleStore.currentRoleId, next);
        roleStore.applyRoleInfo(info);
        info = await clearSessionSlotOverride(roleStore.currentRoleId, slotKey);
        roleStore.applyRoleInfo(info);
        return t("pluginWorkbench.graph.applyPackDone");
      }

      const info = await setSessionSlotOverride(roleStore.currentRoleId, slotKey, {
        backend,
        plugin: backend === "directory" ? (directoryId ?? null) : null,
      });
      roleStore.applyRoleInfo(info);
      return t("pluginWorkbench.graph.applySessionDone");
    }

    throw new Error(t("pluginManager.apply.unsupported"));
  }

  return {
    searchKeyword,
    selectedCategory,
    selectedCardId,
    categories,
    filteredCards,
    selectedCard,
    applyCardChange,
    hasBlueprint: computed(() => {
      const eff = roleStore.roleInfo.slotRegistryEffective;
      return eff != null && Object.keys(eff).length > 0;
    }),
  };
}
