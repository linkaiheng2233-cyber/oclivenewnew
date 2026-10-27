import { expect, test } from '@playwright/test'

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
