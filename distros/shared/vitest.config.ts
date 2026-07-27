import path from 'node:path'
import { fileURLToPath } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

const rootDir = path.dirname(fileURLToPath(import.meta.url))
const sharedSrc = path.join(rootDir, 'src')

export default defineConfig({
  root: rootDir,
  plugins: [vue()],
  resolve: {
    alias: {
      '@oclive/shared': sharedSrc,
    },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
})
