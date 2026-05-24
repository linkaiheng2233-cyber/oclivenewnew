/// <reference types="vitest/config" />
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { visualizer } from "rollup-plugin-visualizer";

const host = process.env.TAURI_DEV_HOST;
const rootDir = path.dirname(fileURLToPath(import.meta.url));

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts", "src/__tests__/**/*.spec.ts"],
  },
  resolve:
    mode === "e2e"
      ? {
          alias: {
            "@tauri-apps/api/tauri": path.join(rootDir, "e2e-mock/tauri.ts"),
            "@tauri-apps/api/event": path.join(rootDir, "e2e-mock/event.ts"),
            "@tauri-apps/api/dialog": path.join(rootDir, "e2e-mock/dialog.ts"),
            "@tauri-apps/api/fs": path.join(rootDir, "e2e-mock/fs.ts"),
          },
        }
      : undefined,
  plugins: [
    vue(),
    mode === "analyze" &&
      visualizer({
        filename: "dist/stats.html",
        gzipSize: true,
        brotliSize: true,
        open: false,
      }),
  ].filter(Boolean),

  optimizeDeps: {
    include: [
      "vue3-sfc-loader",
      "mitt",
      "pinia-plugin-persistedstate",
      "pinia",
      "@vue-flow/core",
      "@vue-flow/background",
      "@vue-flow/controls",
      "@vue-flow/minimap",
      "@vue-flow/node-resizer",
    ],
  },

  esbuild:
    mode === "production"
      ? {
          target: "es2022",
          drop: ["console", "debugger"],
        }
      : {
          target: "es2022",
        },

  build: {
    target: "es2022",
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) return;
          // Order matters: more specific patterns before broader `@vue/` / `/vue/` matches.
          if (id.includes("@sentry")) return "vendor-sentry";
          if (id.includes("@tauri-apps")) return "vendor-tauri";
          if (id.includes("vue-i18n")) return "vendor-i18n";
          if (id.includes("pinia-plugin-persistedstate")) return "vendor-pinia-persist";
          if (id.includes("pinia")) return "vendor-pinia";
          // ArchitectureGraphFlow lazy-loads @vue-flow; keep separate from vendor-vue.
          if (id.includes("@vue-flow")) return "vendor-vue-flow";
          if (id.includes("vue3-sfc-loader")) return "vendor-sfc-loader";
          if (id.includes("acorn")) return "vendor-acorn";
          if (id.includes("idb-keyval")) return "vendor-idb";
          if (id.includes("/vue/") || id.includes("@vue/")) return "vendor-vue";
          return "vendor-misc";
        },
      },
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
