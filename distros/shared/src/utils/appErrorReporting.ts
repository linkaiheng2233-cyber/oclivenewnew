import type { App } from 'vue'
import { toErrorPresentation } from '@oclive/shared/api/helpers'
import { recordDiagnostic } from '@oclive/shared/composables/useDiagnosticHistory'
import { rt } from '@oclive/shared/i18n/runtimeT'

/** Session-local, bounded and redacted by the same diagnostics layer as API errors. */
export function reportAppFailure(error: unknown, source: string, context = ''): void {
  try {
    const display = toErrorPresentation(error, rt('support.applicationError'))
    recordDiagnostic({
      level: 'error',
      source,
      summary: display.summary,
      details: [context, display.details, error instanceof Error ? error.stack : ''].filter(Boolean).join('\n\n'),
    })
  }
  catch {
    recordDiagnostic({ level: 'error', source, summary: 'Application error', details: 'The original error could not be formatted.' })
  }
}

export function installAppErrorReporting(app: App): void {
  app.config.errorHandler = (error, _instance, info) => {
    reportAppFailure(error, 'vue', info)
  }
  if (typeof window === 'undefined')
    return
  const onError = (event: ErrorEvent) => reportAppFailure(event.error ?? event.message, 'window-error')
  const onRejection = (event: PromiseRejectionEvent) => reportAppFailure(event.reason, 'unhandled-rejection')
  window.addEventListener('error', onError)
  window.addEventListener('unhandledrejection', onRejection)
  app.onUnmount(() => {
    window.removeEventListener('error', onError)
    window.removeEventListener('unhandledrejection', onRejection)
  })
}
