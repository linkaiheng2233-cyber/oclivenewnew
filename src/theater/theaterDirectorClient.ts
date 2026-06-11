/**
 * Optional Theater director directory-plugin RPC (Mode 3 / advanced).
 * Not a six-slot backend — see handoff/RFC_THEATER_DIRECTOR_PLUGIN.md
 */
import { directoryPluginInvoke, getDirectoryPluginCatalog } from '../api/plugin'
import type { TheaterBeat, TheaterSpeaker } from './types'

export const THEATER_DIRECTOR_PLUGIN_ID = 'com.oclive.theater.director'

export interface DirectorPingResult {
  ok: boolean
  plugin: string
  version: string
}

export interface DirectorInjectBeatResult {
  beat: TheaterBeat
}

export interface DirectorValidateRulesResult {
  valid: boolean
  violations: string[]
}

export interface DirectorSwitchSceneResult {
  ok: boolean
  scene_id?: string
  skeleton_path?: string
  error?: string
}

let directorAvailableCache: boolean | null = null

export function resetDirectorAvailabilityCache(): void {
  directorAvailableCache = null
}

/** Best-effort catalog probe; cached for the session. */
export async function isDirectorPluginAvailable(): Promise<boolean> {
  if (directorAvailableCache !== null) {
    return directorAvailableCache
  }
  try {
    const catalog = await getDirectoryPluginCatalog()
    directorAvailableCache = catalog.some(e => e.id === THEATER_DIRECTOR_PLUGIN_ID)
  }
  catch {
    directorAvailableCache = false
  }
  return directorAvailableCache
}

export async function pingDirector(): Promise<DirectorPingResult | null> {
  if (!(await isDirectorPluginAvailable())) {
    return null
  }
  try {
    const raw = await directoryPluginInvoke(THEATER_DIRECTOR_PLUGIN_ID, 'theater.director.ping', {})
    const result = raw as DirectorPingResult
    return result?.ok ? result : null
  }
  catch {
    return null
  }
}

export async function validateDirectorRules(params: {
  scene_id: string
  beats: Array<{ id?: string, speaker?: string, summary?: string }>
}): Promise<DirectorValidateRulesResult | null> {
  if (!(await isDirectorPluginAvailable())) {
    return null
  }
  try {
    return await directoryPluginInvoke(
      THEATER_DIRECTOR_PLUGIN_ID,
      'theater.director.validate_rules',
      params,
    ) as DirectorValidateRulesResult
  }
  catch {
    return null
  }
}

export async function injectDirectorBeat(params: {
  scene_id: string
  summary: string
  speaker?: TheaterSpeaker
  beat_id?: string
}): Promise<TheaterBeat | null> {
  if (!(await isDirectorPluginAvailable())) {
    return null
  }
  try {
    const raw = await directoryPluginInvoke(
      THEATER_DIRECTOR_PLUGIN_ID,
      'theater.director.inject_beat',
      params,
    ) as DirectorInjectBeatResult
    const beat = raw?.beat
    if (!beat?.text?.trim()) {
      return null
    }
    return {
      id: beat.id ?? `director_${Date.now()}`,
      speaker: beat.speaker === 'b' ? 'b' : 'a',
      text: beat.text.trim(),
      delay_ms: beat.delay_ms ?? 0,
    }
  }
  catch {
    return null
  }
}

export async function switchDirectorScene(sceneId: string): Promise<DirectorSwitchSceneResult | null> {
  if (!(await isDirectorPluginAvailable())) {
    return null
  }
  try {
    return await directoryPluginInvoke(
      THEATER_DIRECTOR_PLUGIN_ID,
      'theater.director.switch_scene',
      { scene_id: sceneId },
    ) as DirectorSwitchSceneResult
  }
  catch {
    return null
  }
}
