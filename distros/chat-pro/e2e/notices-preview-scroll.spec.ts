import { readFileSync } from 'node:fs'
import { expect, test } from '@playwright/test'
import { gotoApp } from './helpers'

test('failure notices keep the draft and expose copyable redacted diagnostics in one frame', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 960 })
  await gotoApp(page)
  await page.evaluate(() => {
    localStorage.setItem('oclive.e2e.fail.send_message', JSON.stringify({ code: 'LLM_ERROR', message: `service unavailable api_key=private-alert-fixture\n${'stack detail\n'.repeat(60)}`, once: true }))
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async (text: string) => {
      (window as unknown as { copiedDiagnostic: string }).copiedDiagnostic = text
    } } })
  })
  await page.locator('#chat-user-message').fill('keep this draft')
  await page.locator('#chat-user-message').press('Enter')
  const notice = page.locator('.toast .error-notice')
  await expect(notice).toHaveAttribute('role', 'alert')
  await expect(notice.locator('.error-notice-label')).toHaveText('Action incomplete')
  await expect(notice.locator('.error-notice-heading p')).toContainText('model could not reply')
  await expect(page.locator('#chat-user-message')).toHaveValue('keep this draft')
  await expect(page.locator('.toast')).toHaveCSS('border-top-width', '0px')
  await expect(notice.locator('details')).not.toHaveAttribute('open')
  await notice.locator('summary').click()
  await notice.getByRole('button', { name: 'Copy details', exact: true }).click()
  await expect(notice.getByRole('status')).toContainText('Details copied')
  const text = await page.evaluate(() => (window as unknown as { copiedDiagnostic: string }).copiedDiagnostic)
  expect(text).toContain('LLM_ERROR')
  expect(text).not.toContain('private-alert-fixture')
  await expect(notice).not.toContainText('private-alert-fixture')
  expect(await notice.locator('pre').evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true)
  await notice.locator('pre').evaluate((element) => {
    element.scrollTop = 100
  })
  expect(await notice.locator('pre').evaluate(element => element.scrollTop)).toBeGreaterThan(0)
  await page.screenshot({ path: testInfo.outputPath('error-details.png') })
  await page.setViewportSize({ width: 430, height: 860 })
  await expect(notice.getByRole('button', { name: 'Dismiss notification' })).toBeInViewport()
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  await page.screenshot({ path: testInfo.outputPath('error-details-narrow.png') })
  await notice.getByRole('button', { name: 'Open Help & recovery', exact: true }).click()
  await expect(page.locator('.toast')).toHaveCount(0)
  await expect(page.locator('.workspace-help')).toBeFocused()
  await page.locator('.workspace-help').getByRole('tab', { name: /Recent errors/ }).click()
  await expect(page.locator('.support-history')).toContainText('model could not reply')
  await page.keyboard.press('Escape')
  await expect(page.locator('.nav-global').getByRole('button', { name: 'Help & recovery', exact: true })).toBeFocused()
  await expect(page.locator('#chat-user-message')).toHaveValue('keep this draft')
})

test('role cards crop actual pack art while runtime keeps a complete portrait', async ({ page }, testInfo) => {
  const portrait = `data:image/png;base64,${readFileSync(new URL('../roles/mumu/assets/images/normal-v2.png', import.meta.url)).toString('base64')}`
  const tall = `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="400" height="1000"><rect width="400" height="1000" fill="#c4d9d4"/><circle cx="200" cy="180" r="130" fill="#f6bd92"/><path d="M80 420h240v500H80z" fill="#7596b6"/></svg>')}`
  await page.addInitScript(({ portrait, tall }) => {
    localStorage.setItem('oclive.e2e.portrait.role-a', portrait)
    localStorage.setItem('oclive.e2e.portrait.role-b', tall)
  }, { portrait, tall })
  await page.setViewportSize({ width: 1440, height: 960 })
  await gotoApp(page)
  await page.locator('nav').getByRole('button', { name: 'Role library', exact: true }).click()
  const previews = page.locator('.card-portrait .hero--preview img')
  await expect(previews).toHaveCount(2)
  for (const preview of await previews.all()) {
    await expect.poll(() => preview.evaluate((image: HTMLImageElement) => image.complete && image.naturalWidth > 0)).toBe(true)
    await expect(preview).toHaveCSS('object-fit', 'cover')
    await expect(preview).toHaveCSS('object-position', '50% 18%')
  }
  await page.screenshot({ path: testInfo.outputPath('role-crops-desktop.png') })
  await page.setViewportSize({ width: 430, height: 860 })
  await expect(page.locator('.card-portrait').first()).toHaveCSS('height', '190px')
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  await page.screenshot({ path: testInfo.outputPath('role-crops-narrow.png') })
  await page.locator('.role-card .enter-role').first().click()
  await page.locator('.compact-role-controls').click()
  const runtime = page.locator('.runtime-role img.avatar')
  await expect(runtime).toHaveCSS('object-fit', 'contain')
  await expect(page.locator('.runtime-role .hero--preview')).toHaveCount(0)
})

test('scrollbars share dimensions, remain scrollable, and restore classic Win98 chrome', async ({ page }, testInfo) => {
  await page.addInitScript(() => localStorage.setItem('oclive-runtime-theme', 'dark'))
  await page.setViewportSize({ width: 900, height: 620 })
  await gotoApp(page)
  await page.locator('nav').getByRole('button', { name: 'Role library', exact: true }).click()
  const library = page.locator('.library-page')
  await expect.poll(() => library.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true)
  const width = await library.evaluate(element => getComputedStyle(element, '::-webkit-scrollbar').width)
  expect(width).toBe('12px')
  await library.hover()
  await page.mouse.wheel(0, 450)
  await expect.poll(() => library.evaluate(element => element.scrollTop)).toBeGreaterThan(0)
  await library.evaluate((element) => {
    element.scrollTop = 0
  })
  await library.focus()
  await page.keyboard.press('PageDown')
  await expect.poll(() => library.evaluate(element => element.scrollTop)).toBeGreaterThan(0)
  await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
  const settings = page.locator('.sv-body')
  expect(await settings.evaluate(element => getComputedStyle(element, '::-webkit-scrollbar').width)).toBe(width)
  await page.screenshot({ path: testInfo.outputPath('scrollbars-dark.png') })
  await page.keyboard.press('Escape')
  await page.locator('.workspace-header h1').click()
  for (const key of ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'])
    await page.keyboard.press(key)
  await expect(page.locator('html')).toHaveAttribute('data-skin', 'win98')
  expect(await library.evaluate(element => getComputedStyle(element, '::-webkit-scrollbar').width)).toBe('16px')
  await page.screenshot({ path: testInfo.outputPath('scrollbars-win98.png') })
})
