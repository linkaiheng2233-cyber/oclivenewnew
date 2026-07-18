import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

const rootDir = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  build: {
    emptyOutDir: true,
    outDir: path.join(rootDir, 'dist/plugin-bridge'),
    lib: {
      entry: path.join(rootDir, 'distros/chat-pro/src/plugin-bridge.js'),
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
