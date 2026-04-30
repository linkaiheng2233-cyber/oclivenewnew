<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAppToast } from "../composables/useAppToast";
import { usePluginCommunityMarketPane } from "../composables/usePluginCommunityMarketPane";
import { usePluginStore } from "../stores/pluginStore";
import {
  importRolePack,
  installLocalPluginArchive,
  installPluginDir,
  listLocalImportCandidates,
  peekRolePack,
  previewLocalPluginArchive,
  previewPluginDirPermissions,
  previewPluginZipPermissions,
  readLocalImportText,
  type PluginMarketEntryDto,
} from "../utils/tauri-api";
import PluginMarketV2Pane from "../components/PluginManagerV2/PluginMarketV2Pane.vue";

const pluginStore = usePluginStore();
const { showToast } = useAppToast();
const m = usePluginCommunityMarketPane();
const { t } = useI18n();

const localImportsLoading = ref(false);
const localImportsErr = ref("");
const localImportsRootDir = ref("");
const localImports = ref<
  Array<{
    kind: string;
    path: string;
    fileName: string;
    relatedSignaturePath?: string | null;
  }>
>([]);

const localImportsByKind = computed(() => {
  const map = new Map<string, typeof localImports.value>();
  for (const it of localImports.value) {
    const list = map.get(it.kind) ?? [];
    list.push(it);
    map.set(it.kind, list);
  }
  return map;
});

function localKindLabel(k: string): string {
  if (k === "role_pack") return String(t("pluginMarketV1.localKinds.rolePack"));
  if (k === "plugin_archive") return String(t("pluginMarketV1.localKinds.pluginArchive"));
  if (k === "plugin_dir") return String(t("pluginMarketV1.localKinds.pluginDir"));
  if (k === "module_json") return String(t("pluginMarketV1.localKinds.moduleJson"));
  if (k === "profile_json") return String(t("pluginMarketV1.localKinds.profileJson"));
  return k;
}

async function refreshLocalImports(): Promise<void> {
  localImportsLoading.value = true;
  localImportsErr.value = "";
  try {
    const r = await listLocalImportCandidates();
    localImportsRootDir.value = r.rootDir ?? "";
    localImports.value = (r.items ?? []).map((x) => ({
      kind: x.kind,
      path: x.path,
      fileName: x.fileName,
      relatedSignaturePath: x.relatedSignaturePath ?? null,
    }));
  } catch (e) {
    localImportsRootDir.value = "";
    localImports.value = [];
    localImportsErr.value = e instanceof Error ? e.message : String(e);
  } finally {
    localImportsLoading.value = false;
  }
}

function parseLocalMarketEntryJson(text: string): PluginMarketEntryDto {
  let j: unknown;
  try {
    j = JSON.parse(text) as unknown;
  } catch (e) {
    throw new Error(
      String(
        t("pluginMarketV1.localJson.errors.parseFailed", {
          msg: e instanceof Error ? e.message : String(e),
        }),
      ),
    );
  }
  if (!j || typeof j !== "object") {
    throw new Error(String(t("pluginMarketV1.localJson.errors.mustBeObject")));
  }
  const o = j as any;
  const t = String(o.type ?? "").trim();
  if (t !== "module" && t !== "profile") {
    throw new Error(String(t("pluginMarketV1.localJson.errors.typeMustBeModuleOrProfile")));
  }
  const id = String(o.id ?? "").trim();
  const name = String(o.name ?? "").trim();
  const version = String(o.version ?? "").trim();
  if (!id || !name || !version) {
    throw new Error(String(t("pluginMarketV1.localJson.errors.missingRequiredFields")));
  }
  return o as PluginMarketEntryDto;
}

async function onPreviewLocalJson(path: string): Promise<void> {
  try {
    const text = await readLocalImportText(path);
    await navigator.clipboard.writeText(text);
    showToast("success", String(t("pluginMarketV1.localJson.toastCopied")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onApplyLocalModuleOrProfile(path: string): Promise<void> {
  try {
    const text = await readLocalImportText(path);
    const row = parseLocalMarketEntryJson(text);
    const t = m.marketEntryType(row);
    if (t === "module") {
      await m.onApplyModuleEntry(row);
    } else if (t === "profile") {
      await m.onApplyProfileEntry(row);
    } else {
      showToast("error", String(t("pluginMarketV1.localJson.errors.onlyModuleProfile")));
    }
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onImportRolePackFromLocal(path: string, overwrite: boolean): Promise<void> {
  try {
    const peek = await peekRolePack(path);
    const ok = window.confirm(
      overwrite
        ? String(t("pluginMarketV1.rolePack.confirmOverwriteImport", { name: peek.name, id: peek.id, version: peek.version }))
        : String(t("pluginMarketV1.rolePack.confirmImport", { name: peek.name, id: peek.id, version: peek.version })),
    );
    if (!ok) return;
    const roleId = await importRolePack(path, overwrite);
    showToast("success", overwrite ? String(t("pluginMarketV1.rolePack.toastImportedOverwrite", { id: roleId })) : String(t("pluginMarketV1.rolePack.toastImported", { id: roleId })));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

function acceptAllPermsOrCancel(title: string, perms: string[]): string[] | null {
  const list = perms.map((s) => s.trim()).filter(Boolean);
  if (list.length === 0) return [];
  const ok = window.confirm(
    String(t("pluginMarketV1.perms.confirmGrantAll", { title, list: list.join("\n") })),
  );
  return ok ? list : null;
}

async function onInstallPluginArchiveFromLocal(zipPath: string): Promise<void> {
  try {
    const isOclivePlugin = zipPath.toLowerCase().endsWith(".oclive-plugin");
    if (isOclivePlugin) {
      const it = localImports.value.find((x) => x.path === zipPath);
      const prev = await previewLocalPluginArchive({
        archivePath: zipPath,
        signaturePath: it?.relatedSignaturePath ?? null,
      });
      const accepted = acceptAllPermsOrCancel(
        String(t("pluginMarketV1.install.offlineBundleTitle", { id: prev.pluginId })),
        prev.declaredPermissions,
      );
      if (accepted == null) return;
      const overwrite = window.confirm(
        String(t("pluginMarketV1.install.confirmOverwritePlugin", { id: prev.pluginId })),
      );
      const pid = await installLocalPluginArchive({
        archivePath: zipPath,
        signaturePath: it?.relatedSignaturePath ?? null,
        overwrite,
        acceptedPermissions: accepted,
      });
      showToast("success", String(t("pluginMarketV1.install.toastInstalled", { id: pid })));
      await pluginStore.refresh();
      return;
    }

    const prev = await previewPluginZipPermissions(zipPath);
    const accepted = acceptAllPermsOrCancel(String(t("pluginMarketV1.install.zipTitle", { id: prev.pluginId })), prev.permissions);
    if (accepted == null) return;
    await pluginStore.installPluginFromLocalZip(prev.pluginId, zipPath, accepted);
    showToast("success", String(t("pluginMarketV1.install.toastInstalled", { id: prev.pluginId })));
    await pluginStore.refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallPluginDirFromLocal(dirPath: string): Promise<void> {
  try {
    const prev = await previewPluginDirPermissions(dirPath);
    const accepted = acceptAllPermsOrCancel(String(t("pluginMarketV1.install.dirTitle", { id: prev.pluginId })), prev.permissions);
    if (accepted == null) return;
    await installPluginDir(dirPath, prev.pluginId, accepted);
    showToast("success", String(t("pluginMarketV1.install.toastInstalled", { id: prev.pluginId })));
    await pluginStore.refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

watch(
  () => pluginStore.marketPanelVisible,
  (vis) => {
    if (!vis) return;
    void refreshLocalImports();
    void m.bootstrapMarketData();
  },
);
</script>

<template>
  <Teleport to="body">
    <div
      v-if="pluginStore.marketPanelVisible"
      class="pm2-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="String(t('pluginMarketV1.panel.dialogLabel'))"
      @click.self="pluginStore.closeMarketPanel()"
    >
      <div class="pm2-dialog" @click.stop>
        <div class="pmx-head">
          <div class="pmx-title">{{ t("pluginMarketV1.panel.title") }}</div>
          <button type="button" class="pm2-btn secondary" @click="pluginStore.closeMarketPanel()">
            {{ t("common.close") }}
          </button>
        </div>

        <section id="pm-community-index" class="pmx-section">
          <PluginMarketV2Pane />
        </section>

        <section class="pmx-section">
          <h3 class="pmx-h3">{{ t("pluginMarketV1.localImports.title") }}</h3>
          <p class="pmx-muted">
            {{ t("pluginMarketV1.localImports.hint") }}
          </p>
          <p v-if="localImportsRootDir" class="pmx-muted">
            {{ t("pluginMarketV1.localImports.rootLabel") }}：<code>{{ localImportsRootDir }}</code>
          </p>
          <div class="pmx-row">
            <button
              type="button"
              class="pm2-btn secondary"
              :disabled="localImportsLoading"
              @click="refreshLocalImports"
            >
              {{
                localImportsLoading
                  ? t("pluginMarketV1.localImports.scanning")
                  : t("pluginMarketV1.localImports.scan")
              }}
            </button>
            <span v-if="localImportsErr" class="pmx-err">{{ localImportsErr }}</span>
          </div>

          <div v-if="localImports.length === 0" class="pmx-muted">{{ t("pluginMarketV1.localImports.empty") }}</div>
          <div v-else class="pmx-local-grid">
            <div v-for="[kind, items] in localImportsByKind.entries()" :key="kind" class="pmx-local-col">
              <h4 class="pmx-h4">{{ localKindLabel(kind) }}</h4>
              <ul class="pmx-local-list">
                <li v-for="it in items" :key="it.path" class="pmx-local-li">
                  <code class="pmx-mono">{{ it.fileName }}</code>
                  <div class="pmx-local-actions">
                    <button
                      v-if="kind === 'role_pack'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onImportRolePackFromLocal(it.path, false)"
                    >
                      {{ t("pluginMarketV1.localImports.actions.import") }}
                    </button>
                    <button
                      v-if="kind === 'role_pack'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onImportRolePackFromLocal(it.path, true)"
                    >
                      {{ t("pluginMarketV1.localImports.actions.overwriteImport") }}
                    </button>

                    <button
                      v-if="kind === 'plugin_archive'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onInstallPluginArchiveFromLocal(it.path)"
                    >
                      {{ t("pluginMarketV1.localImports.actions.install") }}
                    </button>
                    <button
                      v-if="kind === 'plugin_dir'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onInstallPluginDirFromLocal(it.path)"
                    >
                      {{ t("pluginMarketV1.localImports.actions.install") }}
                    </button>

                    <button
                      v-if="kind === 'module_json' || kind === 'profile_json'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onApplyLocalModuleOrProfile(it.path)"
                    >
                      {{ t("pluginMarketV1.localImports.actions.apply") }}
                    </button>
                    <button
                      v-if="kind === 'module_json' || kind === 'profile_json'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onPreviewLocalJson(it.path)"
                    >
                      {{ t("pluginMarketV1.localImports.actions.copyJson") }}
                    </button>
                  </div>
                </li>
              </ul>
            </div>
          </div>
        </section>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pm2-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10060;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: color-mix(in srgb, #000 45%, transparent);
}
.pm2-dialog {
  width: min(1220px, 100%);
  min-height: min(620px, 88vh);
  max-height: min(92vh, 920px);
  display: flex;
  flex-direction: column;
  overflow: auto;
  padding: 14px 16px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pmx-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}
.pmx-title {
  font-size: 18px;
  font-weight: 700;
}
.pmx-section {
  padding: 12px 0;
  border-top: 1px solid var(--border-light);
}
.pmx-h3 {
  margin: 0 0 6px;
  font-size: 16px;
}
.pmx-h4 {
  margin: 0 0 8px;
  font-size: 14px;
}
.pmx-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 8px 0 12px;
}
.pmx-muted {
  margin: 6px 0 0;
  color: var(--text-muted);
  font-size: 13px;
}
.pmx-err {
  color: var(--color-danger, #d33);
  font-size: 13px;
}
.pmx-local-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin-top: 10px;
}
@media (max-width: 980px) {
  .pmx-local-grid {
    grid-template-columns: 1fr;
  }
}
.pmx-local-col {
  border: 1px solid var(--border-light);
  border-radius: 10px;
  padding: 10px;
  background: var(--bg-secondary);
}
.pmx-local-list {
  list-style: none;
  padding: 0;
  margin: 0;
}
.pmx-local-li {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px 0;
  border-top: 1px dashed var(--border-light);
}
.pmx-local-li:first-child {
  border-top: 0;
  padding-top: 0;
}
.pmx-local-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.pmx-mono {
  word-break: break-all;
}
</style>

