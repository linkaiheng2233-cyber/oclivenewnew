<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoleStore } from "../stores/roleStore";
import { useUiStore } from "../stores/uiStore";
import { buildRelationDropdownOptions } from "../utils/relationOptions";
import {
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setEvolutionFactor,
  setRemoteLifeEnabled,
  setUserRelation,
} from "../utils/tauri-api";

const roleStore = useRoleStore();
const uiStore = useUiStore();
const localFactor = ref(roleStore.roleInfo.eventImpactFactor);
const busy = ref(false);

const pluginBackends = computed(() => roleStore.roleInfo.pluginBackends);

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
  const sel = ev.target as HTMLSelectElement;
  const next = sel.value;
  if (next === roleStore.relationSelectValue) return;
  busy.value = true;
  try {
    const perScene = roleStore.roleInfo.identityBinding === "per_scene";
    if (next === OCLIVE_DEFAULT_RELATION_SENTINEL) {
      if (perScene) {
        await roleStore.setManifestDefaultIdentity(uiStore.sceneId);
      } else {
        await roleStore.setManifestDefaultIdentity();
      }
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

async function onRemoteLifeChange(ev: Event) {
  const checked = (ev.target as HTMLInputElement).checked;
  busy.value = true;
  try {
    const info = await setRemoteLifeEnabled(roleStore.currentRoleId, checked);
    roleStore.applyRoleInfo(info);
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
</script>

<template>
  <section class="runtime">
    <div class="meta">
      <p v-if="roleStore.roleInfo.description" class="desc">
        {{ roleStore.roleInfo.description }}
      </p>
      <p class="sub">
        版本 {{ roleStore.roleInfo.version || "—" }} · 作者
        {{ roleStore.roleInfo.author || "—" }}
      </p>
      <p
        class="sub plugin-backends"
        title="settings.json → plugin_backends（见 creator-docs/plugin-and-architecture/PLUGIN_V1.md）"
      >
        模块后端：mem {{ pluginBackends.memory }} · emotion {{ pluginBackends.emotion }} · event {{ pluginBackends.event }} · prompt {{ pluginBackends.prompt }} · llm {{ pluginBackends.llm }}
      </p>
    </div>
    <div v-if="roleStore.interactionImmersive" class="row row-check">
      <label for="remote-life">异地心声</label>
      <input
        id="remote-life"
        type="checkbox"
        :checked="roleStore.roleInfo.remoteLifeEnabled"
        :disabled="busy"
        @change="onRemoteLifeChange"
      />
      <span
        v-if="roleStore.roleInfo.remoteLifePackDefault === true"
        class="hint"
      >包默认建议开</span>
    </div>
    <template v-if="roleStore.roleInfo.userRelations.length > 0">
    <div class="row">
      <label for="rel-select">关系</label>
      <select
        id="rel-select"
        class="select"
        :disabled="busy"
        :value="roleStore.relationSelectValue"
        @change="onRelationChange"
      >
        <option
          v-for="r in relationRows"
          :key="r.id"
          :value="r.id"
        >
          {{ r.name || r.id }}
        </option>
      </select>
    </div>
    <div class="row">
      <label for="evolve-factor">事件影响</label>
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
  border-radius: 0;
  box-shadow: none;
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
  opacity: 0.9;
}
.plugin-backends {
  margin-top: 6px;
  font-family: ui-monospace, monospace;
  font-size: 11px;
  line-height: 1.4;
  word-break: break-word;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.row:last-child {
  margin-bottom: 0;
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
  color: inherit;
}
.input-num {
  width: 100px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: inherit;
}
.row-check input[type="checkbox"] {
  width: auto;
  accent-color: var(--accent, #6b8cff);
}
.hint {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.9;
}
</style>
