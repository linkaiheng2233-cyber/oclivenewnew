import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vitest/config";

// 路径别名与 `vite.config.js` 对齐：当前工程以相对路径为主，无 `@/` 别名。
export default defineConfig({
  plugins: [vue()],
  test: {
    globals: true,
    environment: "jsdom",
    passWithNoTests: true,
    setupFiles: ["./vitest.setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}", "src/**/__tests__/**/*.ts"],
    css: true,
    server: {
      deps: {
        inline: ["@vue", "@vue/compiler-sfc", "vue-i18n", "pinia"],
      },
    },
  },
});
