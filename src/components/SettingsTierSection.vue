<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { confirm } from "@tauri-apps/api/dialog";
import { isTauriWebview } from "../utils/isTauriWebview";
import type { SettingsTier } from "../lib/settingsNavKeys";
import { settingsTierBadge, settingsTierDescription } from "../lib/settingsNavCopy";

const props = withDefaults(
  defineProps<{
    tier: SettingsTier;
    /** 设置窗关闭时递增，用于将 L4 恢复为默认折叠 */
    resetKey?: number;
  }>(),
  { resetKey: 0 },
);

const { t } = useI18n();

const expanded = ref(props.tier !== "L4");

function syncExpandedFromTier(): void {
  expanded.value = props.tier !== "L4";
}

watch(
  () => props.resetKey,
  () => {
    syncExpandedFromTier();
  },
);

watch(
  () => props.tier,
  () => {
    syncExpandedFromTier();
  },
);

const blockHeading = computed(() => String(t(`settings.tiersUi.blockHeading.${props.tier}`)));
const badge = computed(() => settingsTierBadge(props.tier));
const description = computed(() => settingsTierDescription(props.tier));

async function requestExpand(): Promise<void> {
  if (props.tier !== "L4") {
    expanded.value = true;
    return;
  }
  const title = String(t("settings.tiersUi.confirmExpandTitle"));
  const message = String(t("settings.tiersUi.confirmExpandMessage"));
  let ok = true;
  if (isTauriWebview()) {
    try {
      ok = await confirm(message, {
        title,
        type: "warning",
        okLabel: String(t("settings.tiersUi.confirmExpandOk")),
        cancelLabel: String(t("common.cancel")),
      });
    } catch {
      ok = window.confirm(`${title}\n\n${message}`);
    }
  } else {
    ok = window.confirm(`${title}\n\n${message}`);
  }
  if (ok) expanded.value = true;
}

function collapse(): void {
  expanded.value = false;
}

const showL4Chrome = computed(() => props.tier === "L4");
</script>

<template>
  <section class="sts" :class="`sts--tier-${tier}`">
    <header class="sts-head">
      <div class="sts-head-main">
        <h3 class="sts-title">
          <abbr class="sts-badge" :title="description">{{ badge }}</abbr>
          {{ blockHeading }}
        </h3>
        <p v-if="showL4Chrome && !expanded" class="sts-muted">
          {{ t("settings.tiersUi.l4CollapsedHint") }}
        </p>
      </div>
      <div v-if="showL4Chrome" class="sts-actions">
        <button
          v-if="!expanded"
          type="button"
          class="sts-btn"
          @click="requestExpand"
        >
          {{ t("settings.tiersUi.expandButton") }}
        </button>
        <button
          v-else
          type="button"
          class="sts-btn sts-btn--ghost"
          @click="collapse"
        >
          {{ t("settings.tiersUi.collapseButton") }}
        </button>
      </div>
    </header>
    <div v-show="!showL4Chrome || expanded" class="sts-body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.sts {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 12px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.sts--tier-L1 {
  border-color: color-mix(in srgb, var(--border-light) 88%, var(--text-secondary) 12%);
}
.sts--tier-L4 {
  border-color: color-mix(in srgb, var(--border-light) 70%, var(--text-secondary) 22%);
}
.sts-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.sts-head-main {
  min-width: 0;
  flex: 1;
}
.sts-title {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
  line-height: 1.35;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  color: var(--text-primary);
}
.sts-badge {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 700;
  text-decoration: none;
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
  cursor: help;
  letter-spacing: 0.02em;
}
.sts-muted {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.sts-actions {
  flex-shrink: 0;
  padding-top: 2px;
}
.sts-btn {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
  white-space: nowrap;
}
.sts-btn--ghost {
  background: transparent;
}
.sts-btn:hover {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 35%, var(--border-light));
}
.sts-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.sts-body :deep(.sv-section),
.sts-body :deep(.sv-cloud-section) {
  gap: 8px;
}
</style>
