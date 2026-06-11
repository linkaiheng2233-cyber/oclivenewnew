/// <reference types="vitest/config" />
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import { visualizer } from "rollup-plugin-visualizer";
import { resolveManualChunk } from "./src/build/manualChunks";

const host = process.env.TAURI_DEV_HOST;
const rootDir = path.dirname(fileURLToPath(import.meta.url));

/** Inject build-time shell kind into index.html early-boot script (default empty → tool). */
function injectShellEnv(): Plugin {
  return {
    name: "oclive-inject-shell-env",
    transformIndexHtml(html) {
      const shell = process.env.VITE_OCLIVE_SHELL ?? "";
      return html.replaceAll("__OCLIVE_SHELL__", shell);
    },
  };
}

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
    injectShellEnv(),
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
    // vue3-sfc-loader runtime is intentionally large; raise warning threshold to reduce noise.
    chunkSizeWarningLimit: 2000,
    rollupOptions: {
      output: {
        manualChunks(id) {
          return resolveManualChunk(id);
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
