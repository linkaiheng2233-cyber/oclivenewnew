import type { ExpertRoutingDoc, ExpertRoute, ExpertTrigger } from '../api/role/expert'

export function formatTriggerSummary(trigger: ExpertTrigger): string {
  const parts: string[] = []
  const scenes = trigger.scenes ?? trigger.scene_ids
  if (scenes?.length) {
    parts.push(`场景: ${scenes.join(', ')}`)
  }
  if (trigger.keywords?.length) {
    parts.push(`关键词: ${trigger.keywords.join(', ')}`)
  }
  if (trigger.user_emotion?.length) {
    parts.push(`情绪: ${trigger.user_emotion.join(', ')}`)
  }
  const len = trigger.message_length ?? {}
  const min = len.min ?? trigger.min_message_length
  const max = len.max ?? trigger.max_message_length
  if (min != null) {
    parts.push(`最短 ${min} 字`)
  }
  if (max != null) {
    parts.push(`最长 ${max} 字`)
  }
  if (trigger.time_of_day?.after || trigger.time_of_day?.before) {
    const a = trigger.time_of_day.after ?? '—'
    const b = trigger.time_of_day.before ?? '—'
    parts.push(`时段 ${a}–${b}`)
  }
  if (trigger.user_relation?.length) {
    parts.push(`关系: ${trigger.user_relation.join(', ')}`)
  }
  return parts.join(' · ') || '专家路由'
}

function triggerMatchesClient(
  trigger: ExpertTrigger,
  sceneId: string,
  userMessage: string,
): boolean {
  const scenes = trigger.scenes ?? trigger.scene_ids
  if (scenes?.length && !scenes.includes(sceneId)) {
    return false
  }
  if (trigger.keywords?.length) {
    const lower = userMessage.toLowerCase()
    if (!trigger.keywords.some(k => k && lower.includes(k.toLowerCase()))) {
      return false
    }
  }
  const len = trigger.message_length ?? {}
  const min = len.min ?? trigger.min_message_length
  const max = len.max ?? trigger.max_message_length
  const msgLen = [...userMessage].length
  if (min != null && msgLen < min) {
    return false
  }
  if (max != null && msgLen > max) {
    return false
  }
  return true
}

/** 与后端 `select_expert_route` 一致的简化匹配（前端预览，不含情绪/关系/时段）。 */
export function selectActiveExpertRoute(
  doc: ExpertRoutingDoc | null | undefined,
  sceneId: string,
  userMessage: string,
): ExpertRoute | undefined {
  if (!doc?.routes?.length) {
    return undefined
  }
  let best: { route: ExpertRoute, priority: number, idx: number } | undefined
  doc.routes.forEach((route, idx) => {
    if (route.enabled === false || !triggerMatchesClient(route.trigger, sceneId, userMessage)) {
      return
    }
    const pri = route.priority ?? 0
    if (
      !best
      || pri > best.priority
      || (pri === best.priority && idx < best.idx)
    ) {
      best = { route, priority: pri, idx }
    }
  })
  return best?.route
}

/** 专家路由引用的 LLM 槽位 → tooltip 文案 */
export function expertLlmHighlights(
  doc: ExpertRoutingDoc | null | undefined,
  registry: Record<string, { type: string }> | null | undefined,
): Map<string, string> {
  const out = new Map<string, string>()
  if (!doc?.routes?.length || !registry) {
    return out
  }
  for (const route of doc.routes) {
    if (route.enabled === false) {
      continue
    }
    const hint = formatTriggerSummary(route.trigger)
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
