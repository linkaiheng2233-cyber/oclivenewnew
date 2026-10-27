import type { EnvironmentDiagnostics, SupportLogSnapshot } from '@oclive/shared/api/diagnostics'
import { writeUserTextFile } from '@oclive/shared/api/desktop'
import { readSupportLogs, runEnvironmentDiagnostics } from '@oclive/shared/api/diagnostics'
import { openPathInFileManager } from '@oclive/shared/api/llmSettings'
import { useKernelConnectionStore } from '@oclive/shared/stores/kernelConnectionStore'
import { redactDiagnosticText } from '@oclive/shared/utils/diagnosticText'
import { save } from '@tauri-apps/plugin-dialog'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppToast } from './useAppToast'
import { useDiagnosticHistory } from './useDiagnosticHistory'

export function useSupportDiagnostics(active: () => boolean) {
  const { t } = useI18n()
  const { showToast } = useAppToast()
  const { entries } = useDiagnosticHistory()
  const kernel = useKernelConnectionStore()
  const snapshot = ref<SupportLogSnapshot | null>(null)
  const environment = ref<EnvironmentDiagnostics | null>(null)
  const loading = ref(false)
  const checking = ref(false)
  const exporting = ref(false)
  const error = ref<unknown>(null)
  const search = ref('')
  const level = ref('all')
  let logGeneration = 0
  let checkGeneration = 0

  const logText = computed(() => redactDiagnosticText(snapshot.value?.text ?? ''))
  const logLines = computed(() => logText.value.split('\n').filter(Boolean).map((text, index) => {
    let severity = /\bERROR\b|"level"\s*:\s*"error"/i.test(text) ? 'error' : /\bWARN(?:ING)?\b|"level"\s*:\s*"warn(?:ing)?"/i.test(text) ? 'warning' : 'info'
    if (/\bDEBUG\b|\bTRACE\b|"level"\s*:\s*"(?:debug|trace)"/i.test(text))
      severity = 'debug'
    return { text, severity, index }
  }))
  const filteredLines = computed(() => logLines.value.filter(line => (level.value === 'all' || line.severity === level.value)
    && line.text.toLocaleLowerCase().includes(search.value.trim().toLocaleLowerCase())))

  async function refreshLogs(fileName?: string): Promise<void> {
    const generation = ++logGeneration
    loading.value = true
    error.value = null
    try {
      const result = await readSupportLogs(fileName)
      if (generation === logGeneration && active())
        snapshot.value = result
    }
    catch (failure) {
      if (generation === logGeneration && active())
        error.value = failure
    }
    finally {
      if (generation === logGeneration)
        loading.value = false
    }
  }

  async function checkEnvironment(): Promise<void> {
    const generation = ++checkGeneration
    checking.value = true
    error.value = null
    try {
      const result = await runEnvironmentDiagnostics()
      if (generation === checkGeneration && active())
        environment.value = result
    }
    catch (failure) {
      if (generation === checkGeneration && active())
        error.value = failure
    }
    finally {
      if (generation === checkGeneration)
        checking.value = false
    }
  }

  function reportText(): string {
    return redactDiagnosticText([
      `OCLive Chat Pro — ${t('support.title')}`,
      new Date().toISOString(),
      `${t('support.status')}: ${JSON.stringify(kernel.status)}`,
      environment.value ? JSON.stringify(environment.value, null, 2) : '',
      `\n${t('support.errors')} (${t('support.sessionOnly')})`,
      ...entries.value.map(entry => `${new Date(entry.at).toISOString()} [${entry.level}] ${entry.source} ${entry.code ?? ''}\n${entry.summary}\n${entry.details}`),
      `\n${t('support.logs')}: ${snapshot.value?.directory ?? ''} / ${snapshot.value?.selectedFile ?? ''}`,
      snapshot.value?.truncated ? t('support.truncated') : '',
      logText.value,
    ].filter(Boolean).join('\n\n'))
  }

  async function copyReport(): Promise<void> {
    try {
      await navigator.clipboard.writeText(reportText())
      showToast('success', t('support.copied'))
    }
    catch (failure) {
      error.value = failure
    }
  }

  async function openLogDirectory(): Promise<void> {
    if (!snapshot.value?.directory)
      return
    try {
      await openPathInFileManager(snapshot.value.directory)
    }
    catch (failure) {
      error.value = failure
    }
  }

  async function exportReport(): Promise<void> {
    if (exporting.value)
      return
    exporting.value = true
    try {
      const content = reportText()
      const path = await save({ defaultPath: `oclive-diagnostics-${new Date().toISOString().slice(0, 10)}.txt`, filters: [{ name: 'Text', extensions: ['txt'] }] })
      if (!path)
        return
      await writeUserTextFile(path, content)
      showToast('success', t('support.saved'))
    }
    catch (failure) {
      error.value = failure
    }
    finally {
      exporting.value = false
    }
  }

  function invalidate(): void {
    logGeneration++
    checkGeneration++
    loading.value = false
    checking.value = false
  }
  watch(active, (value) => {
    if (value)
      void refreshLogs()
    else
      invalidate()
  }, { immediate: true })
  onBeforeUnmount(invalidate)

  return { entries, snapshot, environment, loading, checking, exporting, error, search, level, filteredLines, refreshLogs, checkEnvironment, copyReport, exportReport, openLogDirectory, reportText }
}
