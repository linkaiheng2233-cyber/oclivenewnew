import { defineConfig } from 'vite'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const rootDir = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  build: {
    emptyOutDir: false,
    outDir: path.join(rootDir, 'src-tauri/assets'),
    lib: {
      entry: path.join(rootDir, 'src/plugin-bridge.js'),
      name: 'OclivePluginBridgeBundle',
      formats: ['iife'],
      fileName: () => 'plugin-bridge.iife.js',
    },
    rollupOptions: {
      output: {
        extend: true,
      },
    },
  },
})
