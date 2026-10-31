// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { createApp } from 'vue'
import { useDiagnosticHistory } from '../composables/useDiagnosticHistory'
import { i18n } from '../i18n'
import { installAppErrorReporting } from './appErrorReporting'

describe('production error reporting', () => {
  it('records Vue and rejection errors locally and releases window listeners on unmount', () => {
    const add = vi.spyOn(window, 'addEventListener')
    const remove = vi.spyOn(window, 'removeEventListener')
    const app = createApp({ render: () => null })
    app.use(i18n)
    installAppErrorReporting(app)
    const target = document.createElement('div')
    app.mount(target)
    app.config.errorHandler?.(new Error('fixture Vue failure\nAuthorization: Bearer hidden-vue-fixture'), null, 'render')
    const handler = add.mock.calls.find(([type]) => type === 'unhandledrejection')?.[1] as EventListener
    const rejection = new Event('unhandledrejection')
    Object.defineProperty(rejection, 'reason', { value: new Error('fixture rejected promise') })
    handler(rejection)
    const entries = useDiagnosticHistory().entries.value
    expect(entries.some(entry => entry.source === 'unhandled-rejection')).toBe(true)
    expect(entries.some(entry => entry.source === 'vue')).toBe(true)
    expect(JSON.stringify(entries)).not.toContain('hidden-vue-fixture')
    app.unmount()
    expect(remove).toHaveBeenCalledWith('unhandledrejection', handler)
    add.mockRestore()
    remove.mockRestore()
  })
})
