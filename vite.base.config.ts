import type { Plugin, UserConfig } from 'vite'
/// <reference types="vitest/config" />
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { visualizer } from 'rollup-plugin-visualizer'
import { defineConfig } from 'vite'
import { resolveManualChunk } from './distros/shared/src/build/manualChunks'

const host = process.env.TAURI_DEV_HOST
const repoRoot = path.dirname(fileURLToPath(import.meta.url))

/** Inject build-time shell kind into index.html early-boot script (default empty → fluent). */
export function injectShellEnv(): Plugin {
  return {
    name: 'oclive-inject-shell-env',
    transformIndexHtml(html) {
      const shell = process.env.VITE_OCLIVE_SHELL ?? ''
      return html.replaceAll('__OCLIVE_SHELL__', shell)
    },
  }
}

export function createBaseViteConfig(mode: string): UserConfig {
  return {
    plugins: [
      injectShellEnv(),
      vue(),
      mode === 'analyze'
      && visualizer({
        filename: 'dist/stats.html',
        gzipSize: true,
        brotliSize: true,
        open: false,
      }),
    ].filter(Boolean),

    optimizeDeps: {
      include: [
        'vue3-sfc-loader',
        'mitt',
        'pinia-plugin-persistedstate',
        'pinia',
      ],
    },

    esbuild:
      mode === 'production'
        ? {
            target: 'es2022',
            drop: ['console', 'debugger'],
          }
        : {
            target: 'es2022',
          },

    build: {
      target: 'es2022',
      chunkSizeWarningLimit: 2000,
      rollupOptions: {
        output: {
          manualChunks(id) {
            return resolveManualChunk(id)
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
            protocol: 'ws',
            host,
            port: 1421,
          }
        : undefined,
    },

    resolve:
      mode === 'e2e'
        ? {
            alias: {
              '@tauri-apps/api/core': path.join(repoRoot, 'e2e-mock/tauri.ts'),
              '@tauri-apps/api/event': path.join(repoRoot, 'e2e-mock/event.ts'),
              '@tauri-apps/plugin-dialog': path.join(repoRoot, 'e2e-mock/dialog.ts'),
              '@tauri-apps/plugin-opener': path.join(repoRoot, 'e2e-mock/opener.ts'),
              '@tauri-apps/api/fs': path.join(repoRoot, 'e2e-mock/fs.ts'),
            },
          }
        : undefined,
  }
}

export default defineConfig(({ mode }) => createBaseViteConfig(mode))
