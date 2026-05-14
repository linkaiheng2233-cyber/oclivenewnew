<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import {
  getHotkeyBindings,
  saveHotkeyBindings,
  type HotkeyBinding,
  type HotkeyBindingsFile,
} from "../utils/tauri-api";

const { t } = useI18n();
const { showToast } = useAppToast();

const loading = ref(false);
const file = ref<HotkeyBindingsFile>({ schemaVersion: 1, bindings: [] });

onMounted(async () => {
  loading.value = true;
  try {
    file.value = await getHotkeyBindings();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
});

function addBinding(): void {
  const id =
    typeof crypto !== "undefined" && crypto.randomUUID
      ? crypto.randomUUID()
      : `hk-${Date.now()}`;
  const next: HotkeyBinding = {
    id,
    accelerator: "",
    enabled: false,
    action: { type: "openLauncherList" },
  };
  file.value = {
    ...file.value,
    bindings: [...file.value.bindings, next],
  };
}

function removeAt(i: number): void {
  const next = [...file.value.bindings];
  next.splice(i, 1);
  file.value = { ...file.value, bindings: next };
}

function setActionType(i: number, actionType: string): void {
  const next = [...file.value.bindings];
  const b = next[i];
  if (!b) return;
  if (actionType === "openLauncherList") {
    b.action = { type: "openLauncherList" };
  } else {
    b.action = {
      type: "openPluginSlot",
      pluginId: "",
      slot: "chat_toolbar",
      appearanceId: "",
    };
  }
  file.value = { ...file.value, bindings: next };
}

async function onSave(): Promise<void> {
  loading.value = true;
  try {
    await saveHotkeyBindings(file.value);
    showToast("success", t("hotkeys.savedToast"));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <section class="hkset">
    <h3 class="hkset-h">{{ t("hotkeys.title") }}</h3>
    <p class="hkset-lead">
      {{ t("hotkeys.lead") }}
    </p>
    <p v-if="loading" class="hkset-muted">{{ t("common.loading") }}</p>
    <template v-else>
      <div v-for="(b, i) in file.bindings" :key="b.id" class="hkset-row">
        <label class="hkset-field">
          <span>{{ t("hotkeys.fieldAccelerator") }}</span>
          <input v-model="b.accelerator" type="text" :placeholder="t('hotkeys.accelPlaceholder')" />
        </label>
        <label class="hkset-chk">
          <input v-model="b.enabled" type="checkbox" />
          {{ t("hotkeys.enabled") }}
        </label>
        <label class="hkset-field">
          <span>{{ t("hotkeys.action") }}</span>
          <select
            :value="b.action.type"
            @change="
              setActionType(i, ($event.target as HTMLSelectElement).value)
            "
          >
            <option value="openLauncherList">{{ t("hotkeys.actionOpenLauncher") }}</option>
            <option value="openPluginSlot">{{ t("hotkeys.actionOpenSlot") }}</option>
          </select>
        </label>
        <template v-if="b.action.type === 'openPluginSlot'">
          <label class="hkset-field">
            <span>{{ t("hotkeys.pluginId") }}</span>
            <input v-model="b.action.pluginId" type="text" />
          </label>
          <label class="hkset-field">
            <span>{{ t("hotkeys.slotName") }}</span>
            <input v-model="b.action.slot" type="text" />
          </label>
          <label class="hkset-field">
            <span>{{ t("hotkeys.appearanceOptional") }}</span>
            <input v-model="b.action.appearanceId" type="text" />
          </label>
        </template>
        <button type="button" class="hkset-remove" @click="removeAt(i)">{{ t("hotkeys.remove") }}</button>
      </div>
      <div class="hkset-actions">
        <button type="button" class="hkset-btn" @click="addBinding">{{ t("hotkeys.addRow") }}</button>
        <button type="button" class="hkset-btn hkset-btn--primary" @click="onSave">{{ t("hotkeys.save") }}</button>
      </div>
    </template>
  </section>
</template>

<style scoped>
.hkset {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.hkset-h {
  margin: 0;
  font-size: 15px;
}
.hkset-lead {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.hkset-muted {
  font-size: 13px;
  color: var(--text-secondary);
}
.hkset-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: flex-end;
  padding: 10px;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  background: var(--bg-elevated);
}
.hkset-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}
.hkset-field input,
.hkset-field select {
  min-width: 140px;
  padding: 6px 8px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.hkset-chk {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  user-select: none;
}
.hkset-remove {
  margin-left: auto;
  font-size: 12px;
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-light);
  background: transparent;
  cursor: pointer;
}
.hkset-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.hkset-btn {
  padding: 8px 14px;
  font-size: 13px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: transparent;
  cursor: pointer;
}
.hkset-btn--primary {
  background: var(--accent, #3b82f6);
  color: #fff;
  border-color: transparent;
}
</style>
