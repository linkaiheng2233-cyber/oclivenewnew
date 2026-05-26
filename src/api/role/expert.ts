import { invoke } from '@tauri-apps/api/tauri'

export interface ExpertTrigger {
  scene_ids?: string[]
  keywords?: string[]
  min_message_length?: number
  max_message_length?: number
}

export interface ExpertRouteStep {
  action: string
  depends_on?: string[]
}

export interface ExpertRoute {
  id?: string
  enabled?: boolean
  trigger: ExpertTrigger
  steps: ExpertRouteStep[]
}

export interface ExpertRoutingDoc {
  routing_path?: string
  fallback?: 'skip' | 'retry_with_default'
  routes: ExpertRoute[]
}

export function listBlueprintIncludes(roleId: string): Promise<string[]> {
  return invoke('list_blueprint_includes', { roleId })
}

export function getExpertRouting(roleId: string): Promise<ExpertRoutingDoc | null> {
  return invoke('get_expert_routing', { roleId })
}

export function saveExpertRouting(roleId: string, doc: ExpertRoutingDoc): Promise<void> {
  return invoke('save_expert_routing', { roleId, doc })
}
