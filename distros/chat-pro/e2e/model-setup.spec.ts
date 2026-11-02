import { expect, test } from '@playwright/test'
import { gotoApp, unlockWin98 } from './helpers'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('oclive.locale.preference', 'en-US')
  })
})

test('first-run cloud choice opens the right settings after choosing a character', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 })
  await page.goto('/')
  await expect(page.locator('.model-setup')).toBeVisible()
  await expect(page.locator('#chat-user-message')).toHaveCount(0)
  await page.locator('.model-setup').getByRole('button', { name: /Use a cloud API/ }).click()
  await expect(page.locator('.model-setup [role=status]')).toContainText('Open a character')
  await page.locator('.role-card').first().getByRole('button', { name: /Enter workspace/ }).click()
  const panel = page.getByRole('dialog', { name: /model/i })
  await expect(panel).toBeVisible()
  await expect(panel.getByRole('tab', { name: /cloud/i })).toHaveAttribute('aria-selected', 'true')
  await panel.locator('.mm-close').click()
  await expect(page.locator('#chat-user-message')).toBeVisible()
})

test('deferring keeps role selection available and models can be reopened later', async ({ page }) => {
  await page.goto('/')
  await page.locator('.model-setup').getByRole('button', { name: /Use a local model/ }).click()
  await page.locator('.model-setup').getByRole('button', { name: 'Set up later' }).click()
  await page.locator('.role-card').first().getByRole('button', { name: /Enter workspace/ }).click()
  await expect(page.locator('#chat-user-message')).toBeVisible()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await page.reload()
  await expect(page.locator('.model-setup')).toHaveCount(0)
  await page.locator('.role-card').first().getByRole('button', { name: /Enter workspace/ }).click()
  await page.locator('.nav-global').getByRole('button', { name: /Models/i }).click()
  await expect(page.getByRole('dialog', { name: /model/i })).toBeVisible()
})

test('cloud connection testing follows the saved configuration and explains unsaved edits', async ({ page }) => {
  await gotoApp(page)
  await page.locator('.nav-global').getByRole('button', { name: /Models/i }).click()
  const panel = page.getByRole('dialog', { name: /model/i })
  await panel.getByRole('tab', { name: /cloud/i }).click()
  const url = panel.locator('input[type="url"]').last()
  const model = panel.locator('input[list="mm-cloud-model-list"]')
  const probe = panel.locator('.mm-footer button').nth(1)
  await url.fill('https://saved.test/v1')
  await model.fill('mock-cloud')
  await panel.locator('input[type="password"]').fill('synthetic-key')
  await expect(probe).toBeDisabled()
  await expect(panel.locator('#mm-probe-save-hint')).toBeVisible()
  await panel.locator('.mm-footer button').first().click()
  await expect(probe).toBeEnabled()
  await probe.click()
  await expect(page.locator('.toast')).toContainText('Cloud API connection OK')
  await url.fill('https://unsaved.test/v1')
  await expect(probe).toBeDisabled()
  await expect(panel.locator('#mm-probe-save-hint')).toContainText('Save settings first')
  await url.fill('https://saved.test/v1')
  await expect(probe).toBeEnabled()
})

for (const { width, skin } of [
  { width: 800, skin: 'default' },
  { width: 430, skin: 'default' },
  { width: 800, skin: 'win98' },
  { width: 430, skin: 'win98' },
]) {
  test(`model save actions stay in view in a short ${width}px ${skin} window`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 600 })
    await gotoApp(page)
    if (skin === 'win98')
      await unlockWin98(page)
    await page.locator('.nav-global').getByRole('button', { name: 'Models & connection', exact: true }).click()
    const model = page.locator('.mm-dialog')
    const footer = model.locator('.mm-footer')
    await expect(footer).toBeInViewport({ ratio: 1 })
    await expect(footer.getByRole('button', { name: 'Save & apply', exact: true })).toBeInViewport({ ratio: 1 })
    await model.locator('.mm-root').evaluate(element => element.scrollTop = element.scrollHeight)
    await expect(footer).toBeInViewport({ ratio: 1 })
    await page.screenshot({ path: testInfo.outputPath(`model-actions-${width}-${skin}.png`) })
    await page.keyboard.press('Escape')
    await expect(page.locator('#chat-user-message')).toBeVisible()
  })
}
