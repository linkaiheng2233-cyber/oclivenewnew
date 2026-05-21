/** ComfyUI-style graph editor tokens (light/dark via CSS variables). */

export type BackendKind = "builtin" | "remote" | "directory";

export const BACKEND_COLORS = {
  builtin: { bar: "#4CAF50", stroke: "#4CAF50", tagBg: "rgba(76, 175, 80, 0.18)" },
  remote: { bar: "#2196F3", stroke: "#2196F3", tagBg: "rgba(33, 150, 243, 0.18)" },
  directory: { bar: "#9C27B0", stroke: "#9C27B0", tagBg: "rgba(156, 39, 176, 0.18)" },
} as const;

export function normalizeBackendKind(raw: string): BackendKind {
  const v = raw.trim().toLowerCase();
  if (v === "remote") return "remote";
  if (v === "directory") return "directory";
  return "builtin";
}

export function edgeDash(kind: BackendKind): string {
  if (kind === "remote") return "8 4";
  if (kind === "directory") return "2 4";
  return "none";
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
