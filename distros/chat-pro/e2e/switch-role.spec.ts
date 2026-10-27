import { expect, test } from '@playwright/test'
import { gotoApp } from './helpers'

test.describe('switch role (preview + invoke mock)', () => {
  test('updates role selector after role change', async ({ page }) => {
    await gotoApp(page)
    await expect(page.locator('.workspace-header h1')).toContainText('Role Alpha')
    await page.locator('.nav-role-title').getByRole('button', { name: 'Role Beta', exact: true }).click()
    await expect(page.locator('.workspace-header h1')).toContainText('Role Beta')
  })
})
