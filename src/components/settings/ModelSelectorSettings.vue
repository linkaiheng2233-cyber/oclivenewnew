<script setup lang="ts">
import { computed, ref, unref, watch } from "vue";
import { useI18n } from "vue-i18n";
import HostModelPickRow from "../HostModelPickRow.vue";
import { useAppToast } from "../../composables/useAppToast";
import { useHostModelPick } from "../../composables/useHostModelPick";

const props = defineProps<{
  /** 当前侧栏选中本页时为 true，用于按需刷新列表 */
  active: boolean;
}>();

const emit = defineEmits<{
  /** 侧栏跳转到本机模型 / L4 管理入口 */
  openLocalModels: [];
}>();

const { t } = useI18n();
const { showToast } = useAppToast();
const pick = useHostModelPick();
const booting = ref(false);
const loadError = ref<string | null>(null);
const applying = ref(false);

const showEmptyHint = computed(() => {
  if (booting.value || loadError.value) return false;
  const ollama = unref(pick.ollamaNames);
  const cloudOpts = unref(pick.cloudSelectOptions);
  return !ollama.length && !cloudOpts.length;
});

const cloudSummary = computed(() => {
  const pub = unref(pick.cloudPub);
  if (!pub?.baseUrl?.trim()) return "";
  const keyOk = pub.hasApiKey === true;
  return String(
    t("settings.modelHub.cloudSummary", {
      url: pub.baseUrl.trim(),
      key: keyOk ? t("settings.modelHub.cloudKeyPresent") : t("settings.modelHub.cloudKeyMissing"),
    }),
  );
});

async function runBootstrap(): Promise<void> {
  loadError.value = null;
  booting.value = true;
  try {
    await pick.bootstrap();
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
    showToast("error", loadError.value);
  } finally {
    booting.value = false;
  }
}

watch(
  () => props.active,
  (active) => {
    if (active) void runBootstrap();
  },
  { immediate: true },
);

async function applyModel(id: string): Promise<void> {
  const m = id.trim();
  if (!m || m === unref(pick.modelId)) return;
  applying.value = true;
  try {
    await pick.applyChatModelId(m);
    showToast("success", String(t("settings.modelHub.appliedToast", { id: m })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    applying.value = false;
  }
}

function isCurrent(id: string): boolean {
  return id.trim() === unref(pick.modelId).trim();
}
</script>

<template>
  <div class="mss">
    <p v-if="booting" class="mss-status">{{ t("settings.modelSelector.loading") }}</p>
    <p v-else-if="loadError" class="mss-err">
      {{ loadError }}
      <button type="button" class="mss-retry" @click="runBootstrap">
        {{ t("settings.modelSelector.retry") }}
      </button>
    </p>
    <template v-else>
      <section class="mss-block" aria-labelledby="mss-default-h">
        <h3 id="mss-default-h" class="mss-h">{{ t("settings.modelHub.defaultTitle") }}</h3>
        <p class="mss-current">
          <code class="mss-code">{{ pick.modelId || "—" }}</code>
        </p>
        <HostModelPickRow select-id="oclive-settings-default-model" :show-gear="false" :disabled="applying" />
      </section>

      <section v-if="pick.ollamaNames.length" class="mss-block" aria-labelledby="mss-local-h">
        <h3 id="mss-local-h" class="mss-h">{{ t("settings.modelHub.localTitle") }}</h3>
        <ul class="mss-chip-list">
          <li v-for="n in pick.ollamaNames" :key="'loc-' + n">
            <button
              type="button"
              class="mss-chip"
              :class="{ 'mss-chip--on': isCurrent(n) }"
              :disabled="applying"
              @click="applyModel(n)"
            >
              {{ n }}
            </button>
          </li>
        </ul>
      </section>

      <section v-if="pick.cloudSelectOptions.length || cloudSummary" class="mss-block" aria-labelledby="mss-cloud-h">
        <h3 id="mss-cloud-h" class="mss-h">{{ t("settings.modelHub.cloudTitle") }}</h3>
        <p v-if="cloudSummary" class="mss-muted">{{ cloudSummary }}</p>
        <ul v-if="pick.cloudSelectOptions.length" class="mss-chip-list">
          <li v-for="n in pick.cloudSelectOptions" :key="'cld-' + n">
            <button
              type="button"
              class="mss-chip mss-chip--cloud"
              :class="{ 'mss-chip--on': isCurrent(n) }"
              :disabled="applying"
              @click="applyModel(n)"
            >
              {{ n }}
            </button>
          </li>
        </ul>
      </section>

      <div class="mss-actions">
        <button type="button" class="mss-btn" :disabled="applying" @click="emit('openLocalModels')">
          {{ t("settings.modelHub.openLocalManager") }}
        </button>
      </div>

      <p v-if="showEmptyHint" class="mss-empty">{{ t("settings.modelSelector.emptyHint") }}</p>
      <p class="mss-hint">{{ t("settings.modelSelector.syncHint") }}</p>
    </template>
  </div>
</template>

<style scoped>
.mss {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 640px;
}
.mss-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.mss-h {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
}
.mss-current {
  margin: 0;
}
.mss-code {
  font-size: 13px;
  padding: 4px 8px;
  border-radius: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-light);
}
.mss-status,
.mss-err,
.mss-empty,
.mss-muted {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.mss-err {
  color: var(--text-accent, #b45309);
}
.mss-retry {
  margin-left: 8px;
  padding: 2px 8px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
}
.mss-chip-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.mss-chip {
  padding: 6px 10px;
  font-size: 12px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: inherit;
  cursor: pointer;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
}
.mss-chip:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 45%, var(--border-light));
}
.mss-chip:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.mss-chip--on {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 55%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 12%, var(--bg-primary));
}
.mss-chip--cloud {
  font-family: ui-monospace, monospace;
}
.mss-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.mss-btn {
  padding: 7px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  cursor: pointer;
}
.mss-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.mss-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.45;
  color: var(--text-secondary);
}
</style>
