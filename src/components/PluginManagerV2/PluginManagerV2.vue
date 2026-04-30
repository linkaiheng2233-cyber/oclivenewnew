<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
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
const pluginStore = usePluginStore();
const busy = ref(false);
const rightCollapsed = ref(false);

const slotLabel = (slot: string): string => {
  if (slot === SLOT_SETTINGS_PANEL) return "设置页（插件设置）";
  if (slot === SLOT_SETTINGS_PLUGINS) return "插件管理页内嵌";
  if (slot === SLOT_SETTINGS_ADVANCED) return "设置页（高级扩展区）";
  if (slot === SLOT_SIDEBAR) return "左侧边栏";
  if (slot === SLOT_ROLE_DETAIL) return "角色详情";
  if (slot === SLOT_CHAT_HEADER) return "聊天顶部";
  if (slot === SLOT_CHAT_TOOLBAR) return "聊天工具栏";
  if (slot === SLOT_OVERLAY_FLOATING) return "悬浮层";
  if (slot === SLOT_LAUNCHER_PALETTE) return "启动器（快捷入口）";
  if (slot === SLOT_DEBUG_DOCK) return "调试面板";
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
    showToast("success", "已保存：插槽位置与启用状态已写入配置。");
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
