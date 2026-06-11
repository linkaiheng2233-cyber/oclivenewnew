import type { TheaterSceneIndex, TheaterSceneMeta, TheaterSkeleton } from './types'

const SCENES_INDEX_URL = '/theater/scenes.json'

/** Canonical skeleton URL: `/theater/{sceneId}/skeleton.json`. */
export function theaterSkeletonUrl(sceneId: string): string {
  return `/theater/${encodeURIComponent(sceneId)}/skeleton.json`
}

export async function loadSceneIndex(): Promise<TheaterSceneIndex> {
  const res = await fetch(SCENES_INDEX_URL)
  if (!res.ok) {
    throw new Error(`scenes index ${res.status}`)
  }
  return await res.json() as TheaterSceneIndex
}

export function resolveSceneTitle(scene: TheaterSceneMeta, locale: 'zh' | 'en'): string {
  return locale === 'zh' ? scene.title_zh : scene.title_en
}

export async function loadTheaterSkeleton(sceneId: string): Promise<TheaterSkeleton> {
  const res = await fetch(theaterSkeletonUrl(sceneId))
  if (!res.ok) {
    throw new Error(`${res.status}`)
  }
  const skeleton = await res.json() as TheaterSkeleton
  if (skeleton.scene_id !== sceneId) {
    throw new Error(`scene_id mismatch: expected ${sceneId}, got ${skeleton.scene_id}`)
  }
  return skeleton
}

/** Parallel prefetch for index + default scene skeleton (T3-PERF-01). */
export function prefetchTheaterBootstrap(sceneId = 'breakfast'): void {
  void Promise.all([loadSceneIndex(), loadTheaterSkeleton(sceneId)])
}

export async function loadTheaterBootstrap(sceneId: string): Promise<{
  index: TheaterSceneIndex
  skeleton: TheaterSkeleton
}> {
  const [index, skeleton] = await Promise.all([
    loadSceneIndex(),
    loadTheaterSkeleton(sceneId),
  ])
  return { index, skeleton }
}
