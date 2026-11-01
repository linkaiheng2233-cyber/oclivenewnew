import type { Page } from '@playwright/test'
import { expect } from '@playwright/test'
import { resetE2eMockState } from '../../../e2e-mock/fixtures'

export async function gotoApp(page: Page): Promise<void> {
  resetE2eMockState()
  await page.addInitScript(() => {
    window.localStorage.setItem('oclive.locale.preference', 'en-US')
    window.localStorage.setItem('oclive_preset_picker_done', '1')
    // The browser-only E2E mock has no authenticated Tauri stream bridge.
    window.localStorage.setItem('oclive.chat.streamEnabled', 'false')
  })
  await page.goto('/')
  await expect(page.locator('#app')).toBeVisible({ timeout: 60_000 })
  const enterRole = page.locator('.role-card').first().getByRole('button', { name: /Enter workspace/i })
  await expect(enterRole.or(page.locator('#chat-user-message'))).toBeVisible({ timeout: 60_000 })
  if (await enterRole.isVisible())
    await enterRole.click()
  await expect(page.locator('#chat-user-message')).toBeVisible({ timeout: 60_000 })
}

export async function openMoreMenu(page: Page): Promise<void> {
  const toolMore = page.locator('.tool-more-menu').getByRole('button', { name: /More|更多/i })
  if (await toolMore.isVisible()) {
    await toolMore.click()
    return
  }
  const more = page.getByRole('button', { name: /More|更多/i })
  await more.first().click()
}

export async function unlockWin98(page: Page): Promise<void> {
  await page.locator('.workspace-header h1').click()
  for (const key of ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'])
    await page.keyboard.press(key)
  await expect(page.locator('html')).toHaveAttribute('data-skin', 'win98')
}

export async function openSimplePluginManager(page: Page): Promise<void> {
  const pluginsBtn = page
    .locator('nav.tool-activity-bar')
    .getByRole('button', { name: /^Plugins$|^插件$/ })
  if (await pluginsBtn.isVisible()) {
    await pluginsBtn.click()
    await expect(page.getByRole('button', { name: /Install plugin|安装插件/i })).toBeVisible()
    return
  }
  await openMoreMenu(page)
  await page.getByRole('button', { name: /Plugin manager|插件管理/i }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
}
