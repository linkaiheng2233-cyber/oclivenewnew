import type { TheaterCastConfig } from './theaterCastConfig'
import type { TheaterScenePreset } from './theaterSceneCatalog'
import {
  generateTheaterScene,
  type TheaterSceneRequest,
  type TheaterScriptLine,
} from '@oclive/shared/api/theater'
import { normalizePairRelationId, resolvePairRelationHint } from './theaterPairRelation'
import { SCENE_GEN_TIMEOUT_MS, SceneGenTimeoutError, timeoutReject } from './theaterLogic'

export const OUTLINE_STORAGE_KEY = 'oclive.theater.outline.v1'

const FALLBACK_OUTLINE_BEATS: TheaterScriptLine[] = [
  { id: 'o1', cast: 'b', name: '', text: '……你写的大纲里，这一刻该谁开口？' },
  { id: 'o2', cast: 'a', name: '', text: '（大纲模式罐头）我们按你的骨架演，但模型还没接上。' },
]

export function getStoredOutline(): string {
  try {
    return localStorage.getItem(OUTLINE_STORAGE_KEY)?.trim() ?? ''
  }
  catch {
    return ''
  }
}

export function setStoredOutline(text: string): void {
  try {
    localStorage.setItem(OUTLINE_STORAGE_KEY, text)
  }
  catch {
    /* ignore quota */
  }
}

function sceneContextFields(preset: TheaterScenePreset) {
  return {
    theater_scene: preset.id,
    scene_brief: preset.sceneBrief,
    scene_setting_hint: preset.sceneSettingHint,
  }
}

function pairRelationFields(cast: TheaterCastConfig) {
  const id = normalizePairRelationId(cast.pairRelationId)
  return {
    pair_relation_id: id,
    pair_relation_hint: resolvePairRelationHint(id),
  }
}

export async function requestOutlineScene(
  outline: string,
  cast: TheaterCastConfig,
  preset: TheaterScenePreset,
): Promise<{ beats: TheaterScriptLine[]; source: string; failureReason?: string }> {
  const trimmed = outline.trim()
  if (!trimmed)
    throw new Error('outline_empty')

  const req: TheaterSceneRequest = {
    cast_a: { role_id: cast.a.roleId, name: cast.a.name },
    cast_b: { role_id: cast.b.roleId, name: cast.b.name },
    scene_id: preset.runtimeSceneId,
    base_beats: [],
    applied_tweaks: [],
    fallback_beats: FALLBACK_OUTLINE_BEATS.map(b => ({
      ...b,
      name: b.cast === 'a' ? cast.a.name : cast.b.name,
    })),
    mode: 'outline_rewrite',
    script_outline: trimmed,
    max_beats: 12,
    ...sceneContextFields(preset),
    ...pairRelationFields(cast),
  }

  const resp = await Promise.race([
    generateTheaterScene(req),
    timeoutReject(SCENE_GEN_TIMEOUT_MS),
  ])

  if (resp.source === 'fallback') {
    return {
      beats: resp.beats,
      source: resp.source,
      failureReason: resp.failure_reason?.trim() || 'outline_fallback',
    }
  }
  return { beats: resp.beats, source: resp.source }
}

export { SceneGenTimeoutError }
