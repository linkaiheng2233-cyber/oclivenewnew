<script setup lang="ts">
import { computed, ref, watch } from "vue";
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
  if (k === "role_pack") return "角色包";
  if (k === "plugin_archive") return "插件包";
  if (k === "plugin_dir") return "插件目录";
  if (k === "module_json") return "模块条目";
  if (k === "profile_json") return "Profile";
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
    throw new Error(`JSON 解析失败：${e instanceof Error ? e.message : String(e)}`);
  }
  if (!j || typeof j !== "object") {
    throw new Error("JSON 须为对象。");
  }
  const o = j as any;
  const t = String(o.type ?? "").trim();
  if (t !== "module" && t !== "profile") {
    throw new Error('本地条目 type 必须为 "module" 或 "profile"。');
  }
  const id = String(o.id ?? "").trim();
  const name = String(o.name ?? "").trim();
  const version = String(o.version ?? "").trim();
  if (!id || !name || !version) {
    throw new Error("本地条目必须包含 id/name/version。");
  }
  return o as PluginMarketEntryDto;
}

async function onPreviewLocalJson(path: string): Promise<void> {
  try {
    const text = await readLocalImportText(path);
    await navigator.clipboard.writeText(text);
    showToast("success", "已复制 JSON 内容到剪贴板。");
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
      showToast("error", "仅支持 module/profile 本地条目。");
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
        ? `覆盖导入角色包：${peek.name}（id=${peek.id} v${peek.version}）\n\n将替换本机已存在的同 id 角色包内容。确定继续吗？`
        : `导入角色包：${peek.name}（id=${peek.id} v${peek.version}）\n\n确定导入到本机 roles/ 吗？（默认不覆盖同 id）`,
    );
    if (!ok) return;
    const roleId = await importRolePack(path, overwrite);
    showToast("success", overwrite ? `覆盖导入成功：${roleId}` : `导入成功：${roleId}`);
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

function acceptAllPermsOrCancel(title: string, perms: string[]): string[] | null {
  const list = perms.map((s) => s.trim()).filter(Boolean);
  if (list.length === 0) return [];
  const ok = window.confirm(
    `${title}\n\n该插件声明权限：\n${list.join("\n")}\n\n继续则默认授予全部权限（安装后仍可在专业模式里调整）。`,
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
        `安装插件（离线包）：${prev.pluginId}`,
        prev.declaredPermissions,
      );
      if (accepted == null) return;
      const overwrite = window.confirm(
        `是否允许覆盖已存在的同 id 插件？\n\n插件：${prev.pluginId}\n\n“确定”=覆盖安装；“取消”=若已存在则报错。`,
      );
      const pid = await installLocalPluginArchive({
        archivePath: zipPath,
        signaturePath: it?.relatedSignaturePath ?? null,
        overwrite,
        acceptedPermissions: accepted,
      });
      showToast("success", `已安装：${pid}`);
      await pluginStore.refresh();
      return;
    }

    const prev = await previewPluginZipPermissions(zipPath);
    const accepted = acceptAllPermsOrCancel(`安装插件（ZIP）：${prev.pluginId}`, prev.permissions);
    if (accepted == null) return;
    await pluginStore.installPluginFromLocalZip(prev.pluginId, zipPath, accepted);
    showToast("success", `已安装：${prev.pluginId}`);
    await pluginStore.refresh();
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

async function onInstallPluginDirFromLocal(dirPath: string): Promise<void> {
  try {
    const prev = await previewPluginDirPermissions(dirPath);
    const accepted = acceptAllPermsOrCancel(`安装插件（目录）：${prev.pluginId}`, prev.permissions);
    if (accepted == null) return;
    await installPluginDir(dirPath, prev.pluginId, accepted);
    showToast("success", `已安装：${prev.pluginId}`);
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
      aria-label="插件市场"
      @click.self="pluginStore.closeMarketPanel()"
    >
      <div class="pm2-dialog" @click.stop>
        <div class="pmx-head">
          <div class="pmx-title">插件市场</div>
          <button type="button" class="pm2-btn secondary" @click="pluginStore.closeMarketPanel()">
            关闭
          </button>
        </div>

        <section id="pm-community-index" class="pmx-section">
          <PluginMarketV2Pane />
        </section>

        <section class="pmx-section">
          <h3 class="pmx-h3">本地导入（文件夹投放）</h3>
          <p class="pmx-muted">
            把文件放进投放目录后，点击“扫描投放目录”让应用发现它们，然后你再手动确认安装/导入。
          </p>
          <p v-if="localImportsRootDir" class="pmx-muted">
            根目录：<code>{{ localImportsRootDir }}</code>
          </p>
          <div class="pmx-row">
            <button
              type="button"
              class="pm2-btn secondary"
              :disabled="localImportsLoading"
              @click="refreshLocalImports"
            >
              {{ localImportsLoading ? "扫描中…" : "扫描投放目录" }}
            </button>
            <span v-if="localImportsErr" class="pmx-err">{{ localImportsErr }}</span>
          </div>

          <div v-if="localImports.length === 0" class="pmx-muted">暂无候选项。</div>
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
                      导入
                    </button>
                    <button
                      v-if="kind === 'role_pack'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onImportRolePackFromLocal(it.path, true)"
                    >
                      覆盖导入
                    </button>

                    <button
                      v-if="kind === 'plugin_archive'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onInstallPluginArchiveFromLocal(it.path)"
                    >
                      安装
                    </button>
                    <button
                      v-if="kind === 'plugin_dir'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onInstallPluginDirFromLocal(it.path)"
                    >
                      安装
                    </button>

                    <button
                      v-if="kind === 'module_json' || kind === 'profile_json'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onApplyLocalModuleOrProfile(it.path)"
                    >
                      应用
                    </button>
                    <button
                      v-if="kind === 'module_json' || kind === 'profile_json'"
                      type="button"
                      class="pm2-btn secondary pm2-btn--sm"
                      @click="onPreviewLocalJson(it.path)"
                    >
                      复制 JSON
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

