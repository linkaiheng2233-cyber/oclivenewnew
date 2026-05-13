<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/api/dialog";
import { open as openExternal } from "@tauri-apps/api/shell";
import { useRoleStore } from "../stores/roleStore";
import {
  exportRolePack,
  importRolePack,
  peekRolePack,
  readRoleCreatorMessageLines,
  syncRoleMarketIndex,
  installRolePackFromMarket,
  type RoleMarketEntryDto,
  type RoleMarketDownloadDto,
  type RolePackPeek,
} from "../utils/tauri-api";
import ImportProgressModal from "./ImportProgressModal.vue";

const roleStore = useRoleStore();
const { t } = useI18n();

const emit = defineEmits<{
  notify: [payload: { type: "success" | "error" | "info" | "warning"; message: string }];
  imported: [roleId: string];
}>();

/** Windows / 通用非法文件名字符 */
function safeFileSegment(s: string): string {
  const t = s.replace(/[<>:"/\\|?*\x00-\x1f]/g, "_").trim();
  return t.length > 0 ? t.slice(0, 80) : "role";
}

function defaultExportFilename(): string {
  const name = safeFileSegment(roleStore.roleInfo.name || roleStore.currentRoleId);
  const ver = safeFileSegment(roleStore.roleInfo.version || "0");
  return `${name}_${ver}.ocpak`;
}

const conflictOpen = ref(false);
const pendingPath = ref<string | null>(null);
const pendingPeek = ref<RolePackPeek | null>(null);

const importProgressOpen = ref(false);
const importPercent = ref(0);
const importMessage = ref(String(t("rolePackBar.progress.preparing")));
let unlistenProgress: UnlistenFn | null = null;

const creatorEchoOpen = ref(false);
const creatorEchoRole = ref<RolePackPeek | null>(null);
const creatorEchoLines = ref<string[]>([]);
const postImportRoleId = ref<string>("");

const marketOpen = ref(false);
const marketLoading = ref(false);
const marketErr = ref("");
const marketQuery = ref("");
const marketIndex = ref<{ roles: RoleMarketEntryDto[] } | null>(null);
const marketPicked = ref<{ roleId: string; dl: RoleMarketDownloadDto } | null>(null);
const marketSourceUrl = ref<string>("");

/** manifest `creator_message_to_downloader` 与 `creator_message.txt` 合并为展示行（去重）。 */
async function mergeCreatorEchoLines(
  peek: RolePackPeek,
  roleId: string,
): Promise<string[]> {
  const out: string[] = [];
  const m = peek.creatorMessageToDownloader?.trim();
  if (m) out.push(m);
  try {
    const fileLines = await readRoleCreatorMessageLines(roleId);
    for (const line of fileLines) {
      const t = line.trim();
      if (t && !out.includes(t)) out.push(t);
    }
  } catch {
    // ignore file read errors
  }
  return out;
}

async function withImportProgress<T>(fn: () => Promise<T>): Promise<T> {
  importProgressOpen.value = true;
  importPercent.value = 0;
  importMessage.value = String(t("rolePackBar.progress.preparing"));
  unlistenProgress = await listen<{ percent: number; message: string }>(
    "import_progress",
    (e) => {
      importPercent.value = e.payload.percent;
      importMessage.value = e.payload.message;
    },
  );
  try {
    return await fn();
  } finally {
    unlistenProgress?.();
    unlistenProgress = null;
    importProgressOpen.value = false;
  }
}

async function onExport(): Promise<void> {
  try {
    const path = await save({
      filters: [{ name: String(t("rolePackBar.export.filterName")), extensions: ["ocpak"] }],
      defaultPath: defaultExportFilename(),
    });
    if (!path || typeof path !== "string") return;
    await exportRolePack(roleStore.currentRoleId, path);
    emit("notify", { type: "success", message: String(t("rolePackBar.toasts.exported")) });
  } catch (e) {
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  }
}

function closeConflict(): void {
  conflictOpen.value = false;
  pendingPath.value = null;
  pendingPeek.value = null;
}

async function confirmOverwrite(): Promise<void> {
  const path = pendingPath.value;
  if (!path) {
    closeConflict();
    return;
  }
  if (importProgressOpen.value) return;
  try {
    const roleId = await withImportProgress(() => importRolePack(path, true));
    postImportRoleId.value = roleId;
    const peek = pendingPeek.value;
    if (peek) {
      creatorEchoRole.value = peek;
      creatorEchoLines.value = await mergeCreatorEchoLines(peek, roleId);
    }
    creatorEchoOpen.value = true;
  } catch (e) {
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  } finally {
    closeConflict();
  }
}

async function runImportFlow(path: string): Promise<void> {
  const peek = await peekRolePack(path);
  const exists = roleStore.roles.some((r) => r.id === peek.id);
  if (exists) {
    pendingPath.value = path;
    pendingPeek.value = peek;
    conflictOpen.value = true;
    return;
  }

  const roleId = await withImportProgress(() =>
    importRolePack(path, false),
  );
  postImportRoleId.value = roleId;
  creatorEchoRole.value = peek;
  creatorEchoLines.value = await mergeCreatorEchoLines(peek, roleId);
  creatorEchoOpen.value = true;
}

async function switchToImportedRole(): Promise<void> {
  const roleId = postImportRoleId.value.trim();
  if (!roleId) {
    creatorEchoOpen.value = false;
    return;
  }
  creatorEchoOpen.value = false;
  postImportRoleId.value = "";
  emit("notify", { type: "success", message: String(t("rolePackBar.toasts.imported", { id: roleId })) });
  emit("imported", roleId);
}

async function keepCurrentAfterImport(): Promise<void> {
  const roleId = postImportRoleId.value.trim();
  creatorEchoOpen.value = false;
  postImportRoleId.value = "";
  try {
    await roleStore.loadRoles();
  } catch {
    // ignore
  }
  if (roleId) {
    emit(
      "notify",
      {
        type: "success",
        message: String(t("rolePackBar.toasts.importedNoSwitch", { id: roleId })),
      },
    );
  }
}

async function pickImportSource(
  mode: "archive" | "folder",
): Promise<string | null> {
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

async function runImportWithPicker(mode: "archive" | "folder"): Promise<void> {
  if (importProgressOpen.value) return;
  try {
    const path = await pickImportSource(mode);
    if (!path) return;
    await runImportFlow(path);
  } catch (e) {
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  }
}

function onImport(): void {
  void runImportWithPicker("archive");
}

function onImportFolder(): void {
  void runImportWithPicker("folder");
}

function openMarket(): void {
  marketOpen.value = true;
  marketErr.value = "";
  marketQuery.value = "";
  marketPicked.value = null;
  void syncMarket();
}

async function syncMarket(): Promise<void> {
  marketLoading.value = true;
  marketErr.value = "";
  try {
    const url = marketSourceUrl.value.trim();
    const idx = await syncRoleMarketIndex(url ? url : null);
    marketIndex.value = idx;
  } catch (e) {
    marketErr.value = e instanceof Error ? e.message : String(e);
    marketIndex.value = null;
  } finally {
    marketLoading.value = false;
  }
}

function closeMarket(): void {
  marketOpen.value = false;
  marketErr.value = "";
  marketPicked.value = null;
}

function filteredMarketRoles(): RoleMarketEntryDto[] {
  const roles = marketIndex.value?.roles ?? [];
  const q = marketQuery.value.trim().toLowerCase();
  if (!q) return roles;
  return roles.filter((r) => {
    const hay = [r.id, r.name, r.author ?? "", r.description ?? "", ...(r.tags ?? [])]
      .join(" ")
      .toLowerCase();
    return hay.includes(q);
  });
}

function pickDownload(roleId: string, dl: RoleMarketDownloadDto): void {
  marketPicked.value = { roleId, dl };
}

async function installPicked(): Promise<void> {
  const picked = marketPicked.value;
  if (!picked) return;
  if (picked.dl.kind !== "direct") {
    const u = picked.dl.url?.trim();
    if (u) {
      try {
        await openExternal(u);
      } catch {
        // ignore
      }
    }
    emit("notify", {
      type: "info",
      message:
        String(t("rolePackBar.market.notDirectHint")),
    });
    return;
  }
  const exists = roleStore.roles.some((r) => r.id === picked.roleId);
  const overwrite = exists
    ? window.confirm(String(t("rolePackBar.market.confirmOverwrite", { id: picked.roleId })))
    : false;
  if (exists && !overwrite) return;
  try {
    const roleId = await withImportProgress(() =>
      installRolePackFromMarket({
        roleId: picked.roleId,
        downloadUrl: picked.dl.url,
        sha256: picked.dl.sha256,
        overwrite,
      }),
    );
    postImportRoleId.value = roleId;
    try {
      const lines = await readRoleCreatorMessageLines(roleId);
      if (lines.length > 0) {
        creatorEchoRole.value = { id: roleId, name: roleId, version: "" };
        creatorEchoLines.value = lines;
      }
    } catch {
      // ignore
    }
    creatorEchoOpen.value = true;
    closeMarket();
  } catch (e) {
    emit("notify", {
      type: "error",
      message: e instanceof Error ? e.message : String(e),
    });
  }
}
</script>

<template>
  <div
    class="pack-bar"
    :title="String(t('rolePackBar.barTitle'))"
  >
    <button type="button" class="btn" @click="onExport">{{ t("rolePackBar.actions.export") }}</button>
    <button
      type="button"
      class="btn"
      :disabled="importProgressOpen"
      @click="onImport"
    >
      {{ t("rolePackBar.actions.importArchive") }}
    </button>
    <button
      type="button"
      class="btn"
      :disabled="importProgressOpen"
      @click="onImportFolder"
    >
      {{ t("rolePackBar.actions.importFolder") }}
    </button>
    <button type="button" class="btn" :disabled="importProgressOpen" @click="openMarket">
      {{ t("rolePackBar.actions.openMarket") }}
    </button>

    <ImportProgressModal
      :open="importProgressOpen"
      :percent="importPercent"
      :message="importMessage"
    />

    <Teleport to="body">
      <div
        v-if="creatorEchoOpen"
        class="modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="creator-echo-title"
        @click.self="creatorEchoOpen = false"
      >
        <div class="modal-card" @click.stop>
          <h2 id="creator-echo-title" class="modal-title">
            {{
              creatorEchoLines.length > 0
                ? t("rolePackBar.creatorEcho.titleWithMessage")
                : t("rolePackBar.creatorEcho.titleSuccess")
            }}
          </h2>
          <p v-if="creatorEchoRole" class="modal-body">
            <strong>{{ creatorEchoRole.name }}</strong>
            <span class="pm-muted">({{ creatorEchoRole.id }})</span>
            <span v-if="creatorEchoRole.version" class="pm-muted">· v{{ creatorEchoRole.version }}</span>
          </p>
          <div class="modal-body">
            <p v-for="(l, i) in creatorEchoLines" :key="i" class="pm-echo-line">
              {{ l }}
            </p>
            <p v-if="creatorEchoLines.length === 0" class="pm-echo-line">
              {{ t("rolePackBar.creatorEcho.promptSwitchNow") }}
            </p>
          </div>
          <div class="modal-actions">
            <button
              type="button"
              class="btn btn-ghost"
              @click="keepCurrentAfterImport"
            >
              {{ t("rolePackBar.creatorEcho.later") }}
            </button>
            <button type="button" class="btn btn-danger" @click="switchToImportedRole">
              {{ t("rolePackBar.creatorEcho.switchNow") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <Teleport to="body">
      <div
        v-if="conflictOpen && pendingPeek"
        class="modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="pack-conflict-title"
      >
        <div class="modal-card" @click.stop>
          <h2 id="pack-conflict-title" class="modal-title">{{ t("rolePackBar.conflict.title") }}</h2>
          <p class="modal-body">
            <span v-html="t('rolePackBar.conflict.bodyHtml', { id: pendingPeek.id, name: pendingPeek.name, version: pendingPeek.version })"></span>
          </p>
          <div class="modal-actions">
            <button
              type="button"
              class="btn btn-ghost"
              :disabled="importProgressOpen"
              @click="closeConflict"
            >
              {{ t("common.cancel") }}
            </button>
            <button
              type="button"
              class="btn btn-danger"
              :disabled="importProgressOpen"
              @click="confirmOverwrite"
            >
              {{ t("rolePackBar.conflict.overwrite") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <Teleport to="body">
      <div
        v-if="marketOpen"
        class="modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="pack-market-title"
      >
        <div class="modal-card modal-card--wide" @click.stop>
          <h2 id="pack-market-title" class="modal-title">{{ t("rolePackBar.market.title") }}</h2>
          <p class="modal-body">
            {{ t("rolePackBar.market.hint") }}
          </p>
          <div class="pm-row">
            <input
              v-model="marketQuery"
              class="pm-input"
              type="search"
              :placeholder="String(t('rolePackBar.market.searchPlaceholder'))"
              autocomplete="off"
            />
            <button type="button" class="btn" :disabled="marketLoading" @click="syncMarket">
              {{ marketLoading ? t("rolePackBar.market.syncing") : t("rolePackBar.market.sync") }}
            </button>
          </div>
          <div class="pm-row">
            <input
              v-model="marketSourceUrl"
              class="pm-input"
              type="url"
              :placeholder="String(t('rolePackBar.market.sourcePlaceholder'))"
              autocomplete="off"
            />
          </div>
          <p v-if="marketErr" class="pm-err">{{ marketErr }}</p>
          <div class="pm-list">
            <div v-if="marketLoading" class="pm-empty">{{ t("rolePackBar.market.loading") }}</div>
            <div v-else-if="!(marketIndex?.roles?.length ?? 0)" class="pm-empty">
              {{ t("rolePackBar.market.empty") }}
            </div>
            <div v-else>
              <div
                v-for="r in filteredMarketRoles()"
                :key="r.id"
                class="pm-item"
              >
                <div class="pm-item-head">
                  <div class="pm-item-title">
                    <strong>{{ r.name }}</strong>
                    <span class="pm-muted">({{ r.id }})</span>
                  </div>
                  <div class="pm-muted">v{{ r.version }} · {{ r.author || "—" }}</div>
                </div>
                <div v-if="r.description" class="pm-desc">{{ r.description }}</div>
                <div class="pm-dls">
                  <button
                    v-for="(dl, i) in r.downloads"
                    :key="i"
                    type="button"
                    class="pm-dl"
                    :class="{ active: marketPicked?.roleId === r.id && marketPicked?.dl?.url === dl.url }"
                    @click="pickDownload(r.id, dl)"
                    :title="dl.note || ''"
                  >
                    {{ dl.label || dl.kind }}
                    <span class="pm-pill">{{ dl.kind }}</span>
                    <span v-if="dl.trust" class="pm-pill pm-pill--trust">{{ dl.trust }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div class="modal-actions">
            <button type="button" class="btn btn-ghost" :disabled="importProgressOpen" @click="closeMarket">
              {{ t("common.close") }}
            </button>
            <button
              type="button"
              class="btn btn-danger"
              :disabled="!marketPicked || importProgressOpen"
              @click="installPicked"
            >
              {{ t("rolePackBar.market.installPicked") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.pack-bar {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.btn {
  font-size: 11px;
  padding: 4px 8px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--btn-grad-a);
  color: var(--text-secondary);
  cursor: pointer;
}
.btn:hover {
  background: var(--btn-primary-hover-a);
  color: var(--text-primary);
}
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--dialog-backdrop, rgba(0, 0, 0, 0.45));
}
.modal-card {
  max-width: 400px;
  width: 100%;
  padding: 20px;
  border-radius: 12px;
  background: var(--bg-panel, #1a1a22);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md, 0 8px 32px rgba(0, 0, 0, 0.35));
}
.modal-title {
  margin: 0 0 12px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}
.modal-body {
  margin: 0 0 18px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
.btn-ghost {
  background: transparent;
}
.btn-danger {
  border-color: #c45c5c;
  background: linear-gradient(180deg, #a04040, #802828);
  color: #fff;
}
.btn-danger:hover {
  filter: brightness(1.08);
}
.modal-card--wide {
  max-width: 780px;
}
.pm-row {
  display: flex;
  gap: 10px;
  align-items: center;
  margin: 10px 0 8px;
}
.pm-input {
  flex: 1 1 auto;
  min-width: 0;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg, #111118);
  color: var(--text-primary);
  font-size: 12px;
}
.pm-err {
  margin: 6px 0 0;
  color: #c45c5c;
  font-size: 12px;
}
.pm-echo-line {
  margin: 6px 0;
  white-space: pre-wrap;
  word-break: break-word;
}
.pm-list {
  margin-top: 10px;
  border: 1px solid var(--border-light);
  border-radius: 12px;
  overflow: hidden;
  max-height: 50vh;
  overflow-y: auto;
  background: color-mix(in srgb, var(--bg-panel, #1a1a22) 80%, #000);
}
.pm-empty {
  padding: 12px;
  color: var(--text-secondary);
  font-size: 12px;
}
.pm-item {
  padding: 12px;
  border-bottom: 1px solid var(--border-light);
}
.pm-item:last-child {
  border-bottom: none;
}
.pm-item-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: baseline;
}
.pm-item-title {
  display: flex;
  gap: 8px;
  align-items: baseline;
  flex-wrap: wrap;
}
.pm-muted {
  color: var(--text-secondary);
  font-size: 12px;
}
.pm-desc {
  margin-top: 6px;
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.45;
  white-space: pre-wrap;
}
.pm-dls {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}
.pm-dl {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 11px;
}
.pm-dl.active {
  border-color: color-mix(in srgb, #7aa2ff 55%, var(--border-light));
  color: var(--text-primary);
}
.pm-pill {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  color: var(--text-secondary);
}
.pm-pill--trust {
  border-color: color-mix(in srgb, #6ed1a5 55%, var(--border-light));
}
</style>
