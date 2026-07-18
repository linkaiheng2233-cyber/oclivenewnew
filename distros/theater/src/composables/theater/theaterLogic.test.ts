import type { AppliedTweak } from './theaterLogic'
import { describe, expect, it } from 'vitest'
import {
  buildPatchPrompt,
  buildWorkingScript,
  cloneScriptLines,
  defaultInsertAnchor,
  insertForkLines,
  nextVisibleCount,
  parsePatchReply,
  playbackDone,
  resolveChipLeadCast,
  SCENE_GEN_TIMEOUT_MS,
  SceneGenTimeoutError,
  timeoutReject,
  validateSkeleton,
} from './theaterLogic'

const sampleBeat = {
  id: 'b1',
  cast: 'a' as const,
  name: '木木',
  text: '测试',
}

describe('theaterLogic', () => {
  it('insertForkLines splices after beat id', () => {
    const lines = [
      { ...sampleBeat, id: 'b1' },
      { ...sampleBeat, id: 'b2', cast: 'b' as const, name: '枫侵月' },
    ]
    const patch = [{ ...sampleBeat, id: 'p1', text: '补丁' }]
    const next = insertForkLines(lines, 'b1', patch)
    expect(next.map(l => l.id)).toEqual(['b1', 'p1', 'b2'])
  })

  it('playback advances and completes', () => {
    expect(nextVisibleCount(0, 3)).toBe(1)
    expect(playbackDone(3, 3)).toBe(true)
    expect(playbackDone(2, 3)).toBe(false)
  })

  it('parsePatchReply reads role lines', () => {
    const lines = parsePatchReply('木木：哼。\n(别过脸)', 'a', '木木', 'tea', 'x')
    expect(lines).toHaveLength(1)
    expect(lines[0]?.text).toBe('哼。')
    expect(lines[0]?.stageHint).toBe('别过脸')
  })

  it('buildPatchPrompt includes chip label and drama seed', () => {
    const prompt = buildPatchPrompt({
      chipLabel: '早饭咬到舌头',
      dramaSeed: '突发小意外打破平静',
      speakerName: '木木',
      partnerName: '枫侵月',
      contextLines: [sampleBeat],
      anchorLines: [sampleBeat],
    })
    expect(prompt).toContain('早饭咬到舌头')
    expect(prompt).toContain('突发小意外打破平静')
    expect(prompt).toContain('木木')
    expect(prompt).toContain('枫侵月')
  })

  it('validateSkeleton rejects empty beats', () => {
    expect(() => validateSkeleton({ scene: 'x', cast: { a: {}, b: {} }, beats: [] }))
      .toThrow()
  })

  it('cloneScriptLines copies array', () => {
    const cloned = cloneScriptLines([sampleBeat])
    expect(cloned).not.toBe([sampleBeat])
    expect(cloned[0]?.text).toBe('测试')
  })

  it('buildWorkingScript applies tweaks in order', () => {
    const base = [
      { ...sampleBeat, id: 'b1' },
      { ...sampleBeat, id: 'b2', cast: 'b' as const, name: '枫侵月' },
      { ...sampleBeat, id: 'b3', cast: 'b' as const, name: '枫侵月' },
    ]
    const tweakA: AppliedTweak = {
      kind: 'chip',
      chipId: 'tea',
      dramaSeed: 'test',
      insertAfterBeatId: 'b1',
      leadCast: 'a',
      anchorLines: [],
      patchLines: [{ ...sampleBeat, id: 'p1', text: '补丁A' }],
    }
    const tweakB: AppliedTweak = {
      kind: 'custom',
      dramaSeed: 'custom',
      insertAfterBeatId: 'b2',
      leadCast: 'b',
      anchorLines: [],
      patchLines: [{ ...sampleBeat, id: 'p2', cast: 'b', name: '枫侵月', text: '补丁B' }],
    }
    const result = buildWorkingScript(base, [tweakA, tweakB])
    expect(result.map(l => l.id)).toEqual(['b1', 'p1', 'b2', 'p2', 'b3'])
  })

  it('defaultInsertAnchor reads fork mid-point', () => {
    const sk = {
      scene: 'breakfast',
      cast: { a: { roleId: 'mumu', name: '木木' }, b: { roleId: 'x', name: '枫侵月' } },
      beats: [{ ...sampleBeat, id: 'b1' }, { ...sampleBeat, id: 'b6', cast: 'b' as const }],
      forks: { tea: [{ chipId: 'tea' as const, insertAfterBeatId: 'b6', patchLines: [] }] },
    }
    expect(defaultInsertAnchor(sk)).toBe('b6')
  })

  it('resolveChipLeadCast reads first fork patch speaker', () => {
    const sk = {
      scene: 'way_home',
      cast: { a: { roleId: 'mumu', name: '木木' }, b: { roleId: 'x', name: '枫侵月' } },
      beats: [{ ...sampleBeat, id: 'b1' }],
      forks: {
        sprainedAnkle: [{
          chipId: 'sprainedAnkle' as const,
          insertAfterBeatId: 'b5',
          patchLines: [{ ...sampleBeat, id: 'sa-1', cast: 'a' as const, text: '嘶——' }],
        }],
        wrongWay: [{
          chipId: 'wrongWay' as const,
          insertAfterBeatId: 'b5',
          patchLines: [{ ...sampleBeat, id: 'ww-1', cast: 'b' as const, name: '枫侵月', text: '等等' }],
        }],
      },
    }
    expect(resolveChipLeadCast(sk, 'sprainedAnkle')).toBe('a')
    expect(resolveChipLeadCast(sk, 'wrongWay')).toBe('b')
    expect(resolveChipLeadCast(sk, 'tea')).toBeNull()
  })

  it('sCENE_GEN_TIMEOUT_MS is 30 seconds', () => {
    expect(SCENE_GEN_TIMEOUT_MS).toBe(30_000)
  })

  it('timeoutReject rejects with SceneGenTimeoutError', async () => {
    await expect(
      Promise.race([new Promise(() => {}), timeoutReject(20)]),
    ).rejects.toBeInstanceOf(SceneGenTimeoutError)
  })
})
