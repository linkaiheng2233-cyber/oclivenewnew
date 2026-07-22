import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { Script } from 'node:vm'
import { describe, expect, it } from 'vitest'

const pluginsRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../../plugins',
)
const fallbackEntries = [
  ['com.oclive.mumu.quick-actions', 'slots/toolbar.html'],
  ['com.oclive.mumu.chat-header-status', 'slots/header.html'],
  ['com.oclive.mumu.sidebar-glance', 'slots/sidebar.html'],
  ['com.oclive.mumu.role-detail-card', 'slots/detail.html'],
  ['com.oclive.mumu.settings-panel', 'slots/settings.html'],
] as const

describe('mumu isolated HTML slots', () => {
  it.each(fallbackEntries)('%s ships a functional %s entry', (pluginId, entry) => {
    const html = readFileSync(path.join(pluginsRoot, pluginId, entry), 'utf8')

    expect(html).toContain('window.OclivePluginBridge')
    expect(html).toMatch(/bridge\.invoke\('list_roles'/)
    expect(html).toMatch(/bridge\.invoke\('get_role_info'/)
    expect(html).toContain('bridge.listen(event')
    expect(html).toContain('payload?.roleId')
    expect(html).toMatch(/let (?:refresh|load)Generation = 0/)
    expect(html).toMatch(/generation !== (?:refresh|load)Generation/)
    expect(html).not.toContain('iframe 回退')
    expect(html).not.toContain('iframe fallback')
    const script = html.match(/<script>([\s\S]*?)<\/script>/i)?.[1]
    expect(script).toBeTruthy()
    expect(() => new Script(script)).not.toThrow()
  })
})
