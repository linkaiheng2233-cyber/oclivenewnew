import { expect, test } from "@playwright/test";

test.describe("vite preview shell (A1.1b web slice)", () => {
  test("root loads and #app mounts", async ({ page }) => {
    await page.goto("/");
    const app = page.locator("#app");
    await expect(app).toBeVisible({ timeout: 60_000 });
    await expect(app.locator("> *").first()).toBeVisible({ timeout: 60_000 });
  });

  test("document title matches shipped branding", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/OCLIVE|沐沐/);
  });
});
