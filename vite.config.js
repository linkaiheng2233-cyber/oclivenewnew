import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { visualizer } from "rollup-plugin-visualizer";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
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
    include: ["vue3-sfc-loader", "mitt"],
  },

  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) return;
          if (id.includes("@sentry")) return "vendor-sentry";
          if (id.includes("@tauri-apps")) return "vendor-tauri";
          if (id.includes("vue-virtual-scroller")) return "vendor-scroller";
          if (id.includes("pinia")) return "vendor-pinia";
          // vue3-sfc-loader 仅经动态 import 加载，不打入首屏 vendor
          if (id.includes("/vue/") || id.includes("@vue/")) return "vendor-vue";
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
