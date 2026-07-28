import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../../../..',
)
const voicePlugin = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr',
)

function source(relativePath: string): string {
  return readFileSync(path.join(voicePlugin, relativePath), 'utf8')
}

describe('voice role settings parity', () => {
  it('declares the per-role config and read-only role catalog bridge', () => {
    const manifest = JSON.parse(source('manifest.json')) as {
      ui_schema?: { fields?: Array<{ key?: string }> }
      ui_slots?: Array<{
        slot?: string
        bridge?: { invoke?: string[] }
      }>
    }
    const settings = manifest.ui_slots?.find(row => row.slot === 'settings.panel')

    expect(manifest.ui_schema?.fields?.some(row => row.key === 'role_tts_enabled')).toBe(true)
    expect(settings?.bridge?.invoke).toContain('list_roles')
    expect(settings?.bridge?.invoke).toContain('get_role_pack_path')
  })

  it.each([
    ['native Vue', 'slots/VoiceSettings.vue'],
    ['iframe fallback', 'slots/voice-settings.js'],
  ])('%s keeps role discovery, profile detection, and persistence together', (_label, file) => {
    const text = source(file)
    const userFacingText = file.endsWith('.js')
      ? `${text}\n${source('slots/settings.html')}`
      : text

    expect(text).toContain('list_roles')
    expect(text).toContain('voice.read_role_profile')
    expect(text).toContain('role_tts_enabled')
    expect(userFacingText).toContain('voice_profile.json')
  })
})
