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
    const browser = await remote({
      hostname: driverHost,
      port: driverPort,
      path: "/",
      capabilities: {
        "tauri:options": {
          application: appPath,
          env: {
            OCLIVE_ROLES_DIR: rolesDir,
            OCLIVE_SKIP_STARTUP_HEALTH: "1",
            OCLIVE_SKIP_LLM_STARTUP_PROBE: "1",
          },
        },
      },
      connectionRetryCount: 3,
      connectionRetryTimeout: 120_000,
    });

    try {
      await browser.setTimeout({ implicit: 15_000 });
      const title = await browser.getTitle();
      expect(title).toMatch(/OCLIVE|oclivenewnew|沐沐/i);

      const leftPane = await browser.$(".left-pane");
      await expect(leftPane).toBeDisplayed();

      const roleSelect = await browser.$(".selector-row--topbar select");
      await expect(roleSelect).toBeDisplayed();
    } finally {
      await browser.deleteSession();
    }
  });
});
