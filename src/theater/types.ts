export type TheaterSpeaker = 'a' | 'b'

export type TheaterMode = 'tweak' | 'outline' | 'improv'

export type TheaterTurnSpeaker = TheaterSpeaker | 'user'

export interface TheaterBeat {
  id: string
  speaker: TheaterSpeaker
  text: string
  delay_ms: number
}

export interface TheaterVariableDef {
  default: boolean | string
  label_zh: string
  label_en: string
}

export interface TheaterSkeleton {
  schema_version: number
  scene_id: string
  title: string
  role_a: string
  role_b: string
  variables: Record<string, TheaterVariableDef>
  impact_map: Record<string, string[]>
  beats: TheaterBeat[]
  patch_hints?: Record<string, string>
}

export type TheaterVariableState = Record<string, boolean | string>

export const THEATER_POKE_CHIP_IDS = [
  'bitter_medicine',
  'running_late',
  'nickname_change',
] as const

export type TheaterPokeChipId = (typeof THEATER_POKE_CHIP_IDS)[number]

export const NICKNAME_OPTIONS = ['default', '笨蛋', '大佬', '亲爱的'] as const

export const THEATER_MODES: TheaterMode[] = ['tweak', 'outline', 'improv']

/** Scene index entry from `public/theater/scenes.json`. */
export interface TheaterSceneMeta {
  scene_id: string
  title_zh: string
  title_en: string
  skeleton_path: string
}

export interface TheaterSceneIndex {
  schema_version: number
  scenes: TheaterSceneMeta[]
}

/** Mode 2 — editable outline before compile to skeleton. */
export interface TheaterOutlineBeat {
  id: string
  speaker: TheaterTurnSpeaker
  summary: string
}

export interface TheaterOutline {
  schema_version: number
  scene_id: string
  title: string
  role_a: string
  role_b: string
  beats: TheaterOutlineBeat[]
}

/** Mode 3 — live improv session transcript. */
export interface TheaterSessionTurn {
  id: string
  speaker: TheaterTurnSpeaker
  text: string
}

export interface TheaterSession {
  schema_version: number
  scene_id: string
  title: string
  role_a: string
  role_b: string
  turns: TheaterSessionTurn[]
}

export type DirectorPhase = 'waiting_user' | 'generating_a' | 'generating_b' | 'ended'
