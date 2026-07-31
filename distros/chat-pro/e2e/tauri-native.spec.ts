/**
 * A1.1c minimal native-window smoke (Linux CI + local with tauri-driver).
 * Requires: `tauri-driver` on PATH, debug binary at TAURI_E2E_APP_PATH, optional WebKitWebDriver on Linux.
 * Dev dependency `webdriverio` is used here for WebDriver protocol (not Playwright).
 */
import { expect, test } from '@playwright/test'
import { remote } from 'webdriverio'

const appPath = process.env.TAURI_E2E_APP_PATH?.trim()
const driverHost = process.env.TAURI_DRIVER_HOST ?? '127.0.0.1'
const driverPort = Number(process.env.TAURI_DRIVER_PORT ?? '4444')
const shellPluginId = process.env.OCLIVE_SHELL_PLUGIN_ID?.trim() ?? ''
const isolationShellPluginId = 'com.oclive.example.minimal'

async function openNativeSession() {
  try {
    return await remote({
      hostname: driverHost,
      port: driverPort,
      path: '/',
      capabilities: {
        // WDIO 9+ injects BiDi `webSocketUrl`; Ubuntu WebKitWebDriver (pre-2.46) rejects/hangs.
        'wdio:enforceWebDriverClassic': true,
        'tauri:options': {
          application: appPath,
        },
      },
      // Keep retries at 0 (no silent multi-minute hangs); allow one long session create.
      connectionRetryCount: 0,
      connectionRetryTimeout: 120_000,
    })
  }
  catch (err) {
    const detail = err instanceof Error ? err.message : String(err)
    throw new Error(
      `tauri-driver session create failed (POST http://${driverHost}:${driverPort}/session): ${detail}`,
    )
  }
}

test.describe('Tauri native window (A1.1c smoke)', () => {
  test.skip(
    !appPath,
    'Set TAURI_E2E_APP_PATH to the debug oclivenewnew-tauri binary',
  )

  test('main window title and left sidebar pane', async () => {
    test.skip(
      shellPluginId.length > 0,
      'The main-shell smoke requires OCLIVE_SHELL_PLUGIN_ID to be unset',
    )
    // Playwright global/config timeout alone has been observed as 30s in CI; pin again here.
    test.setTimeout(180_000)

    const browser = await openNativeSession()

    try {
      await browser.setTimeout({ implicit: 15_000 })
      // FluentShell is defineAsyncComponent — wait for shell chrome, not only window title.
      await browser.waitUntil(
        async () => {
          const t = await browser.getTitle()
          return /OCLIVE|沐沐/i.test(t)
        },
        {
          timeout: 60_000,
          timeoutMsg: 'window title never matched OCLIVE / oclivenewnew / 沐沐',
        },
      )
      const title = await browser.getTitle()
      expect(title).toMatch(/OCLIVE|沐沐/i)

      const leftPane = await browser.$('.left-pane')
      // Use WebdriverIO wait (Playwright expect has no toBeDisplayed for WDIO elements).
      await leftPane.waitForDisplayed({ timeout: 60_000 })
      expect(await leftPane.isDisplayed()).toBe(true)

      const roleSelect = await browser.$('.selector-row--topbar select')
      await roleSelect.waitForDisplayed({ timeout: 30_000 })
      expect(await roleSelect.isDisplayed()).toBe(true)
    }
    finally {
      await browser.deleteSession()
    }
  })

  test('plugin isolation: full shell cannot access host DOM or Tauri IPC', async () => {
    test.skip(
      shellPluginId !== isolationShellPluginId,
      `Set OCLIVE_SHELL_PLUGIN_ID=${isolationShellPluginId} before starting tauri-driver`,
    )
    test.setTimeout(180_000)
    const browser = await openNativeSession()

    try {
      await browser.setTimeout({ implicit: 15_000 })
      const frame = await browser.$('#oclive-directory-shell-frame')
      await frame.waitForDisplayed({ timeout: 60_000 })
      expect(await frame.getAttribute('sandbox')).toBe('allow-scripts')
      const expectedShellUrl = process.platform === 'win32'
        ? 'https://ocliveplugin.localhost/com.oclive.example.minimal/ui/index.html'
        : 'ocliveplugin://localhost/com.oclive.example.minimal/ui/index.html'
      expect(await frame.getAttribute('src')).toBe(expectedShellUrl)
      expect(await browser.$('.left-pane').isExisting()).toBe(false)

      await browser.switchToFrame(frame)
      const heading = await browser.$('h1')
      await heading.waitForDisplayed({ timeout: 30_000 })
      expect(await heading.getText()).toContain('目录插件整壳')

      const isolation = await browser.execute(() => {
        let parentDomAccessible = true
        try {
          void window.parent.document.body
        }
        catch {
          parentDomAccessible = false
        }
        return {
          parentDomAccessible,
          tauriInternals: typeof (
            globalThis as typeof globalThis & { __TAURI_INTERNALS__?: unknown }
          ).__TAURI_INTERNALS__,
          bridge: typeof (
            window as typeof window & { OclivePluginBridge?: unknown }
          ).OclivePluginBridge,
        }
      })
      expect(isolation).toEqual({
        parentDomAccessible: false,
        tauriInternals: 'undefined',
        bridge: 'object',
      })

      const bootstrap = await browser.$('#boot')
      await browser.waitUntil(
        async () =>
          (await bootstrap.getText()).includes('com.oclive.example.minimal'),
        {
          timeout: 30_000,
          timeoutMsg: 'isolated shell broker never returned bootstrap',
        },
      )
    }
    finally {
      await browser.deleteSession()
    }
  })
})
