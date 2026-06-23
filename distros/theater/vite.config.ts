import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, mergeConfig } from 'vite'
import { createBaseViteConfig } from '../../vite.base.config.ts'

const rootDir = path.dirname(fileURLToPath(import.meta.url))
const sharedDir = path.resolve(rootDir, '../shared/src')

export default defineConfig(({ mode }) =>
  mergeConfig(createBaseViteConfig(mode), {
    root: rootDir,
    publicDir: path.join(rootDir, 'public'),
    resolve: {
      alias: {
        '@oclive/shared': sharedDir,
        '@': path.join(rootDir, 'src'),
      },
    },
    test: {
      environment: 'node',
      include: [
        'src/**/*.test.ts',
        'src/composables/theater/**/*.test.ts',
      ],
    },
    server: {
      watch: {
        ignored: ['**/distros/desktop-tauri/**'],
      },
    },
  }),
)
