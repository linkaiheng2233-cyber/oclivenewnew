<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { createPluginScaffold } from "../utils/tauri-api";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
  close: [];
  created: [pluginDir: string];
}>();

const pluginId = ref("");
const pluginName = ref("");
const language = ref<"node" | "python" | "rust">("node");
const pluginType = ref<"skill" | "agent" | "module_ext">("skill");
const baseDir = ref("");
const busy = ref(false);
const status = ref("");
const { t } = useI18n();

const manifestPreview = computed(() => {
  return {
    id: pluginId.value.trim(),
    name: pluginName.value.trim(),
    version: "0.1.0",
    runtime: language.value,
    type: pluginType.value,
    process: language.value === "rust" ? "target/debug/plugin_scaffold" : "node index.js",
    permissions: ["network"],
  };
});

const allowedPermissions = ["network", "fs", "clipboard", "shell"];

const manifestErrors = computed(() => {
  const errs: string[] = [];
  const v = manifestPreview.value;
  if (!v.id) errs.push(String(t("pluginScaffoldWizard.errors.missingField", { field: "id" })));
  if (!v.name) errs.push(String(t("pluginScaffoldWizard.errors.missingField", { field: "name" })));
  if (!v.version) errs.push(String(t("pluginScaffoldWizard.errors.missingField", { field: "version" })));
  if (!v.process) errs.push(String(t("pluginScaffoldWizard.errors.missingProcessOrRemoteUrl")));
  for (const p of v.permissions) {
    if (!allowedPermissions.includes(p)) {
      errs.push(String(t("pluginScaffoldWizard.errors.invalidPermission", { p })));
    }
  }
  return errs;
});

async function onCreate(): Promise<void> {
  if (manifestErrors.value.length > 0) return;
  busy.value = true;
  status.value = "";
  try {
    const r = await createPluginScaffold({
      pluginId: pluginId.value.trim(),
      pluginName: pluginName.value.trim(),
      language: language.value,
      pluginType: pluginType.value,
      baseDir: baseDir.value.trim() || undefined,
    });
    status.value = String(t("pluginScaffoldWizard.status.created", { dir: r.plugin_dir }));
    emit("created", r.plugin_dir);
  } catch (e) {
    status.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="props.visible" class="psw-backdrop" @click.self="emit('close')">
      <div class="psw-dialog">
        <header class="psw-head">
          <h3>{{ t("pluginScaffoldWizard.title") }}</h3>
          <button type="button" class="psw-close" :aria-label="String(t('common.close'))" @click="emit('close')">×</button>
        </header>
        <div class="psw-body">
          <label>{{ t("pluginScaffoldWizard.fields.id") }} <input v-model="pluginId" class="psw-input" placeholder="com.example.demo" /></label>
          <label>{{ t("pluginScaffoldWizard.fields.name") }} <input v-model="pluginName" class="psw-input" placeholder="Demo Plugin" /></label>
          <label>{{ t("pluginScaffoldWizard.fields.language") }}
            <select v-model="language" class="psw-input">
              <option value="node">Node.js</option>
              <option value="python">Python</option>
              <option value="rust">Rust</option>
            </select>
          </label>
          <label>{{ t("pluginScaffoldWizard.fields.type") }}
            <select v-model="pluginType" class="psw-input">
              <option value="skill">Skill</option>
              <option value="agent">Agent</option>
              <option value="module_ext">{{ t("pluginScaffoldWizard.types.moduleExt") }}</option>
            </select>
          </label>
          <label>{{ t("pluginScaffoldWizard.fields.outputDirOptional") }} <input v-model="baseDir" class="psw-input" :placeholder="String(t('pluginScaffoldWizard.fields.outputDirPlaceholder'))" /></label>

          <h4 class="psw-sub">{{ t("pluginScaffoldWizard.validation.title") }}</h4>
          <pre class="psw-pre">{{ JSON.stringify(manifestPreview, null, 2) }}</pre>
          <ul v-if="manifestErrors.length" class="psw-errs">
            <li v-for="e in manifestErrors" :key="e">{{ e }}</li>
          </ul>
          <p v-else class="psw-ok">{{ t("pluginScaffoldWizard.validation.ok") }}</p>
        </div>
        <footer class="psw-foot">
          <span class="psw-status">{{ status }}</span>
          <button type="button" class="psw-btn" :disabled="busy || manifestErrors.length > 0" @click="onCreate">
            {{ t("pluginScaffoldWizard.actions.create") }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.psw-backdrop { position: fixed; inset: 0; background: #0008; display: grid; place-items: center; z-index: 10080; }
.psw-dialog { width: min(760px, 92vw); max-height: 86vh; display: flex; flex-direction: column; background: var(--bg-primary); border: 1px solid var(--border-light); border-radius: 12px; }
.psw-head,.psw-foot { display:flex; align-items:center; justify-content:space-between; padding:10px 12px; border-bottom:1px solid var(--border-light);}
.psw-foot { border-top:1px solid var(--border-light); border-bottom:none; }
.psw-body { padding: 10px 12px; overflow: auto; display:grid; gap:8px; }
.psw-input { width:100%; box-sizing:border-box; padding:6px 8px; border:1px solid var(--border-light); border-radius:8px; background:var(--bg-elevated);}
.psw-pre { margin:0; padding:8px; border:1px solid var(--border-light); border-radius:8px; background:var(--panel-bg-soft); font-size:12px;}
.psw-errs { margin:0; color:var(--error); font-size:12px; padding-left: 18px;}
.psw-ok { margin:0; font-size:12px; color:#2f9e44;}
.psw-btn { border:1px solid var(--border-light); border-radius:8px; padding:6px 12px; }
.psw-sub { margin: 6px 0 0; font-size: 13px; }
.psw-status { font-size: 12px; color: var(--text-secondary); }
.psw-close { border:none; background:transparent; font-size:18px; cursor:pointer; }
</style>
