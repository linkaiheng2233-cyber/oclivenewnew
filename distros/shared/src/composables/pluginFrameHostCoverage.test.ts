import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const sharedSrc = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const iframeHosts = [
  'components/ChatPluginToolbarSlots.vue',
  'components/PluginChatHeaderSlots.vue',
  'components/PluginRoleDetailSlots.vue',
  'components/PluginSettingsPanelSlots.vue',
  'components/PluginSidebarSlots.vue',
  'components/PluginSlotEmbed.vue',
  'components/hotkey/HotkeyHost.vue',
]

describe('plugin iframe host coverage', () => {
  it.each(iframeHosts)('%s uses the isolated frame broker', (relativePath) => {
    const source = readFileSync(path.join(sharedSrc, relativePath), 'utf8')

    expect(source).toContain('<iframe')
    expect(source).toContain('sandbox="allow-scripts"')
    expect(source).toContain('bindPluginFrame')
    expect(source).toContain('onPluginFrameLoad')
    expect(source).toMatch(/@load="onPluginFrameLoad\([^"\n]+, \$event\)"/)
  })

  it('keeps mounted frames across role-state refreshes', () => {
    const source = readFileSync(
      path.join(sharedSrc, 'composables/useDirectoryPluginSlotEmbed.ts'),
      'utf8',
    )

    expect(source).toContain('slotSignature')
    expect(source).not.toContain('currentRoleId')
  })
})
