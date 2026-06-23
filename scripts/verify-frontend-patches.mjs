/**
 * 自检：关键前端改动是否在本仓库落盘。用法：npm run verify:ui
 * 跳过：set OCLIVE_SKIP_VERIFY=1
 */
import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

if (process.env.OCLIVE_SKIP_VERIFY === '1') {
  console.log('[verify:ui] skipped (OCLIVE_SKIP_VERIFY=1)')
  process.exit(0)
}

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const chatPro = join(root, 'distros', 'chat-pro', 'src')
const shared = join(root, 'distros', 'shared', 'src')

const checks = [
  {
    name: 'SimplePluginManagerPanel.vue exists (plugin entry, Ctrl+Shift+F)',
    ok: () => existsSync(join(chatPro, 'views/SimplePluginManagerPanel.vue')),
  },
  {
    name: 'ModelManagerPanel.vue exists (model entry, Ctrl+Shift+M)',
    ok: () => existsSync(join(chatPro, 'views/ModelManagerPanel.vue')),
  },
  {
    name: 'FluentShell mounts SimplePluginManagerPanel',
    ok: () =>
      readFileSync(join(chatPro, 'shells/fluent/FluentShell.vue'), 'utf8').includes(
        'SimplePluginManagerPanel',
      ),
  },
  {
    name: 'usePluginManagerWindow composable exists',
    ok: () => existsSync(join(shared, 'composables/usePluginManagerWindow.ts')),
  },
  {
    name: 'useGlobalHotkeys wires model manager toggle',
    ok: () =>
      readFileSync(join(shared, 'composables/useGlobalHotkeys.ts'), 'utf8').includes(
        'modelManagerOpen',
      ),
  },
]

let failed = false
for (const c of checks) {
  let pass = false
  let err = ''
  try {
    pass = c.ok()
  }
  catch (e) {
    err = ` (${e.code ?? e.message})`
  }
  console[pass ? 'log' : 'error'](`${pass ? 'OK' : 'FAIL'}  ${c.name}${err}`)
  if (!pass)
    failed = true
}

if (failed) {
  console.error('\n[verify:ui] 未通过。\n')
  process.exit(1)
}
console.log('\n[verify:ui] All checks passed.')
process.exit(0)
