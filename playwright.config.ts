import { defineConfig, devices } from "@playwright/test";

const PREVIEW_PORT = process.env.OCLIVE_PREVIEW_PORT ?? "4180";
const previewOrigin = `http://127.0.0.1:${PREVIEW_PORT}`;
/** CI（Ubuntu）由 workflow 先拉起 `vite preview` 时再设为 `1`，跳过内置 webServer。 */
const externalPreview = process.env.PW_TEST_USE_EXTERNAL === "1";

/**
 * A1.1b 子项：静态构建 + `vite preview` 下的浏览器壳烟测（不经 Tauri 原生窗口）。
 * 需先有 **`npm run build:e2e`** 产物（`--mode e2e` 启用 `e2e-mock/` Tauri 桩）：`npm run build:e2e && npm run test:e2e:preview`
 * 端口默认 **4180**（可用 **`OCLIVE_PREVIEW_PORT`** 覆盖）。
 *
 * **Windows**：若内置 `webServer` 超时，可先 `npm run preview` 再设 **`PW_TEST_USE_EXTERNAL=1`** 后执行 **`npm run test:e2e:preview`**。CI 仅在 **Ubuntu** 跑本套（见 `.github/workflows/ci.yml`）。
 */
export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [["list"]],
  use: {
    baseURL: previewOrigin,
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: externalPreview
    ? undefined
    : {
        command: `node ./node_modules/vite/bin/vite.js preview --host 127.0.0.1 --port ${PREVIEW_PORT} --strictPort`,
        url: `${previewOrigin}/`,
        reuseExistingServer: !process.env.CI,
        timeout: 180_000,
      },
});
