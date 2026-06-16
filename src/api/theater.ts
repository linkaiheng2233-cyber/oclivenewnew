import { invokeWithFriendlyError } from './helpers'

export interface TheaterCastRef {
  role_id: string
  name: string
}

export interface TheaterScriptLine {
  id: string
  cast: string
  name: string
  text: string
  stage_hint?: string | null
  emotion?: string | null
}

export interface TheaterTweak {
  kind: 'chip' | 'custom' | string
  chip_label?: string | null
  drama_seed: string
  insert_after_beat_id: string
  lead_cast: string
}

export interface TheaterForkTemplate {
  chip_id: string
  insert_after_beat_id: string
  patch_lines: TheaterScriptLine[]
}

export interface TheaterPokeChipDef {
  chip_id: string
  drama_seed: string
  label?: string | null
}

export interface TheaterSceneRequest {
  cast_a: TheaterCastRef
  cast_b: TheaterCastRef
  scene_id: string
  base_beats: TheaterScriptLine[]
  applied_tweaks: TheaterTweak[]
  fallback_beats: TheaterScriptLine[]
  max_beats?: number | null
  /** `cast_adapt` for legacy merge; `cast_rewrite` for full script from personas */
  mode?: string | null
  fork_templates?: TheaterForkTemplate[] | null
  /** `voice` | `depth` | `polish` — legacy cast_adapt pass */
  adapt_pass?: string | null
  /** Poke chips for `cast_rewrite` */
  poke_chips?: TheaterPokeChipDef[] | null
}

export interface TheaterSceneResponse {
  beats: TheaterScriptLine[]
  /** `local` | `cloud` | `fallback` */
  source: string
  model: string
  adapted_forks?: TheaterForkTemplate[] | null
  /** When `source === 'fallback'` (e.g. `rewrite_llm_timeout`). */
  failure_reason?: string | null
  /** Partial success (e.g. `rewrite_forks_template`). */
  rewrite_note?: string | null
}

export function generateTheaterScene(
  req: TheaterSceneRequest,
): Promise<TheaterSceneResponse> {
  return invokeWithFriendlyError<TheaterSceneResponse>('generate_theater_scene', { req })
}
