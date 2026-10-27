import { expect, test } from '@playwright/test'
import { gotoApp } from './helpers'

test('anime feature icons load at fixed sizes while navigation keeps its accessible labels', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 960 })
  await gotoApp(page)
  const icons = page.locator('.role-nav [data-feature-icon]')
  await expect(icons).toHaveCount(4)
  for (const icon of await icons.all()) {
    await expect(icon).toHaveAttribute('aria-hidden', 'true')
    await expect(icon).toHaveCSS('width', '40px')
    await expect(icon).toHaveCSS('height', '40px')
    await expect(icon.locator('img')).toHaveAttribute('alt', '')
    await expect.poll(() => icon.locator('img').evaluate((image: HTMLImageElement) => image.complete && image.naturalWidth > 0)).toBe(true)
    const source = await icon.locator('img').evaluate((image: HTMLImageElement) => image.currentSrc)
    const response = await page.request.get(source)
    expect(response.ok()).toBe(true)
    const png = await response.body()
    expect([96, 256]).toContain(png.readUInt32BE(16))
    expect(png.readUInt32BE(20)).toBe(png.readUInt32BE(16))
  }
  await page.locator('nav').getByRole('button', { name: 'Role library', exact: true }).click()
  await page.screenshot({ path: testInfo.outputPath('anime-library.png') })
  for (const [label, dialog, feature] of [
    ['App settings', '.sv-dialog', 'settings'],
    ['Help & recovery', '.workspace-help', 'help'],
  ]) {
    await page.locator('.nav-global').getByRole('button', { name: label, exact: true }).click()
    const surface = page.locator(dialog)
    await expect(surface.locator(`[data-feature-icon=${feature}]`)).toHaveCSS('width', '56px')
    await expect.poll(() => surface.locator(`[data-feature-icon=${feature}] img`).evaluate((image: HTMLImageElement) => image.complete && image.naturalWidth > 0)).toBe(true)
    await page.screenshot({ path: testInfo.outputPath(`anime-${feature}.png`) })
    await page.keyboard.press('Escape')
    await expect(surface).toHaveCount(0)
  }
})

test('missing decorative assets use a fixed-size fallback and leave role navigation usable', async ({ page }) => {
  await page.route('**/assets/library-*.png', route => route.abort())
  await gotoApp(page)
  const library = page.locator('.role-nav [data-feature-icon=library]')
  await expect(library.locator('.mumu-feature-icon__fallback')).toBeVisible()
  await expect(library).toHaveCSS('width', '40px')
  await page.locator('nav').getByRole('button', { name: 'Role library', exact: true }).click()
  await expect(page.locator('.role-card').first()).toBeVisible()
  await page.locator('.role-card .enter-role').first().click()
  await expect(page.locator('#chat-user-message')).toBeVisible()
})

test('icons stay readable without motion or overflow in dark, narrow and Win98 layouts', async ({ page }, testInfo) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })
  await page.addInitScript(() => localStorage.setItem('oclive-runtime-theme', 'dark'))
  await page.setViewportSize({ width: 430, height: 860 })
  await gotoApp(page)
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
  const opener = page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true })
  await opener.hover()
  await expect(opener.locator('.mumu-feature-icon')).toHaveCSS('transform', 'none')
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  await opener.click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
  await expect(page.locator('.sv-close')).toBeInViewport()
  await page.screenshot({ path: testInfo.outputPath('anime-dark-narrow.png') })
  await page.keyboard.press('Escape')
  await page.locator('.workspace-header h1').click()
  for (const key of ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'])
    await page.keyboard.press(key)
  await expect(page.locator('html')).toHaveAttribute('data-skin', 'win98')
  await opener.click()
  await expect(page.locator('.sv-head [data-feature-icon=settings]')).toHaveCSS('width', '28px')
  await expect(page.locator('.sv-close')).toBeInViewport()
  await page.screenshot({ path: testInfo.outputPath('anime-win98-narrow.png') })
})
