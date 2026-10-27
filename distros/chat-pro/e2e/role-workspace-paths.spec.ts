import type { Page } from '@playwright/test'
import { expect, test } from '@playwright/test'
import { gotoApp } from './helpers'

async function failNext(page: Page, command: string, once = true) {
  await page.evaluate(({ command, once }) => {
    localStorage.setItem(`oclive.e2e.fail.${command}`, JSON.stringify({ code: 'LLM_ERROR', message: 'service unavailable: test diagnostic; api_key=secret-test-value', once }))
  }, { command, once })
}

test('library-first startup can recover from a real rendered failure state', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('oclive.locale.preference', 'en-US')
    if (!sessionStorage.getItem('startup-fault-injected')) {
      localStorage.setItem('oclive.e2e.fail.list_roles', JSON.stringify({ code: 'IO_ERROR', message: 'roles directory inaccessible', once: true }))
      sessionStorage.setItem('startup-fault-injected', '1')
    }
  })
  await page.goto('/')
  await expect(page.locator('.workspace-empty [role=alert]')).toBeVisible()
  await page.getByRole('button', { name: /Reload/i }).click()
  await expect(page.locator('.role-card')).toHaveCount(2)
  await expect(page.locator('#chat-user-message')).toHaveCount(0)
})

test('an empty library remains usable with import and model controls', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('oclive.locale.preference', 'en-US')
    localStorage.setItem('oclive.e2e.emptyRoles', '1')
  })
  await page.goto('/')
  await expect(page.locator('.empty-role-icon')).toBeVisible()
  await expect(page.locator('.workspace-empty .pack-bar button')).toHaveCount(2)
  await expect(page.locator('.nav-global').getByRole('button', { name: /Models/i })).toBeEnabled()
})

test('failed send preserves input, shows summary above diagnostics, and recovers by manual retry', async ({ page }) => {
  await gotoApp(page)
  await failNext(page, 'send_message')
  const input = page.locator('#chat-user-message')
  await input.fill('keep these words')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(input).toHaveValue('keep these words')
  const notice = page.locator('.toast .error-notice')
  await expect(notice.locator('.error-notice-heading')).toContainText('model could not reply')
  await expect(notice.locator('pre')).not.toBeVisible()
  await notice.getByText('Technical details', { exact: true }).click()
  await expect(notice.locator('pre')).toContainText('LLM_ERROR')
  await expect(notice.locator('pre')).not.toContainText('secret-test-value')
  await page.waitForTimeout(6000)
  await expect(notice).toBeVisible()
  await notice.getByRole('button', { name: 'Dismiss notification' }).click()
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(page.locator('.workspace-messages')).toContainText('Echo: keep these words')
})

test('new conversations keep separate records across role changes and continuation', async ({ page }) => {
  await gotoApp(page)
  const input = page.locator('#chat-user-message')
  await input.fill('first transcript')
  await input.press('Enter')
  await expect(page.locator('.workspace-messages')).toContainText('Echo: first transcript')
  await page.locator('.conversation-header').getByRole('button', { name: 'New conversation' }).click()
  await expect(page.locator('.workspace-messages')).not.toContainText('first transcript')
  await input.fill('second transcript')
  await input.press('Enter')
  await expect(page.locator('.workspace-messages')).toContainText('Echo: second transcript')
  await page.locator('.nav-conversations').getByRole('button', { name: 'Echo: first transcript' }).click()
  await expect(page.locator('.workspace-messages')).toContainText('first transcript')
  await expect(page.locator('.workspace-messages')).not.toContainText('second transcript')
  await input.fill('continue the first')
  await input.press('Enter')
  await expect(page.locator('.workspace-messages')).toContainText('Echo: continue the first')
  await page.locator('.nav-role-title').getByRole('button', { name: 'Role Beta', exact: true }).click()
  await expect(page.locator('.workspace-messages')).not.toContainText('first transcript')
})

test('unavailable capability is explained and can be checked again', async ({ page }) => {
  await gotoApp(page)
  await failNext(page, 'get_chat_storage_capabilities', false)
  await page.locator('nav').getByRole('button', { name: 'Role library', exact: true }).click()
  await expect(page.locator('.workspace-content .error-notice')).toContainText('Separate conversations are unavailable')
  await page.evaluate(() => localStorage.removeItem('oclive.e2e.fail.get_chat_storage_capabilities'))
  await page.locator('.workspace-content .error-notice').getByRole('button', { name: 'Reload' }).click()
  await expect(page.locator('.workspace-content .error-notice')).toHaveCount(0)
})

test('completed replies remain successful when a later status refresh fails', async ({ page }) => {
  await gotoApp(page)
  await failNext(page, 'get_role_info')
  await page.locator('#chat-user-message').fill('already delivered')
  await page.locator('#chat-user-message').press('Enter')
  await expect(page.locator('.workspace-messages')).toContainText('Echo: already delivered')
  await expect(page.locator('.toast')).toContainText('Your reply arrived')
  await expect(page.locator('.toast .error-notice')).toHaveAttribute('role', 'status')
  await expect(page.locator('.toast [role=alert]')).toHaveCount(0)
  await expect(page.locator('#chat-user-message')).toHaveValue('')
})

test('narrow screens retain identity, records, models and a usable message composer', async ({ page }) => {
  await page.setViewportSize({ width: 430, height: 900 })
  await gotoApp(page)
  await expect(page.locator('#chat-user-message')).toBeVisible()
  await page.locator('.compact-role-controls').click()
  await expect(page.locator('.runtime-role--open')).toBeVisible()
  await expect(page.locator('.runtime-role--open select').first()).toBeVisible()
  await expect(page.locator('.conversation-header').getByRole('button', { name: /records/i })).toBeVisible()
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)
  expect(overflow).toBe(false)
})

test('long recent conversation titles do not widen the role navigation or squeeze chat', async ({ page }) => {
  await gotoApp(page)
  const originalWidth = await page.locator('.role-nav').evaluate(element => element.getBoundingClientRect().width)
  await page.locator('#chat-user-message').fill(`layout verification ${'W'.repeat(140)}`)
  await page.locator('#chat-user-message').press('Enter')
  await expect(page.locator('.workspace-messages')).toContainText('Echo: layout verification')
  const width = await page.locator('.role-nav').evaluate(element => element.getBoundingClientRect().width)
  expect(Math.abs(width - originalWidth)).toBeLessThanOrEqual(1)
  expect(await page.locator('#chat-user-message').evaluate(element => element.getBoundingClientRect().width)).toBeGreaterThan(260)
})
