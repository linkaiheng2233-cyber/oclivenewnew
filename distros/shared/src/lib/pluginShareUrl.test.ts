import { describe, expect, it } from 'vitest'
import { classifyPluginShareUrl } from './pluginShareUrl'

describe('classifyPluginShareUrl', () => {
  it('detects plugins.json catalog URLs', () => {
    expect(
      classifyPluginShareUrl(
        'https://raw.githubusercontent.com/foo/bar/main/plugins.json',
      ),
    ).toBe('index')
  })

  it('detects git repository URLs', () => {
    expect(classifyPluginShareUrl('https://github.com/foo/oclive-plugin-demo')).toBe(
      'git',
    )
    expect(classifyPluginShareUrl('git@github.com:foo/bar.git')).toBe('git')
  })

  it('rejects empty input', () => {
    expect(classifyPluginShareUrl('   ')).toBe('invalid')
  })
})
