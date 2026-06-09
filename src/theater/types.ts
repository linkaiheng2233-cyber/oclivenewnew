export type TheaterSpeaker = 'a' | 'b'

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
