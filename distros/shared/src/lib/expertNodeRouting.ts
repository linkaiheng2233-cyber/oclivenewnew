import type {
  ExpertRoute,
  ExpertRoutingDoc,
  ExpertRouteStep,
  ExpertStepParams,
} from '@oclive/shared/api/role/expert'

export type ExpertNodeConfigKind = 'llm' | 'personality' | 'prompt' | 'memory' | 'lora'

export interface LlmNodeFormValues {
  llmSlotKey: string
  temperature: number
  maxTokens: number
}

export interface PersonalityNodeFormValues {
  traits: Record<string, number>
  delta: number
}

export interface PromptNodeFormValues {
  text: string
}

export interface MemoryNodeFormValues {
  content: string
}

export interface LoraNodeFormValues {
  pluginId: string
}

/** slot_registry keys referenced in expert routing steps (`slot.<key>.<method>`). */
export function expertReferencedSlotKeys(
  doc: ExpertRoutingDoc | null | undefined,
): Set<string> {
  const out = new Set<string>()
  if (!doc?.routes?.length) {
    return out
  }
  for (const route of doc.routes) {
    for (const step of route.steps) {
      const m = /^slot\.([^.]+)\./.exec(step.action.trim())
      if (!m) {
        continue
      }
      const key = m[1]!
      if (
        key !== 'personality'
        && key !== 'prompt_enhance'
        && key !== 'memory'
        && key !== 'lora'
        && key !== 'expert'
      ) {
        out.add(key)
      }
    }
  }
  return out
}

export function slotHasExpertGenerateStep(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
): boolean {
  if (!doc?.routes?.length) {
    return false
  }
  const action = `slot.${slotKey}.generate`
  return doc.routes.some(r =>
    r.steps.some(s => s.action.trim() === action),
  )
}

export function configKindForSlotType(slotType: string): ExpertNodeConfigKind {
  switch (slotType) {
    case 'llm':
      return 'llm'
    case 'prompt':
      return 'prompt'
    case 'memory':
      return 'memory'
    case 'lora':
      return 'lora'
    default:
      return 'personality'
  }
}

function cloneDoc(doc: ExpertRoutingDoc | null | undefined): ExpertRoutingDoc {
  if (!doc) {
    return { fallback: 'skip', routes: [] }
  }
  return JSON.parse(JSON.stringify(doc)) as ExpertRoutingDoc
}

function findRouteIndexForLlm(doc: ExpertRoutingDoc, slotKey: string): number {
  const action = `slot.${slotKey}.generate`
  return doc.routes.findIndex(r =>
    r.steps.some(s => s.action.trim() === action),
  )
}

function ensureLlmRoute(
  doc: ExpertRoutingDoc,
  slotKey: string,
): { doc: ExpertRoutingDoc, routeIndex: number } {
  const next = cloneDoc(doc)
  let idx = findRouteIndexForLlm(next, slotKey)
  if (idx >= 0) {
    return { doc: next, routeIndex: idx }
  }
  const route: ExpertRoute = {
    id: `expert-${slotKey}`,
    enabled: true,
    priority: 10,
    trigger: {},
    steps: [{ action: `slot.${slotKey}.generate`, depends_on: [] }],
  }
  next.routes = [...(next.routes ?? []), route]
  idx = next.routes.length - 1
  return { doc: next, routeIndex: idx }
}

function ensureFacilityRoute(
  doc: ExpertRoutingDoc,
  slotKey: string,
): { doc: ExpertRoutingDoc, routeIndex: number } {
  const next = cloneDoc(doc)
  const routeId = `expert-slot-${slotKey}`
  let idx = next.routes.findIndex(r => r.id === routeId)
  if (idx >= 0) {
    return { doc: next, routeIndex: idx }
  }
  const route: ExpertRoute = {
    id: routeId,
    enabled: true,
    priority: 5,
    trigger: {},
    steps: [],
  }
  next.routes = [...(next.routes ?? []), route]
  idx = next.routes.length - 1
  return { doc: next, routeIndex: idx }
}

function ensureEditingRoute(
  doc: ExpertRoutingDoc,
  slotKey: string,
  slotType: string,
): { doc: ExpertRoutingDoc, routeIndex: number } {
  if (slotType === 'llm') {
    return ensureLlmRoute(doc, slotKey)
  }
  return ensureFacilityRoute(doc, slotKey)
}

/** One-click add LLM slot to expert routing (if generate step not already present). */
export function ensureExpertRouteForLlmSlot(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
): ExpertRoutingDoc {
  return ensureLlmRoute(cloneDoc(doc), slotKey).doc
}

function readParams(step: ExpertRouteStep | undefined): ExpertStepParams {
  return (step?.params ?? {}) as ExpertStepParams
}

function findFacilityStep(
  route: ExpertRoute,
  action: string,
): ExpertRouteStep | undefined {
  return route.steps.find(s => s.action.trim() === action)
}

function upsertFacilityStep(
  route: ExpertRoute,
  action: string,
  params: ExpertStepParams,
): void {
  const idx = route.steps.findIndex(s => s.action.trim() === action)
  const dep
    = route.steps.length && idx < 0
      ? [route.steps[route.steps.length - 1]!.action]
      : []
  const step: ExpertRouteStep = {
    action,
    depends_on: idx >= 0 ? route.steps[idx]!.depends_on : dep,
    params,
  }
  if (idx >= 0) {
    route.steps[idx] = step
  }
  else {
    route.steps.push(step)
  }
}

export function loadLlmFormFromDoc(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
): LlmNodeFormValues {
  const fallback: LlmNodeFormValues = {
    llmSlotKey: slotKey,
    temperature: 0.7,
    maxTokens: 2048,
  }
  if (!doc) {
    return fallback
  }
  const idx = findRouteIndexForLlm(doc, slotKey)
  if (idx < 0) {
    return fallback
  }
  const step = doc.routes[idx]!.steps.find(
    s => s.action.trim() === `slot.${slotKey}.generate`,
  )
  const p = readParams(step) as Record<string, unknown>
  return {
    llmSlotKey:
      typeof p.model_slot === 'string' && p.model_slot
        ? p.model_slot
        : slotKey,
    temperature:
      typeof p.temperature === 'number' ? p.temperature : fallback.temperature,
    maxTokens:
      typeof p.max_tokens === 'number' ? p.max_tokens : fallback.maxTokens,
  }
}

export function applyLlmNodeConfig(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
  values: LlmNodeFormValues,
): ExpertRoutingDoc {
  const { doc: next, routeIndex } = ensureLlmRoute(cloneDoc(doc), slotKey)
  const route = next.routes[routeIndex]!
  const targetKey = values.llmSlotKey.trim() || slotKey
  const action = `slot.${targetKey}.generate`
  const params: ExpertStepParams = {
    temperature: values.temperature,
    max_tokens: values.maxTokens,
    ...(targetKey !== slotKey ? { model_slot: targetKey } : {}),
  }
  const genIdx = route.steps.findIndex(s =>
    s.action.trim().startsWith('slot.')
    && s.action.trim().endsWith('.generate'),
  )
  const step: ExpertRouteStep = {
    action,
    depends_on: genIdx >= 0 ? route.steps[genIdx]!.depends_on : [],
    params,
  }
  if (genIdx >= 0) {
    route.steps[genIdx] = step
  }
  else {
    route.steps.unshift(step)
  }
  return next
}

export function loadPersonalityFormFromDoc(
  doc: ExpertRoutingDoc | null | undefined,
  defaults: Record<string, number>,
): PersonalityNodeFormValues {
  for (const route of doc?.routes ?? []) {
    const step = findFacilityStep(route, 'slot.personality.adjust')
    if (step) {
      const p = readParams(step)
      const traits = { ...defaults }
      if (p.trait && typeof p.trait === 'string' && p.trait in traits) {
        const delta = typeof p.delta === 'number' ? p.delta : 0.05
        traits[p.trait] = Math.min(1, Math.max(0, traits[p.trait]! + delta))
      }
      return {
        traits,
        delta: typeof p.delta === 'number' ? p.delta : 0.05,
      }
    }
  }
  return { traits: { ...defaults }, delta: 0.05 }
}

export function applyPersonalityNodeConfig(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
  slotType: string,
  values: PersonalityNodeFormValues,
  primaryTrait: string,
): ExpertRoutingDoc {
  const { doc: next, routeIndex } = ensureEditingRoute(cloneDoc(doc), slotKey, slotType)
  const route = next.routes[routeIndex]!
  upsertFacilityStep(route, 'slot.personality.adjust', {
    trait: primaryTrait,
    delta: values.delta,
  })
  return next
}

export function loadPromptFormFromDoc(
  doc: ExpertRoutingDoc | null | undefined,
): PromptNodeFormValues {
  for (const route of doc?.routes ?? []) {
    const step = findFacilityStep(route, 'slot.prompt_enhance.apply')
    if (step) {
      const p = readParams(step)
      return { text: String(p.text ?? '') }
    }
  }
  return { text: '' }
}

export function applyPromptNodeConfig(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
  slotType: string,
  values: PromptNodeFormValues,
): ExpertRoutingDoc {
  const { doc: next, routeIndex } = ensureEditingRoute(cloneDoc(doc), slotKey, slotType)
  const route = next.routes[routeIndex]!
  upsertFacilityStep(route, 'slot.prompt_enhance.apply', { text: values.text })
  return next
}

export function loadMemoryFormFromDoc(
  doc: ExpertRoutingDoc | null | undefined,
): MemoryNodeFormValues {
  for (const route of doc?.routes ?? []) {
    const step = findFacilityStep(route, 'slot.memory.inject')
    if (step) {
      const p = readParams(step)
      return { content: String(p.content ?? '') }
    }
  }
  return { content: '' }
}

export function applyMemoryNodeConfig(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
  slotType: string,
  values: MemoryNodeFormValues,
): ExpertRoutingDoc {
  const { doc: next, routeIndex } = ensureEditingRoute(cloneDoc(doc), slotKey, slotType)
  const route = next.routes[routeIndex]!
  upsertFacilityStep(route, 'slot.memory.inject', {
    content: values.content,
    importance: 0.85,
  })
  return next
}

export function loadLoraFormFromDoc(
  doc: ExpertRoutingDoc | null | undefined,
): LoraNodeFormValues {
  for (const route of doc?.routes ?? []) {
    const step = findFacilityStep(route, 'slot.lora.apply')
    if (step) {
      const p = readParams(step)
      return { pluginId: String(p.plugin_id ?? '') }
    }
  }
  return { pluginId: '' }
}

export function applyLoraNodeConfig(
  doc: ExpertRoutingDoc | null | undefined,
  slotKey: string,
  slotType: string,
  values: LoraNodeFormValues,
): ExpertRoutingDoc {
  const { doc: next, routeIndex } = ensureEditingRoute(cloneDoc(doc), slotKey, slotType)
  const route = next.routes[routeIndex]!
  upsertFacilityStep(route, 'slot.lora.apply', {
    plugin_id: values.pluginId,
  })
  return next
}
