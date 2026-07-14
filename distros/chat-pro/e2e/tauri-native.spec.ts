/**
 * A1.1c minimal native-window smoke (Linux CI + local with tauri-driver).
 * Requires: `tauri-driver` on PATH, debug binary at TAURI_E2E_APP_PATH, optional WebKitWebDriver on Linux.
 * Dev dependency `webdriverio` is used here for WebDriver protocol (not Playwright).
 */
import { test, expect } from "@playwright/test";
import { remote } from "webdriverio";

const appPath = process.env.TAURI_E2E_APP_PATH?.trim();
const driverHost = process.env.TAURI_DRIVER_HOST ?? "127.0.0.1";
const driverPort = Number(process.env.TAURI_DRIVER_PORT ?? "4444");
const rolesDir = process.env.OCLIVE_ROLES_DIR ?? process.cwd() + "/roles";

test.describe("Tauri native window (A1.1c smoke)", () => {
  test.skip(
    !appPath,
    "Set TAURI_E2E_APP_PATH to the debug oclivenewnew-tauri binary",
  );

  test("main window title and left sidebar pane", async () => {
    // Playwright global/config timeout alone has been observed as 30s in CI; pin again here.
    test.setTimeout(180_000);

    let browser;
    try {
      browser = await remote({
        hostname: driverHost,
        port: driverPort,
        path: "/",
        capabilities: {
          // WDIO 9+ injects BiDi `webSocketUrl`; Ubuntu WebKitWebDriver (pre-2.46) rejects/hangs.
          "wdio:enforceWebDriverClassic": true,
          "tauri:options": {
            application: appPath,
            env: {
              OCLIVE_ROLES_DIR: rolesDir,
              OCLIVE_SKIP_STARTUP_HEALTH: "1",
              OCLIVE_SKIP_LLM_STARTUP_PROBE: "1",
            },
          },
        },
        // Fail fast on session create — long connectionRetry only extended silent hangs.
        connectionRetryCount: 0,
        connectionRetryTimeout: 10_000,
      });
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      throw new Error(
        `tauri-driver session create failed (POST http://${driverHost}:${driverPort}/session): ${detail}`,
      );
    }

    try {
      await browser.setTimeout({ implicit: 15_000 });
      // FluentShell is defineAsyncComponent — wait for shell chrome, not only window title.
      await browser.waitUntil(
        async () => {
          const t = await browser.getTitle();
          return /OCLIVE|oclivenewnew|沐沐/i.test(t);
        },
        {
          timeout: 60_000,
          timeoutMsg: "window title never matched OCLIVE / oclivenewnew / 沐沐",
        },
      );
      const title = await browser.getTitle();
      expect(title).toMatch(/OCLIVE|oclivenewnew|沐沐/i);

      const leftPane = await browser.$(".left-pane");
      // Use WebdriverIO wait (Playwright expect has no toBeDisplayed for WDIO elements).
      await leftPane.waitForDisplayed({ timeout: 60_000 });
      expect(await leftPane.isDisplayed()).toBe(true);

      const roleSelect = await browser.$(".selector-row--topbar select");
      await roleSelect.waitForDisplayed({ timeout: 30_000 });
      expect(await roleSelect.isDisplayed()).toBe(true);
    } finally {
      await browser.deleteSession();
    }
  });
});
