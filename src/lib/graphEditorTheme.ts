/** Architecture graph palette — ivory (象牙白) base, low-saturation accents. */

export type BackendKind = "builtin" | "remote" | "directory";

/** Warm ivory family for canvas / nodes (works as fixed graph chrome). */
export const IVORY = {
  base: "#FFFAF0",
  paper: "#FAF8F4",
  card: "#FFFEF9",
  elevated: "#F7F5F0",
  border: "#E8E4DC",
  borderMuted: "#DDD8CF",
  grid: "#D8D4CC",
  taupe: "#B8B4AC",
  taupeCool: "#B0B4B0",
  taupeRose: "#B8B0AC",
  inkMuted: "#8A857C",
  shadow: "0 2px 10px color-mix(in srgb, #6d6860 9%, transparent)",
} as const;

export type BackendColorSet = {
  bar: string;
  stroke: string;
  tagBg: string;
  handle: string;
  muted: string;
};

/** Backend distinction via warm/cool taupe on ivory — no saturated primaries. */
export const BACKEND_COLORS: Record<BackendKind, BackendColorSet> = {
  builtin: {
    bar: IVORY.taupe,
    stroke: "#A8A39A",
    tagBg: `color-mix(in srgb, ${IVORY.base} 72%, ${IVORY.elevated})`,
    handle: "#A8A39A",
    muted: `color-mix(in srgb, ${IVORY.taupe} 28%, ${IVORY.border})`,
  },
  remote: {
    bar: IVORY.taupeCool,
    stroke: "#9EA39E",
    tagBg: `color-mix(in srgb, ${IVORY.base} 68%, ${IVORY.borderMuted})`,
    handle: "#9EA39E",
    muted: `color-mix(in srgb, ${IVORY.taupeCool} 26%, ${IVORY.border})`,
  },
  directory: {
    bar: IVORY.taupeRose,
    stroke: "#A69E98",
    tagBg: `color-mix(in srgb, ${IVORY.card} 70%, ${IVORY.border})`,
    handle: "#A69E98",
    muted: `color-mix(in srgb, ${IVORY.taupeRose} 26%, ${IVORY.border})`,
  },
};

export const GRAPH_SURFACE = {
  canvas: IVORY.paper,
  grid: `color-mix(in srgb, ${IVORY.grid} 42%, transparent)`,
  nodeBg: IVORY.card,
  nodeBorder: IVORY.border,
  nodeShadow: IVORY.shadow,
  selectionRing: `color-mix(in srgb, ${IVORY.inkMuted} 35%, ${IVORY.base})`,
  kernelAccent: IVORY.borderMuted,
  busAccent: IVORY.border,
} as const;

export function normalizeBackendKind(raw: string): BackendKind {
  const v = raw.trim().toLowerCase();
  if (v === "remote") return "remote";
  if (v === "directory") return "directory";
  return "builtin";
}

export function edgeDash(kind: BackendKind): string {
  if (kind === "remote") return "7 5";
  if (kind === "directory") return "3 5";
  return "none";
}

export function backendCssVars(kind: BackendKind): Record<string, string> {
  const c = BACKEND_COLORS[kind];
  return {
    "--arch-accent": c.bar,
    "--arch-stroke": c.stroke,
    "--arch-tag-bg": c.tagBg,
    "--arch-handle": c.handle,
    "--arch-muted": c.muted,
    "--arch-node-bg": IVORY.card,
    "--arch-node-border": IVORY.border,
  };
}

/** Classic left→right ComfyUI cubic wire (horizontal tangents). */
export function comfyLinkPath(x1: number, y1: number, x2: number, y2: number): string {
  const dx = Math.abs(x2 - x1);
  const pull = Math.max(48, Math.min(160, dx * 0.45));
  const dir = x2 >= x1 ? 1 : -1;
  return `M ${x1} ${y1} C ${x1 + pull * dir} ${y1}, ${x2 - pull * dir} ${y2}, ${x2} ${y2}`;
}

/** Direction-aware cubic link for radial / arbitrary node positions. */
export function comfyLinkPathDirected(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  tangentScale = 0.42,
): string {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const dist = Math.hypot(dx, dy) || 1;
  const t = Math.min(140, dist * tangentScale);
  const c1x = x1 + (dx / dist) * t;
  const c1y = y1 + (dy / dist) * t;
  const c2x = x2 - (dx / dist) * t;
  const c2y = y2 - (dy / dist) * t;
  return `M ${x1} ${y1} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${x2} ${y2}`;
}

export function bezierPath(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  curvature = 0.45,
): string {
  const dx = Math.max(40, Math.abs(x2 - x1) * curvature);
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
}
