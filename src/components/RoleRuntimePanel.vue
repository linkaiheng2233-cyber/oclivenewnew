<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginStore } from "../stores/pluginStore";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import { buildRelationDropdownOptions } from "../utils/relationOptions";
import {
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setEvolutionFactor,
  setUserRelation,
} from "../utils/tauri-api";
import HelpHint from "./HelpHint.vue";

const { t } = useI18n();
const roleStore = useRoleStore();
const uiStore = useUiStore();
const pluginStore = usePluginStore();
const localFactor = ref(roleStore.roleInfo.eventImpactFactor);
const busy = ref(false);

const personalitySourceLabel = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? t("roleRuntime.personalityProfile")
    : t("roleRuntime.personalityVector"),
);
const personalitySourceHintParagraphs = computed(() =>
  roleStore.roleInfo.personalitySource === "profile"
    ? [t("roleRuntime.profileHint1"), t("roleRuntime.profileHint2")]
    : [t("roleRuntime.vectorHint1")],
);
const relationRows = computed(() =>
  buildRelationDropdownOptions(
    roleStore.roleInfo.userRelations,
    roleStore.roleInfo.defaultRelation,
  ),
);
watch(
  () => [roleStore.currentRoleId, roleStore.roleInfo.eventImpactFactor] as const,
  () => {
    localFactor.value = roleStore.roleInfo.eventImpactFactor;
  },
);
async function onRelationChange(ev: Event) {
  const next = (ev.target as HTMLSelectElement).value;
  if (next === roleStore.relationSelectValue) return;
  busy.value = true;
  try {
    const perScene = roleStore.roleInfo.identityBinding === "per_scene";
    if (next === OCLIVE_DEFAULT_RELATION_SENTINEL) {
      if (perScene) await roleStore.setManifestDefaultIdentity(uiStore.sceneId);
      else await roleStore.setManifestDefaultIdentity();
    } else if (perScene) {
      await roleStore.setSceneUserRelation(uiStore.sceneId, next);
    } else {
      const info = await setUserRelation(roleStore.currentRoleId, next);
      roleStore.applyRoleInfo(info);
    }
  } finally {
    busy.value = false;
  }
}
async function commitFactor() {
  const v = localFactor.value;
  if (
    !Number.isFinite(v) ||
    v < 0.05 ||
    v > 5 ||
    Math.abs(v - roleStore.roleInfo.eventImpactFactor) < 1e-9
  ) {
    return;
  }
  busy.value = true;
  try {
    await setEvolutionFactor(roleStore.currentRoleId, v);
    await roleStore.refreshRoleInfo();
  } finally {
    busy.value = false;
  }
}
function onFactorEnter(ev: KeyboardEvent) {
  (ev.target as HTMLInputElement).blur();
}
function openBackendsPanel(): void {
  void pluginStore.openPanel("graph");
}
</script>

<template>
  <section class="runtime">
    <div class="meta">
      <p v-if="roleStore.roleInfo.description" class="desc">{{ roleStore.roleInfo.description }}</p>
      <p class="sub">
        {{
          t("roleRuntime.versionAuthor", {
            version: roleStore.roleInfo.version || "—",
            author: roleStore.roleInfo.author || "—",
          })
        }}
      </p>
      <p class="sub personality-source-line">
        <span class="ps-inline">
          {{ t("roleRuntime.personalitySource") }}<strong>{{ personalitySourceLabel }}</strong>
          <HelpHint :paragraphs="personalitySourceHintParagraphs" />
        </span>
      </p>
    </div>
    <div class="runtime-backend-hint">
      <p class="sub">
        {{ t("roleRuntime.backendHintBefore") }}
        <button type="button" class="link-open-backends" @click="openBackendsPanel">
          {{ t("roleRuntime.backendLink") }}
        </button>
        {{ t("roleRuntime.backendHintAfter") }}
      </p>
    </div>
    <template v-if="roleStore.roleInfo.userRelations.length > 0">
      <div class="row">
        <label for="rel-select">{{ t("roleRuntime.relation") }}</label>
        <select
          id="rel-select"
          class="select"
          :disabled="busy"
          :value="roleStore.relationSelectValue"
          @change="onRelationChange"
        >
          <option v-for="r in relationRows" :key="r.id" :value="r.id">{{ r.name || r.id }}</option>
        </select>
      </div>
      <div class="row">
        <label for="evolve-factor">{{ t("roleRuntime.eventImpact") }}</label>
        <input
          id="evolve-factor"
          v-model.number="localFactor"
          class="input-num"
          type="number"
          min="0.05"
          max="5"
          step="0.05"
          :disabled="busy"
          @blur="commitFactor"
          @keydown.enter.prevent="onFactorEnter"
        />
      </div>
    </template>
  </section>
</template>

<style scoped>
.runtime {
  padding: 10px 18px 12px;
  margin: 0;
  font-size: 13px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-light);
}
.meta {
  margin-bottom: 10px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-light);
}
.desc {
  margin: 0 0 6px;
  line-height: 1.45;
  color: var(--text-secondary);
  font-size: 12px;
}
.sub {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}
.personality-source-line {
  margin-top: 8px;
}
.ps-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.runtime-backend-hint {
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px dashed var(--border-light);
}
.link-open-backends {
  margin: 0 2px;
  padding: 0;
  border: none;
  background: none;
  color: var(--accent, #6b8cff);
  text-decoration: underline;
  cursor: pointer;
  font: inherit;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
label {
  min-width: 72px;
  color: var(--text-secondary);
}
.select {
  flex: 1;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.input-num {
  width: 100px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
</style>
