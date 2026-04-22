<script setup lang="ts">
import { watch } from "vue";
import { useAppToast } from "../composables/useAppToast";
import { usePluginStore } from "../stores/pluginStore";
import type { PluginMarketEntryDto } from "../utils/tauri-api";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  openManage: [];
}>();

const pluginStore = usePluginStore();
const { showToast } = useAppToast();

watch(
  () => props.visible,
  (v) => {
    if (v) {
      void pluginStore.loadCachedPluginMarket();
    }
  },
);

async function onSyncMarketIndex() {
  try {
    await pluginStore.syncPluginMarket();
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
  try {
    await pluginStore.installFromPluginMarket(row.id, row.git);
    showToast("success", `已安装 ${row.id}，建议到「插件管理」里启用并配置插槽。`);
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

const defaultRepoUrl =
  "https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins";

function resolvedRepoUrl(): string {
  const src = pluginStore.pluginMarketSnapshot?.source?.trim() ?? "";
  if (!src) return defaultRepoUrl;
  if (src.startsWith("http://") || src.startsWith("https://")) {
    return src;
  }
  return defaultRepoUrl;
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="om-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="OCLive Manager（插件市场）"
      @click.self="emit('close')"
    >
      <div class="om-dialog" @click.stop>
        <header class="om-head">
          <h2 class="om-title">OCLive Manager</h2>
          <button type="button" class="om-close" aria-label="关闭" @click="emit('close')">
            ×
          </button>
        </header>

        <p class="om-sub">
          类似 ComfyUI Manager：这里负责
          <strong>插件索引同步、安装、更新</strong>；安装后请到
          <button type="button" class="om-link-btn" @click="emit('openManage')">
            插件管理
          </button>
          完成启用与插槽配置。
        </p>
        <p class="om-sub">
          共享仓库：
          <a :href="resolvedRepoUrl()" target="_blank" rel="noopener noreferrer">
            {{ resolvedRepoUrl() }}
          </a>
          。上传你的插件后，向该仓库 `plugins.json` 提交 PR，即可被他人检索到。
        </p>

        <section class="om-section">
          <div class="om-section-head">
            <h3 class="om-h3">社区索引</h3>
            <div class="om-actions">
              <button
                type="button"
                class="om-btn secondary"
                :disabled="pluginStore.pluginMarketSyncing"
                @click="onSyncMarketIndex"
              >
                {{ pluginStore.pluginMarketSyncing ? "同步中…" : "同步在线索引" }}
              </button>
            </div>
          </div>

          <p v-if="pluginStore.pluginMarketError" class="om-err">
            {{ pluginStore.pluginMarketError }}
          </p>
          <p v-else-if="pluginStore.pluginMarketSnapshot?.warning" class="om-hint">
            {{ pluginStore.pluginMarketSnapshot.warning }}
          </p>
          <p v-if="pluginStore.pluginMarketSnapshot?.offlineMode" class="om-hint">
            当前为离线模式（使用本地缓存索引）。
          </p>

          <p
            v-if="
              !pluginStore.pluginMarketSnapshot?.plugins?.length &&
              !pluginStore.pluginMarketError
            "
            class="om-muted"
          >
            尚无索引数据，请点击「同步在线索引」。
          </p>

          <ul
            v-else-if="(pluginStore.pluginMarketSnapshot?.plugins?.length ?? 0) > 0"
            class="om-market-list"
          >
            <li
              v-for="row in pluginStore.pluginMarketSnapshot!.plugins"
              :key="row.id"
              class="om-market-li"
            >
              <div class="om-market-main">
                <strong>{{ row.id }}</strong>
                <span class="om-muted"> · {{ row.name }} · v{{ row.version }}</span>
                <p v-if="row.description" class="om-market-desc">{{ row.description }}</p>
                <p
                  v-if="(row.missingDependencies ?? []).length"
                  class="om-err om-market-deps"
                >
                  依赖缺失：{{ row.missingDependencies.join("、") }}
                </p>
              </div>
              <div class="om-market-actions">
                <button
                  v-if="!row.installed"
                  type="button"
                  class="om-btn secondary"
                  @click="onInstallMarketEntry(row)"
                >
                  安装
                </button>
                <template v-else>
                  <span v-if="row.hasUpdate" class="om-badge">可更新</span>
                  <span v-else class="om-muted">已安装</span>
                  <button
                    v-if="row.hasUpdate"
                    type="button"
                    class="om-btn secondary"
                    @click="onUpdateMarketEntry(row)"
                  >
                    更新
                  </button>
                </template>
              </div>
            </li>
          </ul>
        </section>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.om-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10045;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 45%, transparent));
}
.om-dialog {
  width: min(960px, 100%);
  max-height: min(92vh, 900px);
  overflow: auto;
  padding: 14px 18px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.om-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.om-title {
  margin: 0;
  font-size: 20px;
}
.om-close {
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
.om-close:hover {
  background: color-mix(in srgb, var(--border-light) 60%, transparent);
}
.om-sub {
  margin: 6px 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}
.om-link-btn {
  border: none;
  background: transparent;
  color: var(--text-accent);
  cursor: pointer;
  padding: 0 2px;
  font: inherit;
  text-decoration: underline;
}
.om-section {
  margin-top: 12px;
}
.om-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.om-actions {
  display: flex;
  gap: 8px;
}
.om-h3 {
  margin: 0;
  font-size: 16px;
}
.om-btn {
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
}
.om-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.om-market-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: min(62vh, 560px);
  overflow: auto;
}
.om-market-li {
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
.om-market-main {
  flex: 1 1 200px;
  min-width: 0;
}
.om-market-desc {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.om-market-deps {
  margin: 6px 0 0;
}
.om-market-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.om-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent, #6c9) 18%, transparent);
  color: var(--text-accent);
  font-size: 11px;
}
.om-hint,
.om-muted {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.om-err {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--danger, #c0392b);
}
</style>
