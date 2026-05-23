import { computed, onMounted, onUnmounted, ref } from 'vue'

export interface GraphCanvasOptions {
  worldWidth: number
  worldHeight: number
  minScale?: number
  maxScale?: number
}

export function useGraphCanvas(opts: GraphCanvasOptions) {
  const minScale = opts.minScale ?? 0.5
  const maxScale = opts.maxScale ?? 2
  const scale = ref(1)
  const panX = ref(0)
  const panY = ref(0)
  const viewportRef = ref<HTMLElement | null>(null)
  const spaceHeld = ref(false)
  const panning = ref(false)
  let panStartX = 0
  let panStartY = 0
  let panOriginX = 0
  let panOriginY = 0

  const transformStyle = computed(
    () =>
      `translate(${panX.value}px, ${panY.value}px) scale(${scale.value})`,
  )

  const gridStyle = computed(() => {
    const step = 18 * scale.value
    const dot = Math.max(1, 1.2 * scale.value)
    return {
      'backgroundSize': `${step}px ${step}px`,
      'backgroundPosition': `${panX.value % step}px ${panY.value % step}px`,
      '--graph-grid-dot': `${dot}px`,
    }
  })

  function clampScale(v: number): number {
    return Math.min(maxScale, Math.max(minScale, v))
  }

  function zoomIn(cx?: number, cy?: number): void {
    zoomByFactor(1.12, cx, cy)
  }

  function zoomOut(cx?: number, cy?: number): void {
    zoomByFactor(1 / 1.12, cx, cy)
  }

  function zoomByFactor(factor: number, cx?: number, cy?: number): void {
    const el = viewportRef.value
    const prev = scale.value
    const next = clampScale(prev * factor)
    if (!el || cx == null || cy == null) {
      scale.value = next
      return
    }
    const rect = el.getBoundingClientRect()
    const px = cx - rect.left
    const py = cy - rect.top
    const wx = (px - panX.value) / prev
    const wy = (py - panY.value) / prev
    scale.value = next
    panX.value = px - wx * next
    panY.value = py - wy * next
  }

  function resetView(): void {
    scale.value = 1
    panX.value = 0
    panY.value = 0
  }

  function fitWorld(pad = 24): void {
    const el = viewportRef.value
    if (!el)
      return
    const vw = el.clientWidth
    const vh = el.clientHeight
    const sx = (vw - pad * 2) / opts.worldWidth
    const sy = (vh - pad * 2) / opts.worldHeight
    const s = clampScale(Math.min(sx, sy, 1))
    scale.value = s
    panX.value = (vw - opts.worldWidth * s) / 2
    panY.value = (vh - opts.worldHeight * s) / 2
  }

  function focusPoint(wx: number, wy: number): void {
    const el = viewportRef.value
    if (!el)
      return
    const vw = el.clientWidth
    const vh = el.clientHeight
    panX.value = vw / 2 - wx * scale.value
    panY.value = vh / 2 - wy * scale.value
  }

  function onWheel(e: WheelEvent): void {
    e.preventDefault()
    zoomByFactor(e.deltaY > 0 ? 1 / 1.1 : 1.1, e.clientX, e.clientY)
  }

  function onPointerDown(e: PointerEvent): void {
    const middle = e.button === 1
    const spacePan = spaceHeld.value && e.button === 0
    if (!middle && !spacePan)
      return
    e.preventDefault()
    panning.value = true
    panStartX = e.clientX
    panStartY = e.clientY
    panOriginX = panX.value
    panOriginY = panY.value;
    (e.currentTarget as HTMLElement)?.setPointerCapture?.(e.pointerId)
  }

  function onPointerMove(e: PointerEvent): void {
    if (!panning.value)
      return
    panX.value = panOriginX + (e.clientX - panStartX)
    panY.value = panOriginY + (e.clientY - panStartY)
  }

  function onPointerUp(): void {
    panning.value = false
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.code === 'Space' && !e.repeat) {
      spaceHeld.value = true
    }
  }

  function onKeyUp(e: KeyboardEvent): void {
    if (e.code === 'Space') {
      spaceHeld.value = false
      panning.value = false
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown)
    window.addEventListener('keyup', onKeyUp)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeyDown)
    window.removeEventListener('keyup', onKeyUp)
  })

  const scalePercent = computed(() => `${Math.round(scale.value * 100)}%`)

  return {
    scale,
    getScale: () => scale.value,
    panX,
    panY,
    viewportRef,
    transformStyle,
    gridStyle,
    spaceHeld,
    panning,
    scalePercent,
    zoomIn,
    zoomOut,
    resetView,
    fitWorld,
    focusPoint,
    onWheel,
    onPointerDown,
    onPointerMove,
    onPointerUp,
  }
}
