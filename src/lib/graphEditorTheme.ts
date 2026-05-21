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
