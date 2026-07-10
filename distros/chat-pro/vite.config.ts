import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, mergeConfig } from 'vite'
import { createBaseViteConfig } from '../../vite.base.config.ts'
import { ocliveDevWatchPlugin } from './vite-plugin-oclive-dev-watch.ts'

const rootDir = path.dirname(fileURLToPath(import.meta.url))
const sharedDir = path.resolve(rootDir, '../shared/src')

export default defineConfig(({ mode }) =>
  mergeConfig(createBaseViteConfig(mode), {
    root: rootDir,
    plugins: mode === 'production' ? [] : [ocliveDevWatchPlugin(rootDir)],
    resolve: {
      alias: {
        '@oclive/shared': sharedDir,
        '@oclive/theater': path.resolve(rootDir, '../theater/src'),
        '@': path.join(rootDir, 'src'),
      },
    },
    test: {
      environment: 'node',
      include: [
        'src/**/*.test.ts',
        'src/__tests__/**/*.spec.ts',
        '../theater/src/composables/theater/**/*.test.ts',
      ],
    },
    server: {
      watch: {
        ignored: ['**/distros/desktop-tauri/**'],
      },
    },
  }),
)
