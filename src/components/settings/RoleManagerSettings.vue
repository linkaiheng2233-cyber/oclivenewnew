<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { confirm } from "@tauri-apps/api/dialog";
import { open, save } from "@tauri-apps/api/dialog";
import ImportProgressModal from "../ImportProgressModal.vue";
import { useAppToast } from "../../composables/useAppToast";
import { useRoleStore } from "../../stores/roleStore";
import {
  deleteRole,
  exportRolePack,
  getRoleInfo,
  importRolePack,
  peekRolePack,
  revealRolePackFolder,
  type RoleInfo,
  type RolePackPeek,
} from "../../utils/tauri-api";

const emit = defineEmits<{
  switchRole: [roleId: string];
  openMarket: [];
  packImported: [roleId: string];
}>();

const roleStore = useRoleStore();
const { t } = useI18n();
const { showToast } = useAppToast();

const revealBusy = ref(false);
const search = ref("");
const focusedRoleId = ref("");
const previewInfo = ref<RoleInfo | null>(null);
const previewLoading = ref(false);
const importBusy = ref(false);
const exportBusy = ref(false);
const deleteBusy = ref(false);
const importProgressOpen = ref(false);
const importPercent = ref(0);
const importMessage = ref("");
let unlistenProgress: UnlistenFn | null = null;

onMounted(() => {
  void roleStore.loadRoles();
});

watch(
  () => roleStore.roles,
  (roles) => {
    if (!roles.length) {
      focusedRoleId.value = "";
      previewInfo.value = null;
      return;
    }
    if (!focusedRoleId.value || !roles.some((r) => r.id === focusedRoleId.value)) {
      focusedRoleId.value = roleStore.currentRoleId || roles[0]!.id;
    }
  },
  { immediate: true },
);

watch(
  () => roleStore.currentRoleId,
  (id) => {
    if (id && !focusedRoleId.value) focusedRoleId.value = id;
  },
  { immediate: true },
);

watch(
  focusedRoleId,
  (id) => {
    if (!id.trim()) {
      previewInfo.value = null;
      return;
    }
    if (id === roleStore.currentRoleId) {
      previewInfo.value = null;
      return;
    }
    previewLoading.value = true;
    void (async () => {
      try {
        previewInfo.value = await getRoleInfo(id);
      } catch {
        previewInfo.value = null;
      } finally {
        previewLoading.value = false;
      }
    })();
  },
  { immediate: true },
);

const filteredRoles = computed(() => {
  const q = search.value.trim().toLowerCase();
  const rows = roleStore.roles;
  if (!q) return rows;
  return rows.filter((r) => {
    const hay = `${r.id} ${r.name}`.toLowerCase();
    return hay.includes(q);
  });
});

const displayName = computed(() => {
  const id = focusedRoleId.value;
  const row = roleStore.roles.find((r) => r.id === id);
  if (id === roleStore.currentRoleId) {
    return { name: roleStore.roleInfo.name, version: roleStore.roleInfo.version, desc: roleStore.roleInfo.description };
  }
  const p = previewInfo.value;
  if (p) {
    return {
      name: p.role_name || p.role_id || row?.name || id,
      version: p.version ?? "",
      desc: p.description ?? "",
    };
  }
  return {
    name: row?.name ?? id,
    version: "",
    desc: "",
  };
});

function safeFileSegment(s: string): string {
  const x = s.replace(/[<>:"/\\|?*\x00-\x1f]/g, "_").trim();
  return x.length > 0 ? x.slice(0, 80) : "role";
}

async function onRevealPack(): Promise<void> {
  const rid = (focusedRoleId.value || roleStore.currentRoleId || "").trim();
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

function setFocused(id: string): void {
  focusedRoleId.value = id;
}

function useAsCurrent(): void {
  const id = focusedRoleId.value.trim();
  if (!id || id === roleStore.currentRoleId) return;
  emit("switchRole", id);
}

async function withImportProgress<T>(fn: () => Promise<T>): Promise<T> {
  importProgressOpen.value = true;
  importPercent.value = 0;
  importMessage.value = String(t("rolePackBar.progress.preparing"));
  unlistenProgress = await listen<{ percent: number; message: string }>("import_progress", (e) => {
    importPercent.value = e.payload.percent;
    importMessage.value = e.payload.message;
  });
  try {
    return await fn();
  } finally {
    unlistenProgress?.();
    unlistenProgress = null;
    importProgressOpen.value = false;
  }
}

async function runImportFlow(path: string): Promise<void> {
  const peek = await peekRolePack(path);
  const exists = roleStore.roles.some((r) => r.id === peek.id);
  if (exists) {
    const ok = await confirm(
      String(
        t("settings.roleSettings.importOverwriteConfirm", {
          id: peek.id,
          name: peek.name,
          version: peek.version,
        }),
      ),
      {
        title: String(t("rolePackBar.conflict.title")),
        type: "warning",
        okLabel: String(t("rolePackBar.conflict.overwrite")),
        cancelLabel: String(t("common.cancel")),
      },
    );
    if (!ok) return;
    const roleId = await withImportProgress(() => importRolePack(path, true));
    showToast("success", String(t("rolePackBar.toasts.imported", { id: roleId })));
    await roleStore.loadRoles();
    focusedRoleId.value = roleId;
    emit("packImported", roleId);
    return;
  }
  const roleId = await withImportProgress(() => importRolePack(path, false));
  showToast("success", String(t("rolePackBar.toasts.imported", { id: roleId })));
  await roleStore.loadRoles();
  focusedRoleId.value = roleId;
  emit("packImported", roleId);
}

async function pickImportSource(mode: "archive" | "folder"): Promise<string | null> {
  const path = await open(
    mode === "folder"
      ? { directory: true, multiple: false }
      : {
          filters: [{ name: "OCPak / ZIP", extensions: ["ocpak", "zip"] }],
          multiple: false,
          directory: false,
        },
  );
  if (path === null || Array.isArray(path)) return null;
  return path;
}

async function onImportArchive(): Promise<void> {
  if (importBusy.value) return;
  importBusy.value = true;
  try {
    const path = await pickImportSource("archive");
    if (!path) return;
    await runImportFlow(path);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    importBusy.value = false;
  }
}

async function onImportFolder(): Promise<void> {
  if (importBusy.value) return;
  importBusy.value = true;
  try {
    const path = await pickImportSource("folder");
    if (!path) return;
    await runImportFlow(path);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    importBusy.value = false;
  }
}

async function onExport(): Promise<void> {
  const rid = (focusedRoleId.value || roleStore.currentRoleId || "").trim();
  if (!rid) {
    showToast("error", String(t("settings.roleSettings.revealNoRole")));
    return;
  }
  exportBusy.value = true;
  try {
    const name = safeFileSegment(displayName.value.name || rid);
    const ver = safeFileSegment(displayName.value.version || "0");
    const path = await save({
      filters: [{ name: String(t("rolePackBar.export.filterName")), extensions: ["ocpak"] }],
      defaultPath: `${name}_${ver}.ocpak`,
    });
    if (!path || typeof path !== "string") return;
    await exportRolePack(rid, path);
    showToast("success", String(t("rolePackBar.toasts.exported")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    exportBusy.value = false;
  }
}

async function onDelete(): Promise<void> {
  const rid = (focusedRoleId.value || "").trim();
  if (!rid) return;
  const ok = await confirm(String(t("settings.roleSettings.deleteConfirm", { id: rid })), {
    title: String(t("settings.roleSettings.deleteTitle")),
    type: "warning",
    okLabel: String(t("settings.roleSettings.deleteOk")),
    cancelLabel: String(t("common.cancel")),
  });
  if (!ok) return;
  deleteBusy.value = true;
  try {
    await deleteRole(rid);
    showToast("success", String(t("settings.roleSettings.deleteOkToast", { id: rid })));
    await roleStore.loadRoles();
    if (roleStore.roles.length) {
      const next = roleStore.roles[0]!.id;
      focusedRoleId.value = next;
      if (rid === roleStore.currentRoleId) {
        emit("switchRole", next);
      }
    } else {
      focusedRoleId.value = "";
      previewInfo.value = null;
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  } finally {
    deleteBusy.value = false;
  }
}
</script>

<template>
  <div class="rms">
    <p class="rms-lead">{{ t("settings.roleSettings.lead") }}</p>

    <div class="rms-toolbar">
      <input
        v-model="search"
        type="search"
        class="rms-search"
        :placeholder="String(t('settings.roleSettings.searchPlaceholder'))"
        autocomplete="off"
        spellcheck="false"
      />
      <div class="rms-toolbar-btns">
        <button type="button" class="rms-btn rms-btn--accent" :disabled="importBusy" @click="onImportArchive">
          {{ t("settings.roleSettings.importArchive") }}
        </button>
        <button type="button" class="rms-btn" :disabled="importBusy" @click="onImportFolder">
          {{ t("settings.roleSettings.importFolder") }}
        </button>
        <button type="button" class="rms-btn" @click="emit('openMarket')">
          {{ t("settings.roleSettings.emptyOpenMarket") }}
        </button>
      </div>
    </div>

    <div v-if="!roleStore.roles.length" class="rms-empty">
      <p class="rms-muted">{{ t("settings.roleSettings.emptyLead") }}</p>
    </div>

    <div v-else class="rms-split">
      <aside class="rms-list" aria-label="roles">
        <button
          v-for="r in filteredRoles"
          :key="r.id"
          type="button"
          class="rms-list-item"
          :class="{ 'rms-list-item--active': r.id === focusedRoleId }"
          @click="setFocused(r.id)"
        >
          <span class="rms-list-name">{{ r.name }}</span>
          <span class="rms-list-id">{{ r.id }}</span>
          <span v-if="r.id === roleStore.currentRoleId" class="rms-pill">{{ t("settings.roleSettings.currentBadge") }}</span>
        </button>
        <p v-if="!filteredRoles.length" class="rms-muted rms-list-empty">{{ t("settings.roleSettings.noSearchHits") }}</p>
      </aside>

      <div class="rms-detail">
        <section class="rms-card" aria-labelledby="rms-summary-h">
          <h3 id="rms-summary-h" class="rms-h">{{ t("settings.roleSettings.summaryTitle") }}</h3>
          <p v-if="previewLoading" class="rms-muted">{{ t("common.loading") }}</p>
          <template v-else>
            <p class="rms-meta">
              <strong>{{ displayName.name }}</strong>
              <span v-if="displayName.version"> · v{{ displayName.version }}</span>
            </p>
            <p v-if="displayName.desc" class="rms-desc">{{ displayName.desc }}</p>
            <p v-else class="rms-muted">{{ t("settings.roleSettings.noDescription") }}</p>
          </template>
        </section>

        <div class="rms-actions">
          <button
            v-if="focusedRoleId && focusedRoleId !== roleStore.currentRoleId"
            type="button"
            class="rms-btn rms-btn--accent"
            @click="useAsCurrent"
          >
            {{ t("settings.roleSettings.useAsCurrent") }}
          </button>
          <button type="button" class="rms-btn" :disabled="revealBusy || !focusedRoleId" @click="onRevealPack">
            {{ t("settings.roleSettings.revealPack") }}
          </button>
          <button type="button" class="rms-btn" :disabled="exportBusy || !focusedRoleId" @click="onExport">
            {{ t("settings.roleSettings.exportPack") }}
          </button>
          <button type="button" class="rms-btn rms-btn--danger" :disabled="deleteBusy || !focusedRoleId" @click="onDelete">
            {{ t("settings.roleSettings.deleteRole") }}
          </button>
        </div>
        <p class="rms-muted rms-pack-editor">{{ t("settings.roleSettings.packEditorHint") }}</p>
      </div>
    </div>

    <ImportProgressModal :open="importProgressOpen" :percent="importPercent" :message="importMessage" />
  </div>
</template>

<style scoped>
.rms {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 920px;
}
.rms-lead {
  margin: 0;
  font-size: 13px;
  line-height: 1.45;
  color: var(--text-secondary);
}
.rms-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}
.rms-search {
  flex: 1 1 200px;
  min-width: 160px;
  padding: 7px 10px;
  font-size: 13px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
}
.rms-toolbar-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.rms-split {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  align-items: flex-start;
}
.rms-list {
  flex: 0 1 260px;
  min-width: 200px;
  max-height: 360px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
}
.rms-list-item {
  text-align: left;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: inherit;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.rms-list-item:hover {
  background: var(--bg-hover, rgba(255, 255, 255, 0.05));
}
.rms-list-item--active {
  border-color: color-mix(in srgb, var(--accent, #3b82f6) 45%, var(--border-light));
  background: color-mix(in srgb, var(--accent, #3b82f6) 10%, var(--bg-primary));
}
.rms-list-name {
  font-size: 13px;
  font-weight: 650;
}
.rms-list-id {
  font-size: 11px;
  color: var(--text-secondary);
  font-family: ui-monospace, monospace;
}
.rms-list-empty {
  padding: 8px;
}
.rms-pill {
  align-self: flex-start;
  margin-top: 4px;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent, #3b82f6) 22%, transparent);
}
.rms-detail {
  flex: 1 1 280px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
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
.rms-btn--danger {
  border-color: color-mix(in srgb, var(--text-danger, #c33) 42%, var(--border-light));
  color: var(--text-danger, #c33);
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
