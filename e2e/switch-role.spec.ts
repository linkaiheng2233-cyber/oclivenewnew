import { expect, test } from "@playwright/test";
import { gotoApp } from "./helpers";

test.describe("switch role (preview + invoke mock)", () => {
  test("updates character title after role change", async ({ page }) => {
    await gotoApp(page);
    await expect(page.getByRole("heading", { name: "Role Alpha" })).toBeVisible();
    const roleSelect = page.locator(".selector-row--topbar select").first();
    await roleSelect.selectOption("role-b");
    await expect(page.getByRole("heading", { name: "Role Beta" })).toBeVisible({
      timeout: 15_000,
    });
  });
});
