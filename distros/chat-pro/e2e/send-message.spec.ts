import { expect, test } from '@playwright/test'
import { gotoApp } from './helpers'

test.describe('send message (preview + invoke mock)', () => {
  test('sends a message and shows it in the chat list', async ({ page }) => {
    await gotoApp(page)
    await page.locator('#chat-user-message').fill('Hello from E2E')
    await page.getByRole('button', { name: /Send|发送/i }).click()
    await expect(page.getByText('Hello from E2E', { exact: true })).toBeVisible()
    await expect(page.getByText('Echo: Hello from E2E')).toBeVisible()
  })

  test('blocks empty message send', async ({ page }) => {
    await gotoApp(page)
    const send = page.getByRole('button', { name: /Send|发送/i })
    await expect(send).toBeDisabled()
    await page.locator('#chat-user-message').fill('   ')
    await expect(send).toBeDisabled()
  })
})
