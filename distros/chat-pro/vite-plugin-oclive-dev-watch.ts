import type { Plugin } from 'vite'
import path from 'node:path'

const RELOAD_DEBOUNCE_MS = 800

/** Plugin assets outside Vite graph; host shared code uses alias + normal HMR. */
function shouldReloadForPluginFile(file: string): boolean {
  const norm = file.replace(/\\/g, '/')
  if (!norm.includes('/plugins/'))
    return false
  if (/\/models\//.test(norm))
    return false
  if (/\.(wav|mp3|onnx|pt|pth|bin|gguf|zip|db|sqlite|log)$/i.test(norm))
    return false
  return /\.(vue|mjs|cjs|js|ts|json|css|html)$/i.test(norm)
}

/**
 * Dev-only: full-reload when directory plugin assets change.
 * Do NOT watch `../shared/src` here — it is already in the Vite module graph via
 * `@oclive/shared` alias; extra full-reload races with optimizeDeps and can parse
 * `.vue` as empty on Windows.
 */
export function ocliveDevWatchPlugin(rootDir: string): Plugin {
  return {
    name: 'oclive-dev-watch',
    apply: 'serve',
    configureServer(server) {
      const pluginsRoot = path.resolve(rootDir, 'plugins')
      server.watcher.add(pluginsRoot)

      let reloadTimer: ReturnType<typeof setTimeout> | undefined
      server.watcher.on('change', (file) => {
        if (!shouldReloadForPluginFile(file))
          return
        if (reloadTimer)
          clearTimeout(reloadTimer)
        reloadTimer = setTimeout(() => {
          reloadTimer = undefined
          server.ws.send({ type: 'full-reload', path: '*' })
        }, RELOAD_DEBOUNCE_MS)
      })
    },
  }
}
