import type { ExpertRoutingDoc, ExpertTrigger } from '../api/role/expert'
import type { SlotRegistryMap } from './slotRegistry'

function formatTrigger(trigger: ExpertTrigger): string {
  const parts: string[] = []
  if (trigger.scene_ids?.length) {
    parts.push(`场景: ${trigger.scene_ids.join(', ')}`)
  }
  if (trigger.keywords?.length) {
    parts.push(`关键词: ${trigger.keywords.join(', ')}`)
  }
  if (trigger.min_message_length != null) {
    parts.push(`最短 ${trigger.min_message_length} 字`)
  }
  if (trigger.max_message_length != null) {
    parts.push(`最长 ${trigger.max_message_length} 字`)
  }
  return parts.join(' · ') || '专家路由'
}

/** 专家路由引用的 LLM 槽位 → tooltip 文案 */
export function expertLlmHighlights(
  doc: ExpertRoutingDoc | null | undefined,
  registry: SlotRegistryMap | null | undefined,
): Map<string, string> {
  const out = new Map<string, string>()
  if (!doc?.routes?.length || !registry) {
    return out
  }
  for (const route of doc.routes) {
    if (route.enabled === false) {
      continue
    }
    const hint = formatTrigger(route.trigger)
    for (const step of route.steps) {
      const m = /^slot\.([^.]+)\.(.+)$/.exec(step.action.trim())
      if (!m) {
        continue
      }
      const key = m[1]!
      if (registry[key]?.type === 'llm') {
        out.set(key, hint)
      }
    }
  }
  return out
}
