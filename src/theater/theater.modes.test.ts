import type { TheaterOutline } from './types'
import { describe, expect, it } from 'vitest'
import skeleton from '../../public/theater/breakfast/skeleton.json'
import scenesIndex from '../../public/theater/scenes.json'
import { theaterSkeletonUrl } from './sceneRegistry'
import { nextSpeakerAfter, useTheaterDirector } from './useTheaterDirector'
import {
  compileOutlineToSkeleton,
  sessionToOutline,
  sessionToSkeleton,
  skeletonToOutline,
  validateOutline,
} from './useTheaterOutlineCompiler'

describe('theater scene registry', () => {
  it('scenes index lists breakfast with canonical skeleton path', () => {
    expect(scenesIndex.scenes.length).toBeGreaterThan(0)
    const breakfast = scenesIndex.scenes.find(s => s.scene_id === 'breakfast')
    expect(breakfast).toBeDefined()
    expect(breakfast!.skeleton_path).toBe('/theater/breakfast/skeleton.json')
    expect(theaterSkeletonUrl('breakfast')).toBe('/theater/breakfast/skeleton.json')
  })
})

describe('theater outline compiler (Mode 2)', () => {
  const sampleOutline: TheaterOutline = {
    schema_version: 1,
    scene_id: 'breakfast',
    title: '早饭',
    role_a: 'theater-breakfast-a',
    role_b: 'theater-breakfast-b',
    beats: [
      { id: 'beat_1', speaker: 'a', summary: '焦味吐司。' },
      { id: 'beat_2', speaker: 'b', summary: '牛奶还温的。' },
      { id: 'beat_3', speaker: 'user', summary: '我今天不想上学。' },
    ],
  }

  it('validates outline structure', () => {
    expect(validateOutline(sampleOutline)).toEqual([])
    expect(validateOutline({ ...sampleOutline, beats: [] }).length).toBeGreaterThan(0)
  })

  it('compiles outline to playable skeleton', () => {
    const compiled = compileOutlineToSkeleton(sampleOutline)
    expect(compiled.scene_id).toBe('breakfast')
    expect(compiled.beats.length).toBe(3)
    expect(compiled.beats[0].delay_ms).toBe(0)
    expect(compiled.beats[1].delay_ms).toBeGreaterThan(0)
    expect(compiled.beats[0].text).toContain('焦味')
  })

  it('round-trips skeleton to outline', () => {
    const outline = skeletonToOutline(skeleton)
    expect(outline.beats.length).toBe(skeleton.beats.length)
    const recompiled = compileOutlineToSkeleton(outline)
    expect(recompiled.beats.map(b => b.id)).toEqual(skeleton.beats.map(b => b.id))
  })
})

describe('theater director (Mode 3)', () => {
  it('next speaker alternates user → A → B → user', () => {
    expect(nextSpeakerAfter('user')).toBe('a')
    expect(nextSpeakerAfter('a')).toBe('b')
    expect(nextSpeakerAfter('b')).toBe('user')
  })

  it('simulates 6 user rounds with correct pending speakers', () => {
    const meta = () => ({
      scene_id: 'breakfast',
      title: '早饭',
      role_a: 'theater-breakfast-a',
      role_b: 'theater-breakfast-b',
    })
    const {
      submitUserLine,
      appendOcLine,
      pendingSpeaker,
      phase,
      roundCount,
    } = useTheaterDirector(meta)

    for (let round = 1; round <= 6; round++) {
      expect(submitUserLine(`用户第${round}句`)).toBe(true)
      expect(pendingSpeaker()).toBe('a')
      appendOcLine('a', `A-${round}`)
      expect(pendingSpeaker()).toBe('b')
      appendOcLine('b', `B-${round}`)
      if (round < 6) {
        expect(phase.value).toBe('waiting_user')
      }
    }
    expect(roundCount.value).toBe(6)
    expect(phase.value).toBe('ended')
  })

  it('exports session to outline and frozen skeleton', () => {
    const session = {
      scene_id: 'breakfast',
      title: '早饭',
      role_a: 'theater-breakfast-a',
      role_b: 'theater-breakfast-b',
      turns: [
        { id: 't1', speaker: 'user' as const, text: '早安' },
        { id: 't2', speaker: 'a' as const, text: '吐司又焦了' },
      ],
    }
    const outline = sessionToOutline(session)
    expect(outline.beats.length).toBe(2)
    const frozen = sessionToSkeleton(session)
    expect(frozen.beats.every(b => b.speaker === 'a' || b.speaker === 'b')).toBe(true)
  })
})
