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
  {
    name: 'production directory plugin slots fail closed to iframe',
    ok: () => {
      const source = readFileSync(
        join(shared, 'composables/useDirectoryPluginSlotEmbed.ts'),
        'utf8',
      )
      return source.includes('!isUnsafeInlinePluginVueEnabled()')
        && source.includes('return true')
        && source.includes('return false')
    },
  },
  {
    name: 'production directory shell blocks same-process Vue',
    ok: () => {
      const source = readFileSync(
        join(shared, 'utils/directoryShellBootstrap.ts'),
        'utf8',
      )
      return source.includes('!isUnsafeInlinePluginVueEnabled()')
    },
  },
  {
    name: 'production directory shell uses opaque-origin parent-broker isolation',
    ok: () => {
      const source = readFileSync(
        join(shared, 'utils/directoryShellBootstrap.ts'),
        'utf8',
      )
      return source.includes("setAttribute('sandbox', 'allow-scripts')")
        && source.includes('createPluginFrameBridge(pluginBridgeInvoke')
        && source.includes('parseDirectoryShellIdentity(shellUrl, shellPid)')
        && !source.includes('window.location.replace(shellUrl)')
        && !existsSync(join(root, 'distros/desktop-tauri/capabilities/plugin-shell-remote.json'))
    },
  },
  {
    name: 'embedded plugin frames use opaque-origin script-only sandbox',
    ok: () => {
      const source = readFileSync(
        join(shared, 'components/PluginSlotEmbed.vue'),
        'utf8',
      )
      return source.includes('sandbox="allow-scripts"')
        && !source.includes('allow-same-origin')
    },
  },
  {
    name: 'embedded plugin bridge binds calls in the parent host',
    ok: () => {
      const component = readFileSync(
        join(shared, 'components/PluginSlotEmbed.vue'),
        'utf8',
      )
      const broker = readFileSync(
        join(shared, 'utils/pluginFrameBridge.ts'),
        'utf8',
      )
      const injectedBridge = readFileSync(
        join(root, 'kernel/crates/oclive_kernel_host/assets/plugin-bridge.iife.js'),
        'utf8',
      )
      return component.includes('frameBridge.register(value.contentWindow')
        && broker.includes("event.origin !== 'null'")
        && broker.includes('request.token !== registration.token')
        && broker.includes('registration.activated')
        && broker.includes('registration.seenRequestIds.has')
        && injectedBridge.includes('oclive-plugin-frame-bridge-v1')
        && injectedBridge.includes('kind==="bind"')
        && injectedBridge.includes('parent.postMessage')
    },
  },
  {
    name: 'release compiler stays development-only and tree-shakeable',
    ok: () => {
      const rootPackage = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
      const chatPackage = JSON.parse(readFileSync(join(root, 'distros/chat-pro/package.json'), 'utf8'))
      const sharedPackage = JSON.parse(readFileSync(join(root, 'distros/shared/package.json'), 'utf8'))
      const compiler = readFileSync(join(shared, 'utils/compilePluginVueSfc.ts'), 'utf8')
      return !rootPackage.dependencies?.['vue3-sfc-loader']
        && !chatPackage.dependencies?.['vue3-sfc-loader']
        && !sharedPackage.dependencies?.['vue3-sfc-loader']
        && rootPackage.devDependencies?.['vue3-sfc-loader']
        && compiler.includes('if (!import.meta.env.DEV)')
    },
  },
  {
    name: 'official Voice HTML fallbacks expose functional isolated controls',
    ok: () => {
      const slots = join(root, 'distros/chat-pro/plugins/com.oclive.voice.asr/slots')
      const toolbar = readFileSync(join(slots, 'voice-toolbar.js'), 'utf8')
      const settings = readFileSync(join(slots, 'voice-settings.js'), 'utf8')
      const component = readFileSync(join(shared, 'components/PluginSlotEmbed.vue'), 'utf8')
      return toolbar.includes("rpc('voice.transcribe'")
        && toolbar.includes('bridge.emit(submitEvent')
        && settings.includes("bridge.invoke('set_plugin_settings_config'")
        && settings.includes("rpc('voice.warm'")
        && component.includes("slot.pluginId === VOICE_ASR_PLUGIN_ID && slot.entry === 'slots/toolbar.html'")
    },
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
