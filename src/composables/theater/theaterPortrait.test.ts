import { describe, expect, it } from 'vitest'
import { buildCastRoster, inferLineEmotion, resolveCastPortraitState, rosterBySide } from './theaterPortrait'
import type { ScriptLine } from './theaterLogic'

describe('theaterPortrait', () => {
  it('inferLineEmotion prefers explicit tag', () => {
    const line: ScriptLine = {
      id: 'x',
      cast: 'a',
      name: '木木',
      text: '……',
      emotion: 'happy',
    }
    expect(inferLineEmotion(line)).toBe('happy')
  })

  it('inferLineEmotion reads shy from stage hint', () => {
    const line: ScriptLine = {
      id: 'x',
      cast: 'a',
      name: '木木',
      text: '谢谢',
      stageHint: '别过脸',
    }
    expect(inferLineEmotion(line)).toBe('shy')
  })

  it('inferLineEmotion reads angry from tongue hint', () => {
    const line: ScriptLine = {
      id: 'x',
      cast: 'a',
      name: '木木',
      text: '呜——！',
      stageHint: '捂嘴蹦起来',
    }
    expect(inferLineEmotion(line)).toBe('angry')
  })

  it('resolveCastPortraitState tracks last line per cast', () => {
    const lines: ScriptLine[] = [
      { id: '1', cast: 'b', name: '枫侵月', text: '早', stageHint: '笑' },
      { id: '2', cast: 'a', name: '木木', text: '哼', stageHint: '别过脸' },
    ]
    const state = resolveCastPortraitState(lines)
    expect(state.a.emotion).toBe('shy')
    expect(state.b.emotion).toBe('happy')
    expect(state.a.active).toBe(true)
    expect(state.b.active).toBe(false)
  })

  it('buildCastRoster defaults side and rosterBySide groups columns', () => {
    const roster = buildCastRoster({
      a: { roleId: 'mumu', name: '木木', side: 'left' },
      b: { roleId: '枫侵月', name: '枫侵月', side: 'right' },
    })
    expect(roster).toHaveLength(2)
    expect(roster[0]?.side).toBe('left')

    const portraitMap = resolveCastPortraitState([
      { id: '1', cast: 'a', name: '木木', text: '哼', stageHint: '别过脸' },
    ])
    const cols = rosterBySide(roster, portraitMap)
    expect(cols.left).toHaveLength(1)
    expect(cols.right).toHaveLength(1)
    expect(cols.left[0]?.emotion).toBe('shy')
    expect(cols.left[0]?.active).toBe(true)
  })
})
