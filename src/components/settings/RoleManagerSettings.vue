<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../../composables/useAppToast";
import { useRoleStore } from "../../stores/roleStore";
import { revealRolePackFolder } from "../../utils/tauri-api";

const emit = defineEmits<{
  switchRole: [roleId: string];
  openMarket: [];
}>();

const roleStore = useRoleStore();
const { t } = useI18n();
const { showToast } = useAppToast();
const revealBusy = ref(false);

onMounted(() => {
  void roleStore.loadRoles();
});

async function onRevealPack(): Promise<void> {
  const rid = (roleStore.currentRoleId ?? "").trim();
  if (!rid) {
    showToast("error", String(t("settings.roleSettings.revealNoRole")));
    return;
  }
  revealBusy.value = true;
  try {
    await revealRolePackFolder(rid);
    showToast("success", String(t("settings.roleSettings.revealOk")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    revealBusy.value = false;
  }
}

function onSelectRole(ev: Event): void {
  const id = (ev.target as HTMLSelectElement).value;
  if (!id || id === roleStore.currentRoleId) return;
  emit("switchRole", id);
}
</script>

<template>
  <div class="rms">
    <p class="rms-lead">{{ t("settings.roleSettings.lead") }}</p>
    <div v-if="!roleStore.roles.length" class="rms-empty">
      <p class="rms-muted">{{ t("settings.roleSettings.emptyLead") }}</p>
      <button type="button" class="rms-btn rms-btn--accent" @click="emit('openMarket')">
        {{ t("settings.roleSettings.emptyOpenMarket") }}
      </button>
    </div>
    <section v-else class="rms-section">
      <label class="rms-label" for="rms-role-select">{{ t("settings.roleSettings.currentRole") }}</label>
      <select
        id="rms-role-select"
        class="rms-select"
        :value="roleStore.currentRoleId"
        :disabled="!roleStore.roles.length"
        @change="onSelectRole"
      >
        <option v-for="r in roleStore.roles" :key="r.id" :value="r.id">{{ r.name }}</option>
      </select>
    </section>
    <section v-if="roleStore.roles.length" class="rms-card" aria-labelledby="rms-summary-h">
      <h3 id="rms-summary-h" class="rms-h">{{ t("settings.roleSettings.summaryTitle") }}</h3>
      <p class="rms-meta"><strong>{{ roleStore.roleInfo.name }}</strong> · v{{ roleStore.roleInfo.version }}</p>
      <p v-if="roleStore.roleInfo.description" class="rms-desc">{{ roleStore.roleInfo.description }}</p>
      <p v-else class="rms-muted">{{ t("settings.roleSettings.noDescription") }}</p>
    </section>
    <div v-if="roleStore.roles.length" class="rms-actions">
      <button type="button" class="rms-btn" :disabled="revealBusy" @click="onRevealPack">
        {{ t("settings.roleSettings.revealPack") }}
      </button>
    </div>
    <p v-if="roleStore.roles.length" class="rms-muted rms-pack-editor">{{ t("settings.roleSettings.packEditorHint") }}</p>
  </div>
</template>

<style scoped>
.rms {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.rms-lead {
  margin: 0;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.rms-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.rms-label {
  font-size: 12px;
  font-weight: 650;
  color: var(--text-secondary);
}
.rms-select {
  max-width: 420px;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
}
.rms-card {
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.rms-h {
  margin: 0 0 8px;
  font-size: 14px;
}
.rms-meta {
  margin: 0 0 6px;
  font-size: 13px;
  color: var(--text-primary);
}
.rms-desc {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.rms-muted {
  margin: 0;
  font-size: 12px;
  line-height: 1.4;
  color: var(--text-secondary);
}
.rms-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.rms-btn {
  padding: 7px 14px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
}
.rms-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.rms-btn--accent {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 38%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 12%, var(--bg-primary));
}
.rms-empty {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border-radius: 10px;
  border: 1px dashed var(--border-light);
}
.rms-pack-editor {
  font-size: 11px;
  line-height: 1.45;
}
</style>
