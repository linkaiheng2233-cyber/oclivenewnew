/** Architecture graph palette — ComfyUI-style dark canvas + dot grid. */
export type BackendKind = 'builtin' | 'remote' | 'directory'
/** Dark graph chrome (fixed; independent of app light/dark theme). */
export const GRAPH_COMFY = {
  canvas: '#252526',
  gridDot: '#454545',
  gridGap: 20,
  gridDotSize: 1.25,
  nodeBg: '#2d2d30',
  nodeElevated: '#3c3c3c',
  nodeBorder: '#4e4e52',
  text: '#d4d4d4',
  textMuted: '#9d9d9d',
  nodeShadow: '0 4px 14px color-mix(in srgb, #000 42%, transparent)',
  selectionRing: '#6b8cae',
  kernelAccent: '#5a6a5a',
  busAccent: '#4a4a4e',
} as const
export interface BackendColorSet {
  bar: string
  stroke: string
  tagBg: string
  handle: string
  muted: string
}
/** Low-saturation accents readable on dark nodes (ComfyUI-like). */
export const BACKEND_COLORS: Record<BackendKind, BackendColorSet> = {
  builtin: {
    bar: '#6d9a7d',
    stroke: '#7aad8f',
    tagBg: 'color-mix(in srgb, #6d9a7d 18%, #2d2d30)',
    handle: '#7aad8f',
    muted: 'color-mix(in srgb, #6d9a7d 30%, #4e4e52)',
  },
  remote: {
    bar: '#7a92b0',
    stroke: '#8b9db8',
    tagBg: 'color-mix(in srgb, #7a92b0 18%, #2d2d30)',
    handle: '#8b9db8',
    muted: 'color-mix(in srgb, #7a92b0 28%, #4e4e52)',
  },
  directory: {
    bar: '#9a88a6',
    stroke: '#a899b0',
    tagBg: 'color-mix(in srgb, #9a88a6 18%, #2d2d30)',
    handle: '#a899b0',
    muted: 'color-mix(in srgb, #9a88a6 28%, #4e4e52)',
  },
}
export const GRAPH_SURFACE = GRAPH_COMFY
export function normalizeBackendKind(raw: string): BackendKind {
  const v = raw.trim().toLowerCase()
  if (v === 'remote')
    return 'remote'
  if (v === 'directory')
    return 'directory'
  return 'builtin'
}
export function edgeDash(kind: BackendKind): string {
  if (kind === 'remote')
    return '7 5'
  if (kind === 'directory')
    return '3 5'
  return 'none'
}
export function backendCssVars(kind: BackendKind): Record<string, string> {
  const c = BACKEND_COLORS[kind]
  return {
    '--arch-accent': c.bar,
    '--arch-stroke': c.stroke,
    '--arch-tag-bg': c.tagBg,
    '--arch-handle': c.handle,
    '--arch-muted': c.muted,
    '--arch-node-bg': GRAPH_COMFY.nodeBg,
    '--arch-node-border': GRAPH_COMFY.nodeBorder,
  }
}
/** Classic left→right ComfyUI cubic wire (horizontal tangents). */
export function comfyLinkPath(x1: number, y1: number, x2: number, y2: number): string {
  const dx = Math.abs(x2 - x1)
  const pull = Math.max(48, Math.min(160, dx * 0.45))
  const dir = x2 >= x1 ? 1 : -1
  return `M ${x1} ${y1} C ${x1 + pull * dir} ${y1}, ${x2 - pull * dir} ${y2}, ${x2} ${y2}`
}
/** Direction-aware cubic link for radial / arbitrary node positions. */
export function comfyLinkPathDirected(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  tangentScale = 0.42,
): string {
  const dx = x2 - x1
  const dy = y2 - y1
  const dist = Math.hypot(dx, dy) || 1
  const t = Math.min(140, dist * tangentScale)
  const c1x = x1 + (dx / dist) * t
  const c1y = y1 + (dy / dist) * t
  const c2x = x2 - (dx / dist) * t
  const c2y = y2 - (dy / dist) * t
  return `M ${x1} ${y1} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${x2} ${y2}`
}
export function bezierPath(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  curvature = 0.45,
): string {
  const dx = Math.max(40, Math.abs(x2 - x1) * curvature)
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
}
