import { expect, test } from '@playwright/test'
import { gotoApp, openSimplePluginManager } from './helpers'

const legacyTool = process.env.OCLIVE_E2E_SHELL === 'tool'

test.describe('plugin entry policy (preview + invoke mock)', () => {
  test(legacyTool ? 'Tool shell installs from zip and lists the plugin' : 'standard role workspace keeps deferred plugin tools hidden', async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem('ui', JSON.stringify({ advancedFeaturesEnabled: true, sceneId: 'home' })))
    await gotoApp(page)
    if (!legacyTool) {
      await expect(page.locator('.role-app')).toBeVisible()
      await expect(page.getByRole('button', { name: /^Plugins$|^Plugin manager$/ })).toHaveCount(0)
      await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
      await expect(page.locator('[data-settings-page=plugins]')).toHaveCount(0)
      await expect(page.locator('[data-settings-page=voice]')).toHaveCount(0)
      return
    }
    await openSimplePluginManager(page)
    await page.getByRole('button', { name: /Install plugin|安装插件/i }).click()
    await expect(page.locator('.spm-title', { hasText: 'e2e-local-plugin' })).toBeVisible({
      timeout: 15_000,
    })
  })
})
