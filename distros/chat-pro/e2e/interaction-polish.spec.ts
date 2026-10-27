import type { Locator, Page } from '@playwright/test'
import { expect, test } from '@playwright/test'
import { gotoApp } from './helpers'

async function expectCloseIcon(button: Locator, size = 34) {
  await expect(button).toBeVisible()
  await expect(button.locator('svg')).toHaveAttribute('aria-hidden', 'true')
  expect((await button.textContent() ?? '').trim()).toBe('')
  await expect(button).toHaveCSS('width', `${size}px`)
  await expect(button).toHaveCSS('height', `${size}px`)
}

async function finishEntry(surface: Locator) {
  await surface.evaluate(async (element) => {
    await Promise.all(element.getAnimations().map(animation => animation.finished))
  })
}

async function unlock(page: Page) {
  await page.locator('.workspace-header h1').click()
  for (const key of ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'])
    await page.keyboard.press(key)
  await expect(page.locator('html')).toHaveAttribute('data-skin', 'win98')
}

test('model, settings and recovery use the same close icon and return focus to their openers', async ({ page }, testInfo) => {
  await gotoApp(page)
  const modelOpener = page.locator('.nav-global').getByRole('button', { name: 'Models & connection', exact: true })
  await modelOpener.click()
  const model = page.locator('.mm-dialog')
  await expect(model).toBeFocused()
  await expectCloseIcon(model.locator('.mm-close'))
  await finishEntry(model)
  await page.screenshot({ path: testInfo.outputPath('model-dialog.png') })
  await model.locator('.mm-close').click()
  await expect(modelOpener).toBeFocused()

  const settingsOpener = page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true })
  await settingsOpener.click()
  await expectCloseIcon(page.locator('.sv-close'))
  await page.keyboard.press('Escape')
  await expect(settingsOpener).toBeFocused()

  const helpOpener = page.getByRole('button', { name: 'Help & recovery', exact: true })
  await helpOpener.click()
  const help = page.locator('.workspace-help')
  await expect(help).toBeFocused()
  await expectCloseIcon(help.locator('.workspace-help-close'))
  await help.getByRole('button', { name: /shortcuts/i }).click()
  const shortcuts = page.locator('.sh-dialog')
  await expect(shortcuts).toBeFocused()
  await expectCloseIcon(shortcuts.locator('.sh-close'))
  await page.keyboard.press('Escape')
  await expect(shortcuts).toHaveCount(0)
  await expect(help.getByRole('button', { name: /shortcuts/i })).toBeFocused()
  await help.locator('.workspace-help-close').click()
  await expect(helpOpener).toBeFocused()
})

test('Tab stays inside the model dialog and Escape preserves the current chat draft', async ({ page }) => {
  await gotoApp(page)
  const input = page.locator('#chat-user-message')
  await input.fill('keep this draft during settings')
  await page.locator('.nav-global').getByRole('button', { name: 'Models & connection', exact: true }).click()
  const model = page.locator('.mm-dialog')
  await expect(model.locator('.mm-tabs')).toBeVisible()
  await expect(model).toBeFocused()
  await page.keyboard.press('Tab')
  await expect(model.locator('.mm-close')).toBeFocused()
  await page.keyboard.press('Shift+Tab')
  expect(await model.evaluate(element => element.contains(document.activeElement))).toBe(true)
  await page.keyboard.press('Tab')
  await expect(model.locator('.mm-close')).toBeFocused()
  await page.keyboard.press('Escape')
  await expect(model).toHaveCount(0)
  await expect(input).toHaveValue('keep this draft during settings')
})

test('reduced motion and Win98 skip panel animation and keep consistent close controls', async ({ page }, testInfo) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })
  await gotoApp(page)
  await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
  await expect(page.locator('.sv-dialog')).toHaveCSS('animation-name', 'none')
  await expect(page.locator('.sv-close')).toHaveCSS('transition-duration', '0s')
  await page.keyboard.press('Escape')
  await page.emulateMedia({ reducedMotion: 'no-preference' })
  await unlock(page)
  expect(await page.evaluate(() => document.documentElement.clientWidth)).toBeGreaterThan(1000)
  await page.locator('.nav-global').getByRole('button', { name: 'Models & connection', exact: true }).click()
  await expect(page.locator('.mm-dialog')).toHaveCSS('animation-name', 'none')
  await expectCloseIcon(page.locator('.mm-close'), 24)
  await page.keyboard.press('Escape')
  await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
  await expectCloseIcon(page.locator('.sv-close'), 24)
  await page.screenshot({ path: testInfo.outputPath('win98-consistent-close.png') })
})

test('library actions stay grouped and card text stays readable on desktop and narrow screens', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 960 })
  await gotoApp(page)
  await page.locator('nav').getByRole('button', { name: 'Role library', exact: true }).click()
  await expect(page.locator('.library-actions').getByRole('button', { name: 'Reload', exact: true })).toBeVisible()
  await expect(page.locator('.library-actions .pack-bar button')).toHaveCount(2)
  await expect(page.locator('.role-description').first()).toHaveCSS('font-size', '13px')
  await expect(page.locator('.role-card-copy > small').first()).toHaveCSS('font-size', '12px')
  await page.screenshot({ path: testInfo.outputPath('library-desktop.png') })
  await page.setViewportSize({ width: 430, height: 860 })
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  await page.locator('.role-card .enter-role').first().scrollIntoViewIfNeeded()
  await expect(page.locator('.role-card .enter-role').first()).toBeInViewport()
  await page.screenshot({ path: testInfo.outputPath('library-narrow.png') })
})
