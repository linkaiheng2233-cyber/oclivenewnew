<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { useModalFocusRestore } from '../composables/useModalFocusRestore'
import { createPluginScaffold } from '../utils/tauri-api'

const props = defineProps<{ visible: boolean }>()

const emit = defineEmits<{
  close: []
  created: [pluginDir: string]
}>()

const { t, locale } = useI18n()

const visibleRef = toRef(props, 'visible')
const dialogRef = ref<HTMLElement | null>(null)
const primaryBtnRef = ref<HTMLButtonElement | null>(null)

useModalFocusRestore(visibleRef, dialogRef, { primary: primaryBtnRef })

const pluginId = ref('')
const pluginName = ref('')
const language = ref<'node' | 'python' | 'rust'>('node')
const pluginType = ref<'skill' | 'agent' | 'module_ext'>('skill')
const baseDir = ref('')
const busy = ref(false)
const status = ref('')

const manifestPreview = computed(() => {
  return {
    id: pluginId.value.trim(),
    name: pluginName.value.trim(),
    version: '0.1.0',
    runtime: language.value,
    type: pluginType.value,
    process: language.value === 'rust' ? 'target/debug/plugin_scaffold' : 'node index.js',
    permissions: ['process:spawn'],
  }
})

const allowedPermissions = ['process:spawn', 'network:*', 'mcp:http', 'mcp:stdio']

const manifestErrors = computed(() => {
  void locale.value
  const errs: string[] = []
  const v = manifestPreview.value
  if (!v.id)
    errs.push(t('devTools.scaffold.errId'))
  if (!v.name)
    errs.push(t('devTools.scaffold.errName'))
  if (!v.version)
    errs.push(t('devTools.scaffold.errVersion'))
  if (!v.process)
    errs.push(t('devTools.scaffold.errProcess'))
  for (const p of v.permissions) {
    if (!allowedPermissions.includes(p)) {
      errs.push(t('devTools.scaffold.errPerm', { p }))
    }
  }
  return errs
})

async function onCreate(): Promise<void> {
  if (manifestErrors.value.length > 0)
    return
  busy.value = true
  status.value = ''
  try {
    const r = await createPluginScaffold({
      pluginId: pluginId.value.trim(),
      pluginName: pluginName.value.trim(),
      language: language.value,
      pluginType: pluginType.value,
      baseDir: baseDir.value.trim() || undefined,
    })
    status.value = t('devTools.scaffold.createdAt', { dir: r.plugin_dir })
    emit('created', r.plugin_dir)
  }
  catch (e) {
    status.value = e instanceof Error ? e.message : String(e)
  }
  finally {
    busy.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="props.visible" class="psw-backdrop" @click.self="emit('close')">
      <div
        ref="dialogRef"
        class="psw-dialog"
        tabindex="-1"
        @click.stop
        @keydown.escape.stop="emit('close')"
      >
        <header class="psw-head">
          <h3>{{ t("devTools.scaffold.title") }}</h3>
          <button type="button" class="psw-close" @click="emit('close')">
            ×
          </button>
        </header>
        <div class="psw-body">
          <label>{{ t("devTools.scaffold.fieldId") }} <input v-model="pluginId" class="psw-input" placeholder="com.example.demo"></label>
          <label>{{ t("devTools.scaffold.fieldName") }} <input v-model="pluginName" class="psw-input" placeholder="Demo Plugin"></label>
          <label>{{ t("devTools.scaffold.fieldLang") }}
            <select v-model="language" class="psw-input">
              <option value="node">Node.js</option>
              <option value="python">Python</option>
              <option value="rust">Rust</option>
            </select>
          </label>
          <label>{{ t("devTools.scaffold.fieldType") }}
            <select v-model="pluginType" class="psw-input">
              <option value="skill">Skill</option>
              <option value="agent">Agent</option>
              <option value="module_ext">{{ t("devTools.scaffold.typeModuleExt") }}</option>
            </select>
          </label>
          <label>{{ t("devTools.scaffold.fieldOutDir") }} <input v-model="baseDir" class="psw-input" :placeholder="t('devTools.scaffold.outDirPh')"></label>

          <h4 class="psw-sub">
            {{ t("devTools.scaffold.manifestLive") }}
          </h4>
          <pre class="psw-pre">{{ JSON.stringify(manifestPreview, null, 2) }}</pre>
          <ul v-if="manifestErrors.length" class="psw-errs">
            <li v-for="e in manifestErrors" :key="e">
              {{ e }}
            </li>
          </ul>
          <p v-else class="psw-ok">
            {{ t("devTools.scaffold.manifestOk") }}
          </p>
        </div>
        <footer class="psw-foot">
          <span class="psw-status">{{ status }}</span>
          <button ref="primaryBtnRef" type="button" class="psw-btn" :disabled="busy || manifestErrors.length > 0" @click="onCreate">
            {{ t("devTools.scaffold.generate") }}
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
