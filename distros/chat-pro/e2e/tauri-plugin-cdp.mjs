import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { chromium } from 'playwright'

async function main() {
  const [portText, outputPath, screenshotPath] = process.argv.slice(2)
  const port = Number(portText)
  if (!Number.isInteger(port) || port < 1 || port > 65535 || !outputPath)
    throw new Error('usage: probe-cdp.mjs <port> <output-json> <screenshot-path>')

  const facts = { port, pageCount: 0, frameCount: 0, error: null }
  let browser
  try {
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 20_000 })
    const context = browser.contexts()[0]
    assert.ok(context, 'CDP context is missing')
    const page = context.pages()[0]
    assert.ok(page, 'CDP page is missing')
    facts.consoleErrors = []
    facts.failedRequests = []
    facts.requests = []
    facts.responses = []
    page.on('request', request => facts.requests.push(request.url()))
    page.on('response', response => facts.responses.push({ url: response.url(), status: response.status() }))
    page.on('console', (message) => {
      if (message.type() === 'error')
        facts.consoleErrors.push(message.text())
    })
    page.on('requestfailed', (request) => {
      facts.failedRequests.push({ url: request.url(), failure: request.failure()?.errorText })
    })
    await page.reload({ waitUntil: 'domcontentloaded', timeout: 30_000 })
    facts.pageCount = context.pages().length
    facts.pageUrl = page.url()
    await page.locator('#oclive-directory-shell-frame').waitFor({ state: 'attached', timeout: 60_000 })
    const hostFrame = page.locator('#oclive-directory-shell-frame')
    facts.sandbox = await hostFrame.getAttribute('sandbox')
    facts.frameSrc = await hostFrame.getAttribute('src')
    facts.hostLeftPaneCount = await page.locator('.left-pane').count()
    let frame = null
    for (let i = 0; i < 60 && !frame; i++) {
      frame = page.frames().find(candidate => candidate.url().includes('/com.oclive.example.minimal/ui/index.html')) ?? null
      if (!frame)
        await page.waitForTimeout(500)
    }
    facts.frameUrls = page.frames().map(candidate => candidate.url())
    assert.ok(frame, 'full-shell iframe did not resolve')
    await frame.locator('h1').waitFor({ state: 'visible', timeout: 30_000 })
    facts.frameCount = page.frames().length
    facts.heading = await frame.locator('h1').textContent()
    facts.boundary = await frame.evaluate(() => {
      let parentDomAccessible = true
      try {
        void window.parent.document.body
      }
      catch {
        parentDomAccessible = false
      }
      return {
        parentDomAccessible,
        tauriInternals: typeof globalThis.__TAURI_INTERNALS__,
        bridge: typeof window.OclivePluginBridge,
        internalKeys: Object.keys(globalThis.__TAURI_INTERNALS__ ?? {}),
      }
    })
    await frame.waitForFunction(
      () => document.querySelector('#boot')?.textContent?.includes('com.oclive.example.minimal') === true,
      undefined,
      { timeout: 30_000 },
    )
    facts.bootstrapContainsOwnIdentity = (await frame.locator('#boot').textContent() ?? '')
      .includes('com.oclive.example.minimal')
    facts.bootstrapText = (await frame.locator('#boot').textContent() ?? '').slice(0, 3000)
    facts.directIpc = await frame.evaluate(async () => {
      const invoke = globalThis.__TAURI_INTERNALS__?.invoke
      if (typeof invoke !== 'function')
        return { available: false }
      try {
        const result = await Promise.race([
          invoke('get_directory_plugin_bootstrap', { roleId: null }),
          new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 5000)),
        ])
        return { available: true, succeeded: true, hasShellPluginId: !!result?.shellPluginId }
      }
      catch (error) { return { available: true, succeeded: false, error: String(error) } }
    })

    const otherPluginUrl = 'https://ocliveplugin.localhost/com.oclive.voice.asr/slots/toolbar.html'
    facts.otherPlugin = await frame.evaluate(async (url) => {
      const child = document.createElement('iframe')
      child.id = 'native-other-plugin-probe'
      child.setAttribute('sandbox', 'allow-scripts')
      child.src = url
      const loaded = await new Promise((resolve) => {
        const timer = setTimeout(resolve, 10_000, false)
        child.addEventListener('load', () => {
          clearTimeout(timer)
          resolve(true)
        }, { once: true })
        document.body.append(child)
      })
      let childDomAccessible = true
      try {
        childDomAccessible = child.contentDocument !== null
      }
      catch {
        childDomAccessible = false
      }
      return { loaded, childDomAccessible, src: child.src }
    }, otherPluginUrl)
    const otherFrame = page.frames().find(candidate => candidate.url() === otherPluginUrl)
    facts.otherPlugin.visibleContent = otherFrame
      ? await otherFrame.locator('button#record').count() === 1
      : false
    facts.assetFetch = await frame.evaluate(async () => {
      async function probe(url) {
        const controller = new AbortController()
        const timer = setTimeout(() => controller.abort(), 5_000)
        try {
          const response = await fetch(url, { signal: controller.signal })
          return { allowed: true, status: response.status, bytes: (await response.text()).length }
        }
        catch (error) { return { allowed: false, error: String(error) } }
        finally { clearTimeout(timer) }
      }
      return {
        own: await probe('https://ocliveplugin.localhost/com.oclive.example.minimal/manifest.json'),
        other: await probe('https://ocliveplugin.localhost/com.oclive.voice.asr/manifest.json'),
      }
    })

    assert.equal(facts.sandbox, 'allow-scripts')
    assert.equal(facts.hostLeftPaneCount, 0)
    assert.equal(facts.boundary.parentDomAccessible, false)
    assert.equal(facts.directIpc.succeeded, false)
    if (facts.directIpc.available)
      assert.match(facts.directIpc.error, /Origin header is not a valid URL|not allowed|forbidden|denied/i)
    assert.equal(facts.boundary.bridge, 'object')
    assert.equal(facts.bootstrapContainsOwnIdentity, true)
    assert.equal(facts.otherPlugin.loaded, true)
    assert.equal(facts.otherPlugin.visibleContent, true)
    assert.equal(facts.otherPlugin.childDomAccessible, false)
    facts.ok = true
    if (screenshotPath)
      await page.screenshot({ path: screenshotPath }).catch(() => {})
  }
  catch (error) {
    facts.ok = false
    facts.error = String(error?.stack ?? error)
    if (browser && screenshotPath) {
      await browser.contexts()[0]?.pages()[0]?.screenshot({ path: screenshotPath }).catch(() => {})
    }
    process.exitCode = 1
  }
  finally {
    writeFileSync(outputPath, `${JSON.stringify(facts, null, 2)}\n`, 'utf8')
    await browser?.close().catch(() => {})
  }
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
