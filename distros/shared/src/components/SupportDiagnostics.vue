<script setup lang="ts">
import { useSupportDiagnostics } from '@oclive/shared/composables/useSupportDiagnostics'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ErrorNotice from './ErrorNotice.vue'
import KernelConnectionSettingsPanel from './settings/KernelConnectionSettingsPanel.vue'

const props = defineProps<{ active: boolean }>()
const { t } = useI18n()
const tab = ref('status')
const { entries, snapshot, environment, loading, checking, exporting, error, search, level, filteredLines, refreshLogs, checkEnvironment, copyReport, exportReport, openLogDirectory } = useSupportDiagnostics(() => props.active)
const tabs = ['status', 'errors', 'logs'] as const
</script>

<template>
  <div class="support-diagnostics">
    <div class="support-toolbar">
      <div class="support-tabs" role="tablist" :aria-label="t('support.title')">
        <button v-for="item in tabs" :id="`support-tab-${item}`" :key="item" type="button" role="tab" :aria-selected="tab === item" :aria-controls="`support-panel-${item}`" @click="tab = item">
          {{ t(`support.${item}`) }}<span v-if="item === 'errors'" class="support-count">{{ entries.length }}</span>
        </button>
      </div>
      <div class="support-actions">
        <button type="button" @click="copyReport">
          {{ t('support.copy') }}
        </button>
        <button type="button" :disabled="exporting" @click="exportReport">
          {{ t('support.export') }}
        </button>
      </div>
    </div>
    <p class="support-note support-share-note">
      {{ t('support.shareHint') }}
    </p>
    <div class="support-scroll" :class="{ 'support-scroll--logs': tab === 'logs' }">
      <ErrorNotice v-if="error" :error="error" dismissible @dismiss="error = null" />

      <section v-if="tab === 'status'" id="support-panel-status" role="tabpanel" aria-labelledby="support-tab-status">
        <KernelConnectionSettingsPanel :active="active && tab === 'status'" />
        <div class="support-check-header">
          <h3>{{ t('support.environment') }}</h3>
          <button type="button" :disabled="checking" @click="checkEnvironment">
            {{ checking ? t('support.checking') : t('support.check') }}
          </button>
        </div>
        <p v-if="!environment" class="support-note">
          {{ t('support.checkHint') }}
        </p>
        <dl v-else class="support-facts">
          <div>
            <dt>{{ t('support.modelService') }}</dt><dd :class="environment.ollamaReachable ? 'support-ok' : 'support-bad'">
              {{ t(environment.ollamaReachable ? 'support.normal' : 'support.attention') }}
            </dd><code>{{ environment.ollamaBaseUrl }}</code>
          </div>
          <div>
            <dt>{{ t('support.roleDirectory') }}</dt><dd :class="environment.rolesDirReadable ? 'support-ok' : 'support-bad'">
              {{ t(environment.rolesDirReadable ? 'support.normal' : 'support.attention') }}
            </dd><code>{{ environment.rolesDir }}</code>
          </div>
          <div>
            <dt>{{ t('support.dataDirectory') }}</dt><dd :class="environment.appDataWritable ? 'support-ok' : 'support-bad'">
              {{ t(environment.appDataWritable ? 'support.normal' : 'support.attention') }}
            </dd><code>{{ environment.appDataDir }}</code>
          </div>
        </dl>
        <ErrorNotice v-if="environment && !environment.ollamaReachable" :error="environment.ollamaDetail" :summary="t('support.modelUnavailable')" />
        <ErrorNotice v-if="environment && !environment.appDataWritable" :error="environment.appDataDetail" :summary="t('support.dataUnavailable')" />
      </section>

      <section v-else-if="tab === 'errors'" id="support-panel-errors" role="tabpanel" aria-labelledby="support-tab-errors">
        <p class="support-note">
          {{ t('support.historyHint') }}
        </p>
        <div v-if="!entries.length" class="support-empty">
          {{ t('support.noErrors') }}
        </div>
        <ol v-else class="support-history">
          <li v-for="entry in entries" :key="entry.id" :class="`support-event--${entry.level}`">
            <div class="support-event-meta">
              <time :datetime="new Date(entry.at).toISOString()">{{ new Date(entry.at).toLocaleString() }}</time><code>{{ entry.source }}</code><span :class="`support-level--${entry.level}`">{{ t(`support.${entry.level}`) }}</span><code v-if="entry.code">{{ entry.code }}</code>
            </div>
            <ErrorNotice :summary="entry.summary" :details="entry.details" />
          </li>
        </ol>
      </section>

      <section v-else id="support-panel-logs" role="tabpanel" aria-labelledby="support-tab-logs" :aria-busy="loading">
        <div class="support-log-controls">
          <label>
            {{ t('support.file') }}
            <select :value="snapshot?.selectedFile ?? ''" :aria-label="t('support.file')" :disabled="loading || !snapshot?.files.length" @change="refreshLogs(($event.target as HTMLSelectElement).value)">
              <option v-if="!snapshot?.files.length" value="">{{ t('support.noFiles') }}</option>
              <option v-for="file in snapshot?.files" :key="file.name" :value="file.name">{{ file.name }} · {{ Math.ceil(file.sizeBytes / 1024) }} KB</option>
            </select>
          </label>
          <label>
            {{ t('support.level') }}
            <select v-model="level" :aria-label="t('support.level')">
              <option v-for="severity in ['all', 'error', 'warning', 'info', 'debug']" :key="severity" :value="severity">{{ t(`support.${severity}`) }}</option>
            </select>
          </label>
          <label class="support-search">{{ t('support.search') }}<input v-model="search" type="search" :placeholder="t('support.searchHint')"></label>
          <button type="button" :disabled="loading" @click="refreshLogs()">
            {{ loading ? t('support.loading') : t('support.refresh') }}
          </button>
        </div>
        <p class="support-note">
          {{ t('support.tailHint') }}
        </p>
        <p v-if="snapshot?.directory" class="support-log-path">
          <code>{{ snapshot.directory }}</code> <button type="button" :disabled="!snapshot.files.length" @click="openLogDirectory">
            {{ t('support.openDirectory') }}
          </button>
        </p>
        <p v-if="snapshot?.truncated" class="support-limit" role="status">
          {{ t('support.truncated') }}
        </p>
        <div v-if="!filteredLines.length" class="support-empty">
          {{ loading ? t('support.loading') : snapshot?.text ? t('support.noMatch') : t('support.noLogs') }}
        </div>
        <div v-else class="support-log-view" tabindex="0" :aria-label="t('support.logs')">
          <div v-for="line in filteredLines" :key="line.index" class="support-log-line" :class="`support-log-line--${line.severity}`">
            <span aria-hidden="true">{{ line.index + 1 }}</span><code>{{ line.text }}</code>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.support-diagnostics {
  --support-border: color-mix(in srgb, var(--accent, #486b62) 12%, var(--border-light));
  --support-muted: color-mix(in srgb, var(--bg-secondary) 65%, var(--bg-primary));
  --support-thumb: var(--scrollbar-chat-thumb, #486b6240);
  display:flex; flex:1; flex-direction:column; min-width:0; min-height:0; overflow:hidden; font-size:14px;
}
.support-diagnostics button { padding:8px 12px; min-height:36px; border:1px solid var(--support-border); border-radius:8px; background:var(--bg-elevated,var(--bg-primary)); color:var(--text-primary); font:inherit; font-size:12px; font-weight:500; cursor:pointer; transition:background .14s,border-color .14s; }
.support-diagnostics button:hover:not(:disabled) { border-color:var(--accent,#486b62); background:var(--support-muted); }
.support-diagnostics button:disabled { opacity:.5; cursor:default; }
.support-diagnostics button:focus-visible,.support-diagnostics select:focus-visible,.support-diagnostics input:focus-visible,.support-log-view:focus-visible { outline:2px solid var(--accent,#486b62); outline-offset:2px; }
.support-toolbar,.support-actions,.support-check-header,.support-event-meta { display:flex; align-items:center; gap:10px; flex-wrap:wrap; }
.support-toolbar { flex-shrink:0; justify-content:space-between; padding:16px 24px 10px; gap:12px; }
.support-tabs { display:flex; gap:4px; padding:4px; border:1px solid var(--support-border); background:var(--support-muted); border-radius:11px; }
.support-tabs button { border-color:transparent; background:transparent; min-height:34px; padding:7px 12px; }
.support-tabs button[aria-selected=true] { background:var(--accent,#486b62); color:var(--bg-primary,#fff); box-shadow:0 1px 3px #00000012; }
.support-tabs button[aria-selected=true]:hover { background:var(--accent,#486b62); }
.support-count { display:inline-block; min-width:18px; padding:0 4px; margin-left:7px; font-size:10px; line-height:18px; border-radius:5px; background:color-mix(in srgb,currentColor 12%,transparent); font-variant-numeric:tabular-nums; }
.support-note { font-size:12px; line-height:1.8; color:var(--text-secondary); margin:12px 0; }
.support-share-note { flex-shrink:0; margin:0; padding:0 24px 14px; border-bottom:1px solid var(--support-border); }
.support-scroll { flex:1; min-height:0; overflow:auto; padding:20px 24px; overscroll-behavior:contain; }
.support-scroll>[role=alert] { margin-bottom:16px; }
.support-scroll>[role=tabpanel] { padding:18px; border:1px solid var(--support-border); border-radius:12px; background:var(--bg-elevated,var(--bg-primary)); min-width:0; }
.support-check-header { justify-content:space-between; padding-top:20px; margin-top:22px; border-top:1px solid var(--support-border); }
h3 { font-size:14px; font-weight:650; margin:0; }
.support-empty { padding:34px 20px; text-align:center; background:var(--support-muted); border:1px dashed var(--support-border); border-radius:10px; color:var(--text-secondary); line-height:1.8; }
.support-facts { display:grid; gap:10px; }
.support-facts>div { display:grid; grid-template-columns:1fr auto; gap:8px 12px; padding:14px; border:1px solid var(--support-border); border-radius:9px; background:var(--support-muted); }
.support-facts dd { margin:0; font-weight:600; font-size:12px; }
.support-facts code { grid-column:1/-1; font-size:12px; line-height:1.7; overflow-wrap:anywhere; }
.support-ok { color:var(--success,#397753); }.support-bad { color:var(--error,#aa4e42); }
.support-history { padding:0; margin:16px 0 0; list-style:none; display:grid; gap:14px; }
.support-history>li { overflow:hidden; border:1px solid var(--support-border); border-left:3px solid var(--error,#aa4e42); border-radius:10px; background:var(--bg-primary); }
.support-history>li.support-event--warning { border-left-color:var(--warning,#a46925); }
.support-event-meta { gap:8px; padding:11px 14px; border-bottom:1px solid var(--support-border); background:var(--support-muted); font-size:11px; color:var(--text-secondary); }
.support-event-meta time { margin-right:auto; font-variant-numeric:tabular-nums; }
.support-event-meta>code { border:1px solid var(--support-border); border-radius:4px; padding:2px 5px; font:11px/1.4 Consolas,monospace; overflow-wrap:anywhere; }
.support-level--error { color:var(--error,#aa4e42); }.support-level--warning { color:var(--warning,#a46925); }
.support-history :deep(.error-notice) { border:0; border-radius:0; background:transparent; }
.support-diagnostics :deep(.error-notice-details) { border-top:1px solid var(--support-border); padding-top:10px; }
.support-diagnostics :deep(.error-notice-details pre) { border:1px solid var(--support-border); padding:12px; background:var(--support-muted); font-size:12px; line-height:1.75; }
.support-log-controls { display:grid; grid-template-columns:minmax(150px,1.2fr) 100px minmax(160px,1.5fr) auto; align-items:end; gap:12px; flex-shrink:0; }
.support-log-controls label { display:flex; flex-direction:column; gap:8px; font-size:12px; font-weight:550; color:var(--text-secondary); min-width:0; }
.support-log-controls select,.support-log-controls input { box-sizing:border-box; width:100%; height:38px; padding:8px 10px; border:1px solid var(--support-border); border-radius:7px; color:var(--text-primary); background:var(--bg-primary); min-width:0; font-size:12px; }
.support-log-controls>button { min-height:38px; }
.support-log-path { flex-shrink:0; display:flex; align-items:center; gap:12px; padding:10px 12px; margin:0 0 12px; border:1px solid var(--support-border); border-radius:8px; background:var(--support-muted); }
.support-log-path>code { flex:1; min-width:0; font:11px/1.7 Consolas,monospace; color:var(--text-secondary); overflow-wrap:anywhere; }
.support-log-path>button { flex-shrink:0; padding:5px 9px; min-height:30px; font-size:11px; }
.support-limit { font-size:12px; color:var(--warning,#a46925); line-height:1.8; margin:0 0 12px; }
.support-scroll--logs { display:flex; flex-direction:column; overflow:hidden; }
.support-scroll--logs>[role=tabpanel] { flex:1; min-height:0; display:flex; flex-direction:column; }
.support-scroll--logs .support-note,.support-limit { flex-shrink:0; }
.support-log-view { flex:1; min-height:100px; overflow:auto; border:1px solid var(--support-border); border-radius:9px; background:var(--support-muted); padding:8px 6px; user-select:text; overscroll-behavior:contain; }
.support-log-line { display:flex; gap:12px; padding:7px 8px; border-left:3px solid transparent; border-bottom:1px solid color-mix(in srgb,var(--support-border) 45%,transparent); }
.support-log-line:last-child { border-bottom:0; }
.support-log-line>span { flex-shrink:0; min-width:24px; text-align:right; color:var(--text-secondary); opacity:.65; font:11px/1.75 Consolas,monospace; }
.support-log-line>code { white-space:pre-wrap; overflow-wrap:anywhere; min-width:0; font:12px/1.75 Consolas,monospace; }
.support-log-line--error { border-left-color:var(--error,#aa4e42); background:color-mix(in srgb,var(--error,#aa4e42) 7%,transparent); }
.support-log-line--warning { border-left-color:var(--warning,#a46925); background:color-mix(in srgb,var(--warning,#a46925) 6%,transparent); }
.support-diagnostics :deep(.kernel-panel__facts) { padding:14px; border:1px solid var(--support-border); border-radius:9px; background:var(--support-muted); gap:12px; }
.support-diagnostics :deep(.kernel-panel__fact) { gap:8px; line-height:1.7; }
.support-diagnostics :deep(.kernel-panel__fact-key) { flex-shrink:0; min-width:105px; font-size:12px; color:var(--text-secondary); }
.support-diagnostics :deep(.kernel-panel__code) { min-width:0; font:12px/1.7 Consolas,monospace; overflow-wrap:anywhere; }
.support-diagnostics :deep(.kernel-panel__raw) { padding:12px 14px; border:1px solid var(--support-border); border-radius:9px; }
.support-diagnostics :deep(.kernel-panel__pre) { border:1px solid var(--support-border); white-space:pre-wrap; overflow-wrap:anywhere; line-height:1.7; }
.support-scroll,.support-log-view,.support-diagnostics :deep(pre) { scrollbar-width:thin; scrollbar-color:var(--support-thumb) transparent; scrollbar-gutter:stable; }
.support-scroll::-webkit-scrollbar,.support-log-view::-webkit-scrollbar,.support-diagnostics :deep(pre::-webkit-scrollbar) { width:10px; height:10px; }
.support-scroll::-webkit-scrollbar-track,.support-log-view::-webkit-scrollbar-track,.support-diagnostics :deep(pre::-webkit-scrollbar-track) { background:transparent; margin-block:6px; }
.support-scroll::-webkit-scrollbar-thumb,.support-log-view::-webkit-scrollbar-thumb,.support-diagnostics :deep(pre::-webkit-scrollbar-thumb) { background:var(--support-thumb); border:3px solid transparent; background-clip:padding-box; border-radius:12px; }
.support-scroll::-webkit-scrollbar-thumb:hover,.support-log-view::-webkit-scrollbar-thumb:hover,.support-diagnostics :deep(pre::-webkit-scrollbar-thumb:hover) { background-color:var(--scrollbar-chat-thumb-hover,#486b6260); }
.support-scroll::-webkit-scrollbar-corner,.support-log-view::-webkit-scrollbar-corner,.support-diagnostics :deep(pre::-webkit-scrollbar-corner) { background:transparent; }
@media(max-width:820px) { .support-log-controls{grid-template-columns:minmax(140px,1fr) 100px}.support-search{grid-column:1}.support-toolbar{align-items:flex-start}.support-actions{gap:6px} }
@media(max-width:600px) {
  .support-toolbar{padding:12px 14px 10px;gap:10px}.support-tabs{width:100%;box-sizing:border-box}.support-tabs>button{flex:1;padding:7px 5px}.support-actions{width:100%;justify-content:flex-end}.support-actions>button{flex:1}
  .support-share-note{padding:0 16px 12px;font-size:11px}.support-scroll{padding:14px}.support-scroll>[role=tabpanel]{padding:12px}.support-log-controls{gap:10px}.support-log-path{align-items:flex-start;flex-direction:column;gap:8px}
  .support-scroll--logs{overflow:auto;display:block}.support-scroll--logs>[role=tabpanel]{min-height:100%;box-sizing:border-box}.support-log-view{flex:none;height:280px;min-height:140px}.support-event-meta{padding:10px;gap:6px}.support-event-meta time{width:100%}.support-log-line{gap:8px;padding:6px 2px}
}
@media(max-height:650px) { .support-scroll--logs{overflow:auto;display:block}.support-log-view{flex:none;height:240px} }
</style>

<style>
@import '@oclive/shared/styles/win98/component-recovery.css';
</style>
