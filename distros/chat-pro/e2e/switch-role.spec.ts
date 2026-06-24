import { expect, test } from "@playwright/test";
import { gotoApp } from "./helpers";

test.describe("switch role (preview + invoke mock)", () => {
  test("updates role selector after role change", async ({ page }) => {
    await gotoApp(page);
    const roleSelect = page.locator(".selector-row--topbar select").first();
    await expect(roleSelect).toHaveValue("role-a", { timeout: 15_000 });
    await roleSelect.selectOption("role-b");
    await expect(roleSelect).toHaveValue("role-b", { timeout: 15_000 });
  });
});
