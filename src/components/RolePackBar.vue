<script setup lang="ts">
import { ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/api/dialog";
import { open as openExternal } from "@tauri-apps/api/shell";
import { useRoleStore } from "../stores/roleStore";
import {
  exportRolePack,
  importRolePack,
  peekRolePack,
  syncRoleMarketIndex,
  installRolePackFromMarket,
  type RoleMarketEntryDto,
  type RoleMarketDownloadDto,
} from "../utils/tauri-api";
import ImportProgressModal from "./ImportProgressModal.vue";

const roleStore = useRoleStore();

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
const pendingPeek = ref<{ id: string; name: string; version: string } | null>(
  null,
);

const importProgressOpen = ref(false);
const importPercent = ref(0);
const importMessage = ref("准备中…");
let unlistenProgress: UnlistenFn | null = null;

const marketOpen = ref(false);
const marketLoading = ref(false);
const marketErr = ref("");
const marketQuery = ref("");
const marketIndex = ref<{ roles: RoleMarketEntryDto[] } | null>(null);
const marketPicked = ref<{ roleId: string; dl: RoleMarketDownloadDto } | null>(null);
const marketSourceUrl = ref<string>("");

async function withImportProgress<T>(fn: () => Promise<T>): Promise<T> {
  importProgressOpen.value = true;
  importPercent.value = 0;
  importMessage.value = "准备中…";
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
      filters: [{ name: "OCPak 角色包", extensions: ["ocpak"] }],
      defaultPath: defaultExportFilename(),
    });
    if (!path || typeof path !== "string") return;
    await exportRolePack(roleStore.currentRoleId, path);
    emit("notify", { type: "success", message: "角色包已导出" });
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
    emit("imported", roleId);
    emit("notify", { type: "success", message: `已覆盖并导入角色: ${roleId}` });
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
  emit("imported", roleId);
  emit("notify", { type: "success", message: `已导入角色: ${peek.name}` });
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
        "该镜像不是直链下载（page/pan）。已尝试为你打开链接；请手动下载后用「导入压缩包」安装。",
    });
    return;
  }
  const exists = roleStore.roles.some((r) => r.id === picked.roleId);
  const overwrite = exists ? window.confirm(`本地已存在角色「${picked.roleId}」。是否覆盖安装？`) : false;
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
    emit("imported", roleId);
    emit("notify", { type: "success", message: `已从市场安装角色: ${roleId}` });
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
    title="安装 .ocpak / .zip 压缩包，或已解压的目录（与 roles/{id}/ 一致）"
  >
    <button type="button" class="btn" @click="onExport">导出角色包</button>
    <button type="button" class="btn" :disabled="importProgressOpen" @click="openMarket">
      从市场安装
    </button>
    <button
      type="button"
      class="btn"
      :disabled="importProgressOpen"
      @click="onImport"
    >
      导入压缩包
    </button>
    <button
      type="button"
      class="btn"
      :disabled="importProgressOpen"
      @click="onImportFolder"
    >
      从文件夹导入
    </button>

    <ImportProgressModal
      :open="importProgressOpen"
      :percent="importPercent"
      :message="importMessage"
    />

    <Teleport to="body">
      <div
        v-if="conflictOpen && pendingPeek"
        class="modal-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="pack-conflict-title"
      >
        <div class="modal-card" @click.stop>
          <h2 id="pack-conflict-title" class="modal-title">角色已存在</h2>
          <p class="modal-body">
            本地已有角色 ID「<strong>{{ pendingPeek.id }}</strong>」
            （{{ pendingPeek.name }} v{{ pendingPeek.version }}）。
            导入将覆盖该角色目录，是否继续？
          </p>
          <div class="modal-actions">
            <button
              type="button"
              class="btn btn-ghost"
              :disabled="importProgressOpen"
              @click="closeConflict"
            >
              取消
            </button>
            <button
              type="button"
              class="btn btn-danger"
              :disabled="importProgressOpen"
              @click="confirmOverwrite"
            >
              覆盖导入
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
          <h2 id="pack-market-title" class="modal-title">角色包市场（roles.json）</h2>
          <p class="modal-body">
            直链镜像会下载并校验 SHA-256 后导入；网盘/下载页镜像请手动下载后用「导入压缩包」。
          </p>
          <div class="pm-row">
            <input
              v-model="marketQuery"
              class="pm-input"
              type="search"
              placeholder="搜索 id / 名称 / 作者 / 标签…"
              autocomplete="off"
            />
            <button type="button" class="btn" :disabled="marketLoading" @click="syncMarket">
              {{ marketLoading ? "同步中…" : "同步" }}
            </button>
          </div>
          <div class="pm-row">
            <input
              v-model="marketSourceUrl"
              class="pm-input"
              type="url"
              placeholder="可选：自定义 roles.json 源（留空=官方默认）"
              autocomplete="off"
            />
          </div>
          <p v-if="marketErr" class="pm-err">{{ marketErr }}</p>
          <div class="pm-list">
            <div v-if="marketLoading" class="pm-empty">正在加载…</div>
            <div v-else-if="!(marketIndex?.roles?.length ?? 0)" class="pm-empty">
              索引为空（或未加载）。
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
              关闭
            </button>
            <button
              type="button"
              class="btn btn-danger"
              :disabled="!marketPicked || importProgressOpen"
              @click="installPicked"
            >
              安装所选镜像
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
