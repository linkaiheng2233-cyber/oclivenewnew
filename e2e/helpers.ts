import { expect, type Page } from "@playwright/test";
import { resetE2eMockState } from "../e2e-mock/fixtures";

export async function gotoApp(page: Page): Promise<void> {
  resetE2eMockState();
  await page.addInitScript(() => {
    window.localStorage.setItem("oclive.locale.preference", "en-US");
  });
  await page.goto("/");
  await expect(page.locator("#app")).toBeVisible({ timeout: 60_000 });
  await expect(page.locator("#chat-user-message")).toBeVisible({ timeout: 60_000 });
}

export async function openMoreMenu(page: Page): Promise<void> {
  const more = page.getByRole("button", { name: /More|更多/i });
  await more.click();
}

export async function openSimplePluginManager(page: Page): Promise<void> {
  await openMoreMenu(page);
  await page.getByRole("button", { name: /Plugin manager|插件管理/i }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
}
