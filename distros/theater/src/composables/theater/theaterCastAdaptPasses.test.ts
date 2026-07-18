import type { ScriptLine } from './theaterLogic'
import { describe, expect, it } from 'vitest'
import { pickCastRewritePreviewLine } from './theaterCastAdaptPasses'

describe('theaterCastAdaptPasses', () => {
  it('pickCastRewritePreviewLine returns first beat snippet', () => {
    const beats: ScriptLine[] = [
      { id: 'b1', cast: 'b', name: '诗梦', text: '……烦死了，自己不会热吗。' },
    ]
    expect(pickCastRewritePreviewLine(beats)).toContain('诗梦')
  })
})
