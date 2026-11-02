import type { Page } from '@playwright/test'
import { expect, test } from '@playwright/test'
import { gotoApp, unlockWin98 } from './helpers'

async function releaseHeld(page: Page, stage: string): Promise<void> {
  await page.evaluate(async (stage) => {
    localStorage.removeItem(`oclive.e2e.hold${stage}`)
    window.dispatchEvent(new Event(`oclive:e2e:release-${stage.toLowerCase()}`))
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => resolve())
    })
  }, stage)
}

for (const skin of ['default', 'win98']) {
  test(`long unbroken and multiline messages keep the composer usable in ${skin}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width: skin === 'win98' ? 430 : 800, height: 600 })
    await gotoApp(page)
    if (skin === 'win98')
      await unlockWin98(page)
    const input = page.locator('#chat-user-message')
    await input.fill(`长消息边界检查。${'保留每一句话。\n'.repeat(160)}${'x'.repeat(2400)}`)
    await input.press('Enter')
    await expect(page.locator('.bubble.assistant .content')).toContainText('Echo: 长消息边界检查。')
    await expect(input).toBeInViewport({ ratio: 1 })
    await expect(input).toBeEnabled()
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true)
    const user = page.locator('.bubble.user .content').first()
    expect(await user.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true)
    await input.fill('下一条草稿仍然可以输入')
    await expect(input).toHaveValue('下一条草稿仍然可以输入')
    await page.screenshot({ path: testInfo.outputPath(`long-chat-${skin}.png`) })
  })
}

test('stopping a pending reply lets the next message complete while the older result stays out of the foreground', async ({ page }) => {
  await gotoApp(page)
  await page.evaluate(() => localStorage.setItem('oclive.e2e.holdSend', '1'))
  const input = page.locator('#chat-user-message')
  await input.fill('older stopped request')
  await input.press('Enter')
  await page.locator('.stop-button').click()
  await expect(page.locator('.stop-button')).toHaveCount(0)
  await page.evaluate(() => localStorage.removeItem('oclive.e2e.holdSend'))
  await input.fill('new foreground request')
  await input.press('Enter')
  await expect(page.locator('.workspace-messages')).toContainText('Echo: new foreground request')
  await releaseHeld(page, 'Send')
  await expect(page.locator('.workspace-messages')).not.toContainText('Echo: older stopped request')
  await expect(page.locator('.stop-button')).toHaveCount(0)
  await expect(input).toBeEnabled()
})

test('switching characters and opening settings preserve each unsent draft', async ({ page }) => {
  await gotoApp(page)
  const input = page.locator('#chat-user-message')
  await input.fill('draft belonging to Alpha')
  await page.locator('.nav-role-title').getByRole('button', { name: 'Role Beta', exact: true }).click()
  await expect(page.locator('.workspace-header h1')).toContainText('Role Beta')
  await input.fill('draft belonging to Beta')
  await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
  await page.keyboard.press('Escape')
  await expect(input).toHaveValue('draft belonging to Beta')
  await page.locator('.nav-role-title').getByRole('button', { name: 'Role Alpha', exact: true }).click()
  await expect(input).toHaveValue('draft belonging to Alpha')
  await page.locator('.nav-role-title').getByRole('button', { name: 'Role Beta', exact: true }).click()
  await expect(input).toHaveValue('draft belonging to Beta')
})

for (const { stage, command } of [
  { stage: 'Settings', command: 'get_llm_user_settings' },
  { stage: 'Global', command: 'get_global_ollama_model' },
]) {
  test(`a closed model window cannot publish a late ${stage} error over its replacement`, async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem('ui', JSON.stringify({ modelSetupDismissed: true })))
    await gotoApp(page)
    await expect(page.locator('.model-setup')).toHaveCount(0)
    await page.evaluate(stage => localStorage.setItem(`oclive.e2e.hold${stage}`, '1'), stage)
    const openModel = page.locator('.nav-global').getByRole('button', { name: 'Models & connection', exact: true })
    await openModel.click()
    const model = page.locator('.mm-dialog')
    await expect(model).toBeFocused()
    if (stage === 'Settings')
      await expect(model).toContainText('Loading')
    else
      await expect(model.getByRole('button', { name: 'Save & apply', exact: true })).toBeEnabled()
    await page.keyboard.press('Escape')
    await expect(model).toHaveCount(0)
    await page.evaluate(stage => localStorage.removeItem(`oclive.e2e.hold${stage}`), stage)
    await openModel.click()
    await expect(model.getByRole('button', { name: 'Save & apply', exact: true })).toBeEnabled()
    await page.evaluate(command => localStorage.setItem(`oclive.e2e.fail.${command}`, JSON.stringify({ code: 'IO_ERROR', message: 'obsolete closed-window failure', once: true })), command)
    await releaseHeld(page, stage)
    await expect(model.getByRole('button', { name: 'Save & apply', exact: true })).toBeEnabled()
    await expect(model.locator('.mm-load-error')).toHaveCount(0)
    await expect(page.locator('.toast .error-notice[role=alert]')).toHaveCount(0)
  })
}

for (const stage of ['Picker', 'Peek']) {
  test(`import actions stay busy during a slow ${stage} and recover after cancellation`, async ({ page }) => {
    await gotoApp(page)
    await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
    const settings = page.locator('.sv-dialog')
    await settings.locator('[data-settings-page=roles]').click()
    await page.evaluate(stage => localStorage.setItem(`oclive.e2e.hold${stage}`, '1'), stage)
    const actions = settings.locator('.pack-bar button')
    await actions.nth(1).click()
    await expect(actions.nth(1)).toBeDisabled()
    await expect(actions.nth(2)).toBeDisabled()
    await releaseHeld(page, stage)
    const conflict = page.getByRole('dialog').filter({ has: page.locator('#pack-conflict-title') })
    await conflict.getByRole('button', { name: 'Cancel', exact: true }).click()
    await expect(actions.nth(1)).toBeEnabled()
    await expect(actions.nth(2)).toBeEnabled()
  })
}

test('overwrite progress stays visible until the refreshed role library is available', async ({ page }) => {
  await gotoApp(page)
  await page.locator('.nav-global').getByRole('button', { name: 'App settings', exact: true }).click()
  const settings = page.locator('.sv-dialog')
  await settings.locator('[data-settings-page=roles]').click()
  await settings.locator('.pack-bar button').nth(1).click()
  const conflict = page.getByRole('dialog').filter({ has: page.locator('#pack-conflict-title') })
  await page.evaluate(() => localStorage.setItem('oclive.e2e.holdLibrary', '1'))
  await conflict.locator('.btn-danger').click()
  const progress = page.getByRole('dialog').filter({ has: page.locator('.bar-track') })
  await expect(progress).toBeVisible()
  await expect(conflict.locator('.btn-danger')).toBeDisabled()
  await releaseHeld(page, 'Library')
  await expect(progress).toHaveCount(0)
  await expect(conflict).toHaveCount(0)
  await expect(settings.locator('.pack-bar button').nth(1)).toBeEnabled()
})
