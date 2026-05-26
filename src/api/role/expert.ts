import { invoke } from '@tauri-apps/api/tauri'

export interface MessageLengthRange {
  min?: number
  max?: number
}

export interface TimeOfDayWindow {
  after?: string
  before?: string
}

export interface ExpertTrigger {
  /** 新字段 */
  scenes?: string[]
  /** 兼容旧字段 */
  scene_ids?: string[]
  keywords?: string[]
  user_emotion?: string[]
  message_length?: MessageLengthRange
  min_message_length?: number
  max_message_length?: number
  time_of_day?: TimeOfDayWindow
  user_relation?: string[]
}

export interface ExpertStepParams {
  trait?: string
  delta?: number
  text?: string
  content?: string
  importance?: number
  plugin_id?: string
}

export interface ExpertRouteStep {
  action: string
  depends_on?: string[]
  params?: ExpertStepParams
}

export interface ExpertRoute {
  id?: string
  enabled?: boolean
  priority?: number
  trigger: ExpertTrigger
  steps: ExpertRouteStep[]
}

export interface ExpertRoutingDoc {
  routing_path?: string
  fallback?: 'skip' | 'retry_with_default'
  routes: ExpertRoute[]
}

export const EXPERT_FACILITY_ACTIONS = [
  'slot.personality.adjust',
  'slot.prompt_enhance.apply',
  'slot.memory.inject',
  'slot.lora.apply',
  'slot.expert.fallback',
] as const

export function listBlueprintIncludes(roleId: string): Promise<string[]> {
  return invoke('list_blueprint_includes', { roleId })
}

export function getExpertRouting(roleId: string): Promise<ExpertRoutingDoc | null> {
  return invoke('get_expert_routing', { roleId })
}

export function saveExpertRouting(roleId: string, doc: ExpertRoutingDoc): Promise<void> {
  return invoke('save_expert_routing', { roleId, doc })
}
