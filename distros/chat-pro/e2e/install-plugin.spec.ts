import { expect, test } from "@playwright/test";
import { gotoApp, openSimplePluginManager } from "./helpers";

test.describe("install local plugin (preview + invoke mock)", () => {
  test("installs from zip and lists plugin in manager", async ({ page }) => {
    await gotoApp(page);
    await openSimplePluginManager(page);
    await page.getByRole("button", { name: /Install plugin|安装插件/i }).click();
    await expect(page.locator(".spm-title", { hasText: "e2e-local-plugin" })).toBeVisible({
      timeout: 15_000,
    });
  });
});
