import type {
  AppliedTweak,
  PokeChipId,
  ScriptLine,
  TheaterCast,
  TheaterSkeleton,
  TheaterSourceKind,
  TheaterStageState,
} from './theater/theaterLogic'
import type { TheaterCastConfig } from './theater/theaterCastConfig'
import type { CastAdaptIssue } from './theater/theaterCastAdapt'
import { computed, inject, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { loadRole, setRoleInteractionMode } from '../api/role'
import {
  bindCastToSkeleton,
  castConfigChanged,
  DEFAULT_THEATER_CAST_CONFIG,
  enrichCastConfigFromRoles,
  getTheaterCastConfig,
  resolveCastTier,
  setTheaterCastConfig,
} from './theater/theaterCastConfig'
import {
  buildRuntimeFromRewrite,
  clearAdaptedCacheForCast,
  clearAllAdaptedCache,
  computeSkeletonHash,
  getAdaptedCache,
  needsCastAdaptation,
  setAdaptedCache,
  skeletonToForkTemplates,
} from './theater/theaterCastAdapt'
import {
  CAST_REWRITE_PROGRESS_KEY,
  pickCastRewritePreviewLine,
} from './theater/theaterCastAdaptPasses'
import {
  normalizePairRelationId,
  resolvePairRelationHint,
} from './theater/theaterPairRelation'
import {
  generateTheaterScene,
  type TheaterSceneRequest,
  type TheaterScriptLine,
  type TheaterTweak,
} from '../api/theater'
import { useRoleStore } from '../stores/roleStore'
import { hostEventBus } from '../lib/hostEventBus'
import { MAIN_SHELL_KEY } from './mainShellKey'
import {
  buildWorkingScript,
  beatsAfterInsert,
  cloneScriptLines,
  defaultInsertAnchor,
  fetchSkeletonForPreset,
  nextVisibleCount,
  pickCanFork,
  playbackDone,
  resolveChipLeadCast,
  SCENE_GEN_TIMEOUT_MS,
  SceneGenTimeoutError,
  timeoutReject,
} from './theater/theaterLogic'
import {
  getTheaterCustomLeadCast,
  getTheaterPokeMode,
  getTheaterVariantSwipeEnabled,
} from './useTheaterPokeSettings'
import {
  getPokeChipsForPreset,
  getTheaterScenePreset,
  getTheaterScenePresetId,
  listTheaterScenePresets,
  resolveActivePokeChips,
  setTheaterScenePresetId,
  type TheaterScenePreset,
  type TheaterScenePresetId,
} from './theater/theaterSceneCatalog'
import { ApiInvokeError } from '../api/helpers'

const LINE_REVEAL_MS = 720
const THINK_STEP_MS = 650
const CAST_ADAPT_DONE_VISIBLE_MS = 1000
/** Two kernel attempts × cast_rewrite timeout (default 45s) + buffer. */
const CAST_REWRITE_TIMEOUT_MS = 100_000

export interface PokeVariant {
  id: 'a' | 'b'
  patchLines: ScriptLine[]
  fullBeats: ScriptLine[]
  source: TheaterSourceKind
}

interface PendingVariantContext {
  key: string
  tweaks: AppliedTweak[]
  insertAfterBeatId: string
}

function extractPatchSegment(
  beats: ScriptLine[],
  insertAfterBeatId: string,
  baseBeats: ScriptLine[],
): ScriptLine[] {
  const tailIds = new Set(beatsAfterInsert(baseBeats, insertAfterBeatId).map(b => b.id))
  const anchorIdx = beats.findIndex(b => b.id === insertAfterBeatId)
  if (anchorIdx < 0)
    return []
  const tailIdx = beats.findIndex((b, i) => i > anchorIdx && tailIds.has(b.id))
  const end = tailIdx >= 0 ? tailIdx : beats.length
  return beats.slice(anchorIdx + 1, end)
}

function pokeVariantKey(
  presetId: string,
  chipId: PokeChipId | 'custom',
  tweakIndex: number,
): string {
  return `${presetId}:${chipId}:${tweakIndex}`
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function castName(sk: TheaterSkeleton, cast: TheaterCast): string {
  return cast === 'a' ? sk.cast.a.name : sk.cast.b.name
}

function toScriptLineDto(line: ScriptLine): TheaterScriptLine {
  return {
    id: line.id,
    cast: line.cast,
    name: line.name,
    text: line.text,
    stage_hint: line.stageHint ?? undefined,
    emotion: line.emotion ?? undefined,
  }
}

function fromScriptLineDto(line: TheaterScriptLine): ScriptLine {
  return {
    id: line.id,
    cast: line.cast as TheaterCast,
    name: line.name,
    text: line.text,
    stageHint: line.stage_hint ?? undefined,
    emotion: line.emotion ?? undefined,
  }
}

function tweakToDto(
  tweak: AppliedTweak,
  translate: (key: string) => string,
  presetId: TheaterScenePresetId,
): TheaterTweak {
  let chipLabel: string | undefined
  if (tweak.kind === 'chip' && tweak.chipId) {
    const chip = getPokeChipsForPreset(presetId).find(c => c.id === tweak.chipId)
    chipLabel = chip ? translate(chip.labelKey) : tweak.chipId
  }
  else if (tweak.kind === 'custom') {
    chipLabel = translate('theater.poke.customLabel')
  }
  return {
    kind: tweak.kind,
    chip_label: chipLabel,
    drama_seed: tweak.dramaSeed,
    insert_after_beat_id: tweak.insertAfterBeatId,
    lead_cast: tweak.leadCast,
  }
}

function mapFooterSource(source: string): TheaterSourceKind {
  if (source === 'cloud')
    return 'cloud'
  if (source === 'local')
    return 'local'
  return 'pregen'
}

function beatsEqual(a: ScriptLine[], b: ScriptLine[]): boolean {
  if (a.length !== b.length)
    return false
  return a.every((line, i) => {
    const other = b[i]
    return other != null && line.id === other.id && line.text === other.text
  })
}

export function useTheaterShell() {
  const shell = inject(MAIN_SHELL_KEY)
  const roleStore = useRoleStore()
  const { t, te } = useI18n()

  const activeScenePresetId = ref<TheaterScenePresetId>(getTheaterScenePresetId())
  const canonicalSkeleton = shallowRef<TheaterSkeleton | null>(null)
  const skeleton = shallowRef<TheaterSkeleton | null>(null)
  const displayLines = ref<ScriptLine[]>([])
  const visibleCount = ref(0)
  const stageState = ref<TheaterStageState>('playing')
  const footerSource = ref<TheaterSourceKind>('pregen')
  const funnelVisible = ref(false)
  const loadError = ref<string | null>(null)
  const appliedTweaks = ref<AppliedTweak[]>([])
  const settingsOpen = ref(false)
  const settingsTab = ref<'general' | 'stage' | 'cast' | 'model'>('general')

  const thinkingActive = ref(false)
  const thinkingSteps = ref<string[]>([])
  const thinkingChipLabel = ref('')
  const waitingSeconds = ref(0)
  const waitingPhase = ref<'thinking' | 'model'>('thinking')
  const thinkingTitle = computed(() =>
    t('theater.think.title', { chip: thinkingChipLabel.value }),
  )

  /** Hover preview on poke chips — highlights the affected cast on stage portraits. */
  const previewChipId = ref<PokeChipId | null>(null)
  /** While a poke/custom tweak runs, keep the affected cast highlighted. */
  const patchingEventCast = ref<TheaterCast | null>(null)

  const pokeVariants = ref(new Map<string, [PokeVariant, PokeVariant?]>())
  const activeVariantKey = ref<string | null>(null)
  const activeVariantIndex = ref<0 | 1>(0)
  const variantBackdropOpen = ref(false)
  const pendingVariantContext = ref<PendingVariantContext | null>(null)
  const variantPair = computed(() => {
    const key = activeVariantKey.value
    if (!key)
      return null
    return pokeVariants.value.get(key) ?? null
  })
  const variantPatchA = computed(() => variantPair.value?.[0]?.patchLines ?? [])
  const variantPatchB = computed(() => variantPair.value?.[1]?.patchLines ?? [])
  const variantBReady = computed(() => (variantPair.value?.[1]?.patchLines.length ?? 0) > 0)

  const castAdaptActive = ref(false)
  const castAdaptSteps = ref<string[]>([])
  const castAdaptPassProgress = ref<{ current: number, total: number, label: string } | null>(null)
  const castAdaptWaitingSeconds = ref(0)
  const castAdaptWaitingPhase = ref<'thinking' | 'model'>('thinking')
  const castAdaptLastIssue = ref<CastAdaptIssue | null>(null)
  const castAdaptProgressLabel = computed(() => {
    const p = castAdaptPassProgress.value
    if (!p)
      return ''
    return t('theater.think.castPass.progress', p)
  })

  let revealTimer: ReturnType<typeof setInterval> | null = null
  let waitingTimer: ReturnType<typeof setInterval> | null = null
  let castAdaptWaitingTimer: ReturnType<typeof setInterval> | null = null
  let thinkToken = 0
  let castAdaptToken = 0
  let skeletonLoadPromise: Promise<void> | null = null

  const visibleLines = computed(() => displayLines.value.slice(0, visibleCount.value))

  const castLabel = computed(() => {
    const sk = skeleton.value
    if (!sk)
      return ''
    return `${sk.cast.a.name} ✕ ${sk.cast.b.name}`
  })

  const castTier = computed(() => {
    const saved = getTheaterCastConfig()
    const sk = skeleton.value
    if (!sk) {
      return resolveCastTier(enrichCastConfigFromRoles(saved, roleStore.roles))
    }
    return resolveCastTier({
      ...saved,
      castA: { roleId: sk.cast.a.roleId, displayName: sk.cast.a.name },
      castB: { roleId: sk.cast.b.roleId, displayName: sk.cast.b.name },
    })
  })

  const castInfo = computed(() => skeleton.value?.cast ?? null)

  const activeScenePreset = computed(() => getTheaterScenePreset(activeScenePresetId.value))
  const scenePresets = computed<TheaterScenePreset[]>(() => listTheaterScenePresets())

  const sceneLabelKey = computed(() => activeScenePreset.value.labelKey)

  const activePokeChips = computed(() =>
    resolveActivePokeChips(activeScenePreset.value, skeleton.value),
  )

  const pokeEnabled = computed(() => activePokeChips.value.length > 0)

  const dockDisabled = computed(() => stageState.value === 'patching' || !pokeEnabled.value)

  const eventHighlightCast = computed((): TheaterCast | null => {
    if (stageState.value === 'patching' && patchingEventCast.value)
      return patchingEventCast.value
    const sk = skeleton.value
    const chipId = previewChipId.value
    if (sk && chipId)
      return resolveChipLeadCast(sk, chipId)
    return null
  })

  function setPreviewChip(chipId: PokeChipId | null) {
    if (dockDisabled.value) {
      previewChipId.value = null
      return
    }
    previewChipId.value = chipId
  }

  function clearWaitingTimer() {
    if (waitingTimer != null) {
      clearInterval(waitingTimer)
      waitingTimer = null
    }
  }

  function clearCastAdaptWaitingTimer() {
    if (castAdaptWaitingTimer != null) {
      clearInterval(castAdaptWaitingTimer)
      castAdaptWaitingTimer = null
    }
  }

  function startWaitingTimer() {
    clearWaitingTimer()
    waitingSeconds.value = 0
    waitingPhase.value = 'thinking'
    waitingTimer = setInterval(() => {
      waitingSeconds.value += 1
    }, 1000)
  }

  function startCastAdaptWaitingTimer() {
    clearCastAdaptWaitingTimer()
    castAdaptWaitingSeconds.value = 0
    castAdaptWaitingPhase.value = 'thinking'
    castAdaptWaitingTimer = setInterval(() => {
      castAdaptWaitingSeconds.value += 1
    }, 1000)
  }

  function clearPokeThinkingState() {
    thinkingActive.value = false
    thinkingSteps.value = []
    patchingEventCast.value = null
    clearWaitingTimer()
    waitingSeconds.value = 0
    waitingPhase.value = 'thinking'
  }

  function clearCastAdaptProgress() {
    castAdaptActive.value = false
    castAdaptSteps.value = []
    castAdaptPassProgress.value = null
    clearCastAdaptWaitingTimer()
    castAdaptWaitingSeconds.value = 0
    castAdaptWaitingPhase.value = 'thinking'
  }

  function appendCastAdaptStep(token: number, step: string): boolean {
    if (token !== castAdaptToken)
      return false
    castAdaptSteps.value = [...castAdaptSteps.value, step]
    return true
  }

  function isKernelOfflineError(err: unknown): boolean {
    if (!(err instanceof ApiInvokeError))
      return false
    const code = err.code ?? ''
    const raw = err.raw.toLowerCase()
    return code === 'KERNEL_HTTP_UNAVAILABLE'
      || code === 'KERNEL_ATTACH_FAILED'
      || code === 'REMOTE_SERVICE_UNAVAILABLE'
      || raw.includes('connection refused')
      || raw.includes('failed to fetch')
      || raw.includes('内核')
  }

  function sceneGenErrorToast(err: unknown): string {
    if (err instanceof SceneGenTimeoutError)
      return t('theater.poke.sceneTimeout')
    if (isKernelOfflineError(err))
      return t('theater.poke.kernelOffline')
    return t('theater.poke.sceneFailed')
  }
  function clearRevealTimer() {
    if (revealTimer != null) {
      clearInterval(revealTimer)
      revealTimer = null
    }
  }

  function startReveal(from = 0) {
    clearRevealTimer()
    visibleCount.value = from
    stageState.value = 'playing'
    revealTimer = setInterval(() => {
      if (playbackDone(visibleCount.value, displayLines.value.length)) {
        clearRevealTimer()
        stageState.value = 'idle'
        return
      }
      visibleCount.value = nextVisibleCount(visibleCount.value, displayLines.value.length)
    }, LINE_REVEAL_MS)
  }

  async function ensureRole(roleId: string, sceneId: string) {
    if (roleStore.currentRoleId !== roleId)
      await roleStore.switchRole(roleId)
    if (roleStore.roleInfo.interactionMode !== 'pure_chat') {
      const info = await setRoleInteractionMode(roleId, 'pure_chat')
      roleStore.applyRoleInfo(info)
    }
    if (shell && roleStore.currentRoleId === roleId) {
      void shell.chatStore.loadMessagesForRoleScene(roleId, sceneId)
    }
  }

  async function ensureBothCastLoaded(sk: TheaterSkeleton) {
    const sceneId = sk.sceneId ?? 'home'
    await loadRole(sk.cast.a.roleId)
    await loadRole(sk.cast.b.roleId)
    await ensureRole(sk.cast.a.roleId, sceneId)
  }

  function applyRuntimeCast(canonical: TheaterSkeleton, config: TheaterCastConfig): TheaterSkeleton {
    return bindCastToSkeleton(canonical, config)
  }

  function clearCastAdaptCache(): number {
    return clearAllAdaptedCache()
  }

  async function ensureCanonicalSkeletonLoaded(): Promise<TheaterSkeleton | null> {
    if (canonicalSkeleton.value)
      return canonicalSkeleton.value
    if (skeletonLoadPromise) {
      await skeletonLoadPromise
      return canonicalSkeleton.value
    }
    skeletonLoadPromise = loadSkeleton(activeScenePresetId.value)
    try {
      await skeletonLoadPromise
    }
    finally {
      skeletonLoadPromise = null
    }
    return canonicalSkeleton.value
  }

  function sceneContextFields(preset: TheaterScenePreset) {
    return {
      theater_scene: preset.id,
      scene_brief: preset.sceneBrief,
      scene_setting_hint: preset.sceneSettingHint,
    }
  }

  function presetIdForCanonical(canonical: TheaterSkeleton): string {
    return canonical.scene
  }

  function pairRelationRequestFields(config: TheaterCastConfig): {
    pair_relation_id: string
    pair_relation_hint: string
  } {
    const id = normalizePairRelationId(config.pairRelationId)
    return {
      pair_relation_id: id,
      pair_relation_hint: resolvePairRelationHint(id, canonicalSkeleton.value),
    }
  }

  async function reAdaptCurrentCast() {
    const canonical = await ensureCanonicalSkeletonLoaded()
    if (!canonical)
      return

    const config = enrichCastConfigFromRoles(getTheaterCastConfig(), roleStore.roles)
    if (!needsCastAdaptation(config)) {
      shell?.showToast('info', t('theater.cast.reAdaptDefaultHint'))
      return
    }

    clearAllAdaptedCache()
    clearCastAdaptProgress()
    appliedTweaks.value = []
    funnelVisible.value = false
    footerSource.value = 'pregen'

    const baseline = applyRuntimeCast(canonical, config)
    skeleton.value = baseline
    displayLines.value = cloneScriptLines(baseline.beats)
    startReveal(0)

    const runtime = await adaptRuntimeSkeleton(canonical, config, {
      skipCache: true,
      showProgress: true,
      showToast: false,
    })
    skeleton.value = runtime
    displayLines.value = cloneScriptLines(runtime.beats)
    await ensureBothCastLoaded(runtime)
    startReveal(0)
    const renamedOnly = beatsEqual(runtime.beats, baseline.beats)
    if (renamedOnly)
      castAdaptFallbackToast()
    else
      shell?.showToast('success', t('theater.cast.reAdaptDone'))
    await delay(CAST_ADAPT_DONE_VISIBLE_MS)
    clearCastAdaptProgress()
  }

  async function applyDefaultCast() {
    const canonical = await ensureCanonicalSkeletonLoaded()
    if (!canonical)
      return

    castAdaptToken++
    clearCastAdaptProgress()
    setTheaterCastConfig({ ...DEFAULT_THEATER_CAST_CONFIG })
    appliedTweaks.value = []
    funnelVisible.value = false
    footerSource.value = 'pregen'

    const runtime = bindCastToSkeleton(canonical, DEFAULT_THEATER_CAST_CONFIG)
    skeleton.value = runtime
    displayLines.value = cloneScriptLines(runtime.beats)
    await ensureBothCastLoaded(runtime)
    startReveal(0)
    shell?.showToast('success', t('theater.cast.restoreDefaultDone'))
  }

  function castAdaptFallbackToast() {
    const issue = castAdaptLastIssue.value
    if (issue) {
      const key = `theater.cast.issue.${issue.code}`
      shell?.showToast('info', te(key) ? t(key) : t('theater.cast.adaptFallback'))
      return
    }
    shell?.showToast('info', t('theater.cast.adaptFallback'))
  }

  function setCastAdaptIssue(failureReason?: string | null, rewriteNote?: string | null) {
    if (failureReason?.trim()) {
      castAdaptLastIssue.value = { kind: 'failure', code: failureReason.trim() }
      return
    }
    if (rewriteNote?.trim()) {
      castAdaptLastIssue.value = { kind: 'degraded', code: rewriteNote.trim() }
      return
    }
    castAdaptLastIssue.value = null
  }

  function castAdaptIssueStepLabel(issue: CastAdaptIssue): string {
    const key = `theater.cast.issue.${issue.code}`
    return te(key)
      ? t(key)
      : issue.kind === 'failure'
        ? t('theater.cast.issue.unknown')
        : t('theater.cast.issue.degradedUnknown')
  }

  function classifyCastRewriteInvokeError(err: unknown): string {
    if (err instanceof SceneGenTimeoutError)
      return 'client_timeout'
    if (isKernelOfflineError(err))
      return 'kernel_offline'
    if (err instanceof ApiInvokeError) {
      const blob = `${err.message}\n${err.raw}`.toLowerCase()
      if (blob.includes('base_beats must not be empty') || blob.includes('cast_rewrite'))
        return 'kernel_stale_cast_rewrite'
    }
    return 'invoke_error'
  }

  async function runCastRewrite(
    baseline: TheaterSkeleton,
    preset: TheaterScenePreset,
    token: number,
    showProgress: boolean,
    castConfig: TheaterCastConfig,
  ): Promise<{
    runtime: TheaterSkeleton
    source: TheaterSourceKind
    failedEarly: boolean
    failureReason?: string
    rewriteNote?: string
  }> {
    const rewriteLabel = t(CAST_REWRITE_PROGRESS_KEY)

    if (showProgress) {
      castAdaptPassProgress.value = { current: 1, total: 1, label: rewriteLabel }
      appendCastAdaptStep(token, t('theater.think.rewrite.readPersona'))
      appendCastAdaptStep(token, t(`theater.think.rewrite.${preset.id}`, t('theater.think.rewrite.start')))
      castAdaptWaitingPhase.value = 'model'
      if (castAdaptWaitingTimer == null)
        startCastAdaptWaitingTimer()
    }

    const pokeChips = preset.pokeEnabled
      ? getPokeChipsForPreset(preset.id).map(chip => ({
          chip_id: chip.id,
          drama_seed: chip.dramaSeed,
          label: t(chip.labelKey),
        }))
      : []

    const req: TheaterSceneRequest = {
      cast_a: { role_id: baseline.cast.a.roleId, name: baseline.cast.a.name },
      cast_b: { role_id: baseline.cast.b.roleId, name: baseline.cast.b.name },
      scene_id: baseline.sceneId ?? preset.runtimeSceneId,
      base_beats: [],
      applied_tweaks: [],
      fallback_beats: baseline.beats.map(toScriptLineDto),
      fork_templates: preset.pokeEnabled ? skeletonToForkTemplates(baseline) : [],
      mode: 'cast_rewrite',
      poke_chips: pokeChips,
      max_beats: 12,
      ...sceneContextFields(preset),
      ...pairRelationRequestFields(castConfig),
    }

    try {
      const resp = await Promise.race([
        generateTheaterScene(req),
        timeoutReject(CAST_REWRITE_TIMEOUT_MS),
      ])

      if (token !== castAdaptToken)
        return { runtime: baseline, source: 'pregen', failedEarly: false }

      if (resp.source === 'fallback') {
        const failureReason = resp.failure_reason?.trim() || 'rewrite_unknown'
        setCastAdaptIssue(failureReason, null)
        if (showProgress)
          appendCastAdaptStep(token, castAdaptIssueStepLabel({ kind: 'failure', code: failureReason }))
        return { runtime: baseline, source: 'pregen', failedEarly: true, failureReason }
      }

      const forks = resp.adapted_forks ?? []
      const runtime = buildRuntimeFromRewrite(baseline, resp.beats, forks)
      const source = mapFooterSource(resp.source)
      const rewriteNote = resp.rewrite_note?.trim() || undefined
      if (rewriteNote)
        setCastAdaptIssue(null, rewriteNote)
      else
        setCastAdaptIssue(null, null)

      if (showProgress) {
        const preview = pickCastRewritePreviewLine(runtime.beats)
        appendCastAdaptStep(
          token,
          preview
            ? t('theater.think.rewrite.donePreview', { preview })
            : t('theater.think.rewrite.done'),
        )
        if (rewriteNote)
          appendCastAdaptStep(token, castAdaptIssueStepLabel({ kind: 'degraded', code: rewriteNote }))
        castAdaptPassProgress.value = null
        castAdaptWaitingPhase.value = 'thinking'
      }

      return { runtime, source, failedEarly: false, rewriteNote }
    }
    catch (err) {
      const failureReason = classifyCastRewriteInvokeError(err)
      setCastAdaptIssue(failureReason, null)
      if (showProgress) {
        appendCastAdaptStep(token, t('theater.think.rewrite.failed'))
        appendCastAdaptStep(token, castAdaptIssueStepLabel({ kind: 'failure', code: failureReason }))
      }
      return { runtime: baseline, source: 'pregen', failedEarly: true, failureReason }
    }
  }

  async function adaptRuntimeSkeleton(
    canonical: TheaterSkeleton,
    config: TheaterCastConfig,
    options?: {
      showProgress?: boolean
      showToast?: boolean
      skipCache?: boolean
      /** When set (e.g. from applyCastConfig), reuse instead of bumping castAdaptToken again. */
      adaptToken?: number
    },
  ): Promise<TheaterSkeleton> {
    const showToastMessage = options?.showToast ?? true
    const baseline = applyRuntimeCast(canonical, config)
    if (!needsCastAdaptation(config))
      return baseline

    const presetId = presetIdForCanonical(canonical)
    const skeletonHash = computeSkeletonHash(canonical)
    if (!options?.skipCache) {
      const cached = getAdaptedCache(config, presetId, skeletonHash)
      if (cached) {
        if (showToastMessage)
          shell?.showToast('info', t('theater.cast.cacheHit'))
        footerSource.value = mapFooterSource(cached.source)
        return buildRuntimeFromRewrite(baseline, cached.beats, cached.forks)
      }
    }

    const showProgress = options?.showProgress ?? true
    const token = options?.adaptToken ?? ++castAdaptToken
    if (showProgress) {
      if (options?.adaptToken == null)
        clearCastAdaptProgress()
      castAdaptActive.value = true
      castAdaptSteps.value = []
      castAdaptLastIssue.value = null
      startCastAdaptWaitingTimer()
    }

    try {
      const { runtime, source, failedEarly } = await runCastRewrite(
        baseline,
        getTheaterScenePreset(presetId as TheaterScenePresetId),
        token,
        showProgress,
        config,
      )

      if (token !== castAdaptToken)
        return baseline

      const equivalentToBaseline = beatsEqual(runtime.beats, baseline.beats)

      if (failedEarly || equivalentToBaseline) {
        if (showToastMessage)
          castAdaptFallbackToast()
        footerSource.value = 'pregen'
        return baseline
      }

      const finalForks = skeletonToForkTemplates(runtime)
      setAdaptedCache(config, presetId, {
        skeletonHash,
        beats: runtime.beats.map(toScriptLineDto),
        forks: finalForks,
        source,
        ts: Date.now(),
      })
      footerSource.value = source

      return runtime
    }
    catch (err) {
      if (token !== castAdaptToken)
        throw err
      if (showToastMessage)
        castAdaptFallbackToast()
      footerSource.value = 'pregen'
      return baseline
    }
    finally {
      if (showProgress && token !== castAdaptToken)
        clearCastAdaptProgress()
    }
  }

  async function applyCastConfig(config: TheaterCastConfig) {
    const applyToken = ++castAdaptToken
    clearCastAdaptProgress()

    const presetId = activeScenePresetId.value
    const previousConfig = enrichCastConfigFromRoles(getTheaterCastConfig(), roleStore.roles)
    const enriched = enrichCastConfigFromRoles(config, roleStore.roles)

    let canonical: TheaterSkeleton | null
    try {
      canonical = await fetchSkeletonForPreset(presetId)
      canonicalSkeleton.value = canonical
    }
    catch {
      canonical = await ensureCanonicalSkeletonLoaded()
    }
    if (!canonical) {
      shell?.showToast('error', t('theater.cast.applyFailed'))
      return
    }

    if (castConfigChanged(previousConfig, enriched))
      clearAdaptedCacheForCast(previousConfig, presetId)

    setTheaterCastConfig(enriched)
    castAdaptLastIssue.value = null

    appliedTweaks.value = []
    funnelVisible.value = false
    footerSource.value = 'pregen'

    const baseline = applyRuntimeCast(canonical, enriched)
    skeleton.value = baseline
    displayLines.value = cloneScriptLines(baseline.beats)
    startReveal(0)

    if (!needsCastAdaptation(enriched)) {
      await ensureBothCastLoaded(baseline)
      shell?.showToast('success', t('theater.cast.applyDone'))
      return
    }

    let runtime = baseline
    let adaptErrored = false
    try {
      runtime = await adaptRuntimeSkeleton(canonical, enriched, {
        skipCache: true,
        showProgress: true,
        showToast: false,
        adaptToken: applyToken,
      })
    }
    catch (err) {
      adaptErrored = true
      console.warn('[theater] cast adapt failed; keeping rename-only baseline', err)
    }

    if (applyToken !== castAdaptToken)
      return

    skeleton.value = runtime
    displayLines.value = cloneScriptLines(runtime.beats)
    await ensureBothCastLoaded(runtime)
    startReveal(0)

    if (adaptErrored) {
      castAdaptFallbackToast()
    }
    else {
      const renamedOnly = beatsEqual(runtime.beats, baseline.beats)
      if (renamedOnly)
        castAdaptFallbackToast()
      else
        shell?.showToast('success', t('theater.cast.applyDone'))
    }

    await delay(CAST_ADAPT_DONE_VISIBLE_MS)
    clearCastAdaptProgress()
  }

  async function revealThinkingChain(token: number, name: string) {
    const steps = [
      t('theater.think.steps.recall'),
      t('theater.think.steps.mood', { name }),
      t('theater.think.steps.replay'),
      t('theater.think.steps.draft'),
    ]
    thinkingSteps.value = []
    for (const step of steps) {
      if (token !== thinkToken)
        return
      thinkingSteps.value = [...thinkingSteps.value, step]
      await delay(THINK_STEP_MS)
    }
    if (token === thinkToken)
      waitingPhase.value = 'model'
  }

  function buildSceneRequest(
    sk: TheaterSkeleton,
    tweaks: AppliedTweak[],
    options?: { mode?: string, patchVariant?: number },
  ): TheaterSceneRequest {
    const fallbackBeats = buildWorkingScript(sk.beats, tweaks)
    const castConfig = enrichCastConfigFromRoles(getTheaterCastConfig(), roleStore.roles)
    const preset = getTheaterScenePreset(sk.scene as TheaterScenePresetId)
    const pokeMode = options?.mode ?? getTheaterPokeMode()
    return {
      cast_a: { role_id: sk.cast.a.roleId, name: sk.cast.a.name },
      cast_b: { role_id: sk.cast.b.roleId, name: sk.cast.b.name },
      scene_id: sk.sceneId ?? preset.runtimeSceneId,
      base_beats: sk.beats.map(toScriptLineDto),
      applied_tweaks: tweaks.map(tweak => tweakToDto(tweak, t, preset.id)),
      fallback_beats: fallbackBeats.map(toScriptLineDto),
      mode: pokeMode,
      patch_variant: options?.patchVariant ?? undefined,
      ...sceneContextFields(preset),
      ...pairRelationRequestFields(castConfig),
    }
  }

  async function generateAndReplay(
    tweaks: AppliedTweak[],
    options?: { mode?: string, patchVariant?: number },
  ): Promise<{
    beats: ScriptLine[]
    source: TheaterSourceKind
    usedFallback: boolean
    timedOut: boolean
  }> {
    const sk = skeleton.value
    if (!sk) {
      return { beats: [], source: 'pregen', usedFallback: true, timedOut: false }
    }

    const fallbackBeats = buildWorkingScript(sk.beats, tweaks)
    const req = buildSceneRequest(sk, tweaks, options)
    await ensureBothCastLoaded(sk)

    try {
      const resp = await Promise.race([
        generateTheaterScene(req),
        timeoutReject(SCENE_GEN_TIMEOUT_MS),
      ])
      const beats = resp.beats.map(fromScriptLineDto)
      const usedFallback = resp.source === 'fallback'
      const finalBeats = beats.length > 0 ? beats : fallbackBeats
      return {
        beats: finalBeats,
        source: mapFooterSource(resp.source),
        usedFallback,
        timedOut: false,
      }
    }
    catch (err) {
      if (err instanceof SceneGenTimeoutError) {
        return {
          beats: fallbackBeats,
          source: 'pregen',
          usedFallback: true,
          timedOut: true,
        }
      }
      throw err
    }
  }

  async function generateVariantB(ctx: PendingVariantContext) {
    const sk = skeleton.value
    if (!sk || !getTheaterVariantSwipeEnabled() || getTheaterPokeMode() !== 'patch')
      return
    try {
      const resp = await generateTheaterScene(
        buildSceneRequest(sk, ctx.tweaks, { mode: 'patch', patchVariant: 1 }),
      )
      if (resp.source === 'fallback' || resp.beats.length === 0)
        return
      const beats = resp.beats.map(fromScriptLineDto)
      const patchLines = extractPatchSegment(beats, ctx.insertAfterBeatId, sk.beats)
      if (patchLines.length === 0)
        return
      const existing = pokeVariants.value.get(ctx.key)
      if (!existing?.[0])
        return
      const variantB: PokeVariant = {
        id: 'b',
        patchLines,
        fullBeats: beats,
        source: mapFooterSource(resp.source),
      }
      pokeVariants.value.set(ctx.key, [existing[0], variantB])
      variantBackdropOpen.value = true
      shell?.showToast('info', t('theater.variant.ready'))
    }
    catch {
      /* second candidate optional */
    }
  }

  function storePokeVariantA(
    key: string,
    beats: ScriptLine[],
    insertAfterBeatId: string,
    source: TheaterSourceKind,
  ) {
    const sk = skeleton.value
    if (!sk)
      return
    const patchLines = extractPatchSegment(beats, insertAfterBeatId, sk.beats)
    pokeVariants.value.set(key, [{
      id: 'a',
      patchLines,
      fullBeats: cloneScriptLines(beats),
      source,
    }])
    activeVariantKey.value = key
    activeVariantIndex.value = 0
    variantBackdropOpen.value = false
  }

  function selectPokeVariant(index: 0 | 1) {
    const key = activeVariantKey.value
    if (!key)
      return
    const pair = pokeVariants.value.get(key)
    const chosen = pair?.[index]
    if (!chosen)
      return
    activeVariantIndex.value = index
    displayLines.value = cloneScriptLines(chosen.fullBeats)
    footerSource.value = chosen.source
    const lastTweak = appliedTweaks.value[appliedTweaks.value.length - 1]
    if (lastTweak)
      lastTweak.patchLines = cloneScriptLines(chosen.patchLines)
    visibleCount.value = displayLines.value.length
    variantBackdropOpen.value = false
  }

  function dismissVariantBackdrop() {
    variantBackdropOpen.value = false
  }

  function scheduleVariantBAfterReveal(ctx: PendingVariantContext) {
    if (!getTheaterVariantSwipeEnabled() || getTheaterPokeMode() !== 'patch')
      return
    pendingVariantContext.value = ctx
  }

  type SceneGenOutcome = {
    beats: ScriptLine[]
    source: TheaterSourceKind
    applied: boolean
    reason?: 'timeout' | 'fallback' | 'error'
  }

  async function runSceneGeneration(
    tweaks: AppliedTweak[],
    leadName: string,
    mode: 'chip' | 'custom',
  ): Promise<SceneGenOutcome> {
    const sk = skeleton.value!
    const token = thinkToken
    thinkingActive.value = true
    startWaitingTimer()

    try {
      const [, result] = await Promise.all([
        revealThinkingChain(token, leadName),
        generateAndReplay(tweaks),
      ])

      if (result.timedOut) {
        if (mode === 'chip') {
          shell?.showToast('info', t('theater.poke.sceneTimeout'))
          return { beats: result.beats, source: result.source, applied: true, reason: 'timeout' }
        }
        return {
          beats: buildWorkingScript(sk.beats, appliedTweaks.value),
          source: 'pregen',
          applied: false,
          reason: 'timeout',
        }
      }

      if (mode === 'custom') {
        const fallbackBefore = buildWorkingScript(sk.beats, tweaks)
        if (result.usedFallback && beatsEqual(result.beats, fallbackBefore)) {
          return {
            beats: buildWorkingScript(sk.beats, appliedTweaks.value),
            source: result.source,
            applied: false,
            reason: 'fallback',
          }
        }
      }

      return { beats: result.beats, source: result.source, applied: true }
    }
    catch (err) {
      stageState.value = 'idle'
      shell?.showToast('info', sceneGenErrorToast(err))
      const fallbackBeats = buildWorkingScript(
        sk.beats,
        mode === 'chip' ? tweaks : appliedTweaks.value,
      )
      return { beats: fallbackBeats, source: 'pregen', applied: false, reason: 'error' }
    }
    finally {
      clearPokeThinkingState()
    }
  }

  async function onPoke(chipId: PokeChipId) {
    const sk = skeleton.value
    if (!sk || stageState.value === 'patching')
      return

    const fork = pickCanFork(sk, chipId)
    if (!fork)
      return

    stageState.value = 'patching'
    clearRevealTimer()

    const leadCast = resolveChipLeadCast(sk, chipId) ?? (fork.patchLines[0]?.cast ?? 'a')
    patchingEventCast.value = leadCast
    const name = castName(sk, leadCast)
    const preset = activeScenePreset.value
    const chip = getPokeChipsForPreset(preset.id).find(c => c.id === chipId)
    const chipLabel = chip ? t(chip.labelKey) : chipId
    const dramaSeed = chip?.dramaSeed ?? ''
    thinkingChipLabel.value = chipLabel
    thinkToken += 1

    const tweak: AppliedTweak = {
      kind: 'chip',
      chipId,
      dramaSeed,
      insertAfterBeatId: fork.insertAfterBeatId,
      leadCast,
      anchorLines: fork.patchLines,
      patchLines: cloneScriptLines(fork.patchLines),
    }
    const prior = appliedTweaks.value.filter(
      t => !(t.kind === 'chip' && t.chipId === chipId),
    )
    const tweaks = [...prior, tweak]

    const outcome = await runSceneGeneration(tweaks, name, 'chip')
    if (!outcome.applied) {
      stageState.value = 'idle'
      return
    }

    appliedTweaks.value = tweaks
    footerSource.value = outcome.source
    displayLines.value = outcome.beats
    const vKey = pokeVariantKey(preset.id, chipId, tweaks.length - 1)
    storePokeVariantA(vKey, outcome.beats, tweak.insertAfterBeatId, outcome.source)
    scheduleVariantBAfterReveal({
      key: vKey,
      tweaks,
      insertAfterBeatId: tweak.insertAfterBeatId,
    })
    startReveal(0)
  }

  async function onCustomTweak(text: string) {
    const sk = skeleton.value
    if (!sk || stageState.value === 'patching')
      return

    const dramaSeed = text.trim()
    if (!dramaSeed)
      return

    stageState.value = 'patching'
    clearRevealTimer()

    const insertAfterBeatId = defaultInsertAnchor(sk)
    const leadCast: TheaterCast = getTheaterCustomLeadCast()
    patchingEventCast.value = leadCast
    const name = castName(sk, leadCast)
    const chipLabel = t('theater.poke.customLabel')
    thinkingChipLabel.value = chipLabel
    thinkToken += 1

    const tweak: AppliedTweak = {
      kind: 'custom',
      dramaSeed,
      insertAfterBeatId,
      leadCast,
      anchorLines: [],
      patchLines: [],
    }
    const tweaks = [...appliedTweaks.value, tweak]

    const outcome = await runSceneGeneration(tweaks, name, 'custom')
    if (!outcome.applied) {
      stageState.value = 'idle'
      if (outcome.reason === 'timeout')
        shell?.showToast('info', t('theater.poke.customTimeout'))
      else if (outcome.reason === 'fallback')
        shell?.showToast('info', t('theater.poke.customFailed'))
      return
    }

    appliedTweaks.value = tweaks
    footerSource.value = outcome.source
    displayLines.value = outcome.beats
    const preset = activeScenePreset.value
    const vKey = pokeVariantKey(preset.id, 'custom', tweaks.length - 1)
    storePokeVariantA(vKey, outcome.beats, insertAfterBeatId, outcome.source)
    scheduleVariantBAfterReveal({
      key: vKey,
      tweaks,
      insertAfterBeatId,
    })
    startReveal(0)
  }

  function dismissFunnel() {
    funnelVisible.value = false
  }

  async function onFunnelCreate() {
    funnelVisible.value = false
    const roleId = skeleton.value?.cast.a.roleId ?? 'mumu'
    const { openPackEditorForRole } = await import('../utils/openPackEditor')
    const result = await openPackEditorForRole(roleId)
    if (!result.ok && shell)
      shell.showToast('info', result.message ?? '')
  }

  function restartScene() {
    const sk = skeleton.value
    if (!sk)
      return
    thinkToken++
    clearPokeThinkingState()
    appliedTweaks.value = []
    funnelVisible.value = false
    footerSource.value = 'pregen'
    pokeVariants.value = new Map()
    activeVariantKey.value = null
    variantBackdropOpen.value = false
    pendingVariantContext.value = null
    displayLines.value = cloneScriptLines(sk.beats)
    startReveal(0)
  }

  async function loadSkeleton(presetId: TheaterScenePresetId) {
    const canonical = await fetchSkeletonForPreset(presetId)
    canonicalSkeleton.value = canonical
    loadError.value = null
    try {
      const castConfig = enrichCastConfigFromRoles(getTheaterCastConfig(), roleStore.roles)
      if (!needsCastAdaptation(castConfig)) {
        const runtime = applyRuntimeCast(canonical, castConfig)
        skeleton.value = runtime
        displayLines.value = cloneScriptLines(runtime.beats)
        footerSource.value = 'pregen'
        startReveal(0)
      }
      else {
        const baseline = applyRuntimeCast(canonical, castConfig)
        const cachePresetId = presetIdForCanonical(canonical)
        const skeletonHash = computeSkeletonHash(canonical)
        const cached = getAdaptedCache(castConfig, cachePresetId, skeletonHash)
        if (cached) {
          skeleton.value = buildRuntimeFromRewrite(baseline, cached.beats, cached.forks)
          footerSource.value = mapFooterSource(cached.source)
        }
        else {
          skeleton.value = baseline
          footerSource.value = 'pregen'
        }
        displayLines.value = cloneScriptLines(skeleton.value.beats)
        startReveal(0)
      }
    }
    catch (e) {
      loadError.value = e instanceof Error ? e.message : String(e)
      stageState.value = 'idle'
    }
  }

  async function switchScenePreset(id: TheaterScenePresetId) {
    if (id === activeScenePresetId.value)
      return

    castAdaptToken++
    clearCastAdaptProgress()
    clearPokeThinkingState()
    appliedTweaks.value = []
    funnelVisible.value = false
    footerSource.value = 'pregen'

    activeScenePresetId.value = id
    setTheaterScenePresetId(id)
    canonicalSkeleton.value = null

    shell?.showToast('info', t('theater.scene.switching'))

    await loadSkeleton(id)
    const canonical = canonicalSkeleton.value
    if (!canonical)
      return

    const config = enrichCastConfigFromRoles(getTheaterCastConfig(), roleStore.roles)
    const baseline = applyRuntimeCast(canonical, config)
    skeleton.value = baseline
    displayLines.value = cloneScriptLines(baseline.beats)
    startReveal(0)

    if (needsCastAdaptation(config)) {
      const runtime = await adaptRuntimeSkeleton(canonical, config, {
        skipCache: false,
        showProgress: false,
        showToast: false,
      })
      skeleton.value = runtime
      displayLines.value = cloneScriptLines(runtime.beats)
      startReveal(0)
      const skeletonHash = computeSkeletonHash(canonical)
      const cachePresetId = presetIdForCanonical(canonical)
      const cacheHit = getAdaptedCache(config, cachePresetId, skeletonHash)
      if (!cacheHit && beatsEqual(runtime.beats, baseline.beats))
        castAdaptFallbackToast()
    }

    await ensureBothCastLoaded(skeleton.value)
  }

  function closeSettings() {
    settingsOpen.value = false
    if (shell)
      shell.settingsViewOpen.value = false
  }

  function openSettingsToTab(tab: 'general' | 'stage' | 'cast' | 'model') {
    settingsTab.value = tab
    settingsOpen.value = true
    shell?.closeModelManager()
    if (shell)
      shell.settingsViewOpen.value = false
  }

  function openSettings() {
    settingsOpen.value = !settingsOpen.value
    if (shell) {
      shell.settingsViewOpen.value = false
      shell.closeModelManager()
    }
  }

  function onTheaterSettingsEvent(payload?: { action?: string }) {
    const action = payload?.action
    if (action === 'close' || action === 'escape') {
      if (funnelVisible.value) {
        dismissFunnel()
        return
      }
      if (settingsOpen.value) {
        closeSettings()
      }
      return
    }
    if (action === 'model') {
      openSettingsToTab('model')
      return
    }
    if (action === 'open') {
      settingsOpen.value = true
      shell?.closeModelManager()
      if (shell)
        shell.settingsViewOpen.value = false
      return
    }
    if (action === 'toggle') {
      openSettings()
      return
    }
    openSettings()
  }

  function onHostOpenModelManagerInTheater() {
    openSettingsToTab('model')
  }

  let stopSettingsWatch: (() => void) | undefined

  onMounted(async () => {
    shell?.closeModelManager()
    hostEventBus.on('theater:settings', onTheaterSettingsEvent)
    hostEventBus.on('ui:open_model_manager', onHostOpenModelManagerInTheater)
    if (shell) {
      if (shell.settingsViewOpen.value)
        shell.settingsViewOpen.value = false
      stopSettingsWatch = watch(
        () => shell.settingsViewOpen.value,
        (open) => {
          if (open) {
            settingsOpen.value = true
            shell.settingsViewOpen.value = false
          }
        },
      )
    }

    try {
      if (roleStore.roles.length === 0)
        await roleStore.loadRoles()
      skeletonLoadPromise = loadSkeleton(activeScenePresetId.value)
      await skeletonLoadPromise
      skeletonLoadPromise = null
      const sk = skeleton.value
      if (sk)
        await ensureBothCastLoaded(sk)
    }
    catch (e) {
      loadError.value = e instanceof Error ? e.message : String(e)
      stageState.value = 'idle'
    }
  })

  onBeforeUnmount(() => {
    clearRevealTimer()
    clearWaitingTimer()
    clearCastAdaptWaitingTimer()
    hostEventBus.off('theater:settings', onTheaterSettingsEvent)
    hostEventBus.off('ui:open_model_manager', onHostOpenModelManagerInTheater)
    stopSettingsWatch?.()
  })

  watch(displayLines, () => {
    if (stageState.value === 'playing' && playbackDone(visibleCount.value, displayLines.value.length))
      stageState.value = 'idle'
  })

  watch(stageState, (state, prev) => {
    if (state === 'idle' && prev === 'playing') {
      patchingEventCast.value = null
      const ctx = pendingVariantContext.value
      if (ctx) {
        pendingVariantContext.value = null
        void generateVariantB(ctx)
      }
    }
  })

  const castAdaptSkeletonHash = computed(() => {
    const canonical = canonicalSkeleton.value
    return canonical ? computeSkeletonHash(canonical) : ''
  })

  const castAdaptPresetId = computed(() => canonicalSkeleton.value?.scene ?? activeScenePresetId.value)

  const castSkeletonReady = computed(() => canonicalSkeleton.value != null)

  return {
    skeleton,
    canonicalSkeleton,
    castSkeletonReady,
    displayLines,
    visibleLines,
    visibleCount,
    stageState,
    footerSource,
    funnelVisible,
    loadError,
    castLabel,
    castTier,
    castInfo,
    activeScenePresetId,
    scenePresets,
    sceneLabelKey,
    activePokeChips,
    pokeEnabled,
    dockDisabled,
    thinkingActive,
    thinkingSteps,
    thinkingTitle,
    waitingSeconds,
    waitingPhase,
    castAdaptActive,
    castAdaptSteps,
    castAdaptPassProgress,
    castAdaptProgressLabel,
    castAdaptWaitingSeconds,
    castAdaptWaitingPhase,
    castAdaptLastIssue,
    castAdaptSkeletonHash,
    castAdaptPresetId,
    onPoke,
    onCustomTweak,
    setPreviewChip,
    eventHighlightCast,
    variantBackdropOpen,
    variantBReady,
    variantPatchA,
    variantPatchB,
    selectPokeVariant,
    dismissVariantBackdrop,
    dismissFunnel,
    onFunnelCreate,
    restartScene,
    settingsOpen,
    settingsTab,
    openSettings,
    closeSettings,
    applyCastConfig,
    applyDefaultCast,
    clearCastAdaptCache,
    reAdaptCurrentCast,
    switchScenePreset,
    showToast: shell?.showToast,
  }
}
