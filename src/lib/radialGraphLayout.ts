/** Radial (hub-and-spoke) layout for architecture graph — ComfyUI-style orbit. */

export type RadialNodeLayout = {
  key: string;
  x: number;
  y: number;
  cx: number;
  cy: number;
  /** Radians from center; 0 = east, −π/2 = top */
  angle: number;
};

export function layoutOnRing(
  centerX: number,
  centerY: number,
  radius: number,
  index: number,
  count: number,
  nodeWidth: number,
  nodeHeight: number,
  startAngle = -Math.PI / 2,
): RadialNodeLayout {
  const angle = startAngle + (2 * Math.PI * index) / count;
  const cx = centerX + radius * Math.cos(angle);
  const cy = centerY + radius * Math.sin(angle);
  return {
    key: "",
    x: cx - nodeWidth / 2,
    y: cy - nodeHeight / 2,
    cx,
    cy,
    angle,
  };
}

export function pointOnRay(cx: number, cy: number, angle: number, distance: number): { x: number; y: number } {
  return {
    x: cx + Math.cos(angle) * distance,
    y: cy + Math.sin(angle) * distance,
  };
}

/** Edge from center hub rim to node port facing the hub */
export function hubToNodeEdge(
  hubCx: number,
  hubCy: number,
  hubRadius: number,
  nodeCx: number,
  nodeCy: number,
  nodeAngle: number,
  nodeHalfWidth: number,
  bulge = 0.12,
): string {
  const from = pointOnRay(hubCx, hubCy, nodeAngle, hubRadius * 0.95);
  const to = pointOnRay(nodeCx, nodeCy, nodeAngle + Math.PI, nodeHalfWidth);
  return radialQuadraticPath(from.x, from.y, to.x, to.y, bulge);
}

/** Edge from facility node (outward) to directory plugin */
export function nodeToPluginEdge(
  nodeCx: number,
  nodeCy: number,
  nodeAngle: number,
  nodeHalfWidth: number,
  pluginCx: number,
  pluginCy: number,
  pluginHalfHeight: number,
  bulge = 0.1,
): string {
  const from = pointOnRay(nodeCx, nodeCy, nodeAngle, nodeHalfWidth);
  const to = pointOnRay(pluginCx, pluginCy, nodeAngle + Math.PI, pluginHalfHeight);
  return radialQuadraticPath(from.x, from.y, to.x, to.y, bulge);
}

export function radialQuadraticPath(x1: number, y1: number, x2: number, y2: number, bulge = 0.12): string {
  const mx = (x1 + x2) / 2;
  const my = (y1 + y2) / 2;
  const dx = x2 - x1;
  const dy = y2 - y1;
  const len = Math.hypot(dx, dy) || 1;
  const nx = (-dy / len) * len * bulge;
  const ny = (dx / len) * len * bulge;
  return `M ${x1} ${y1} Q ${mx + nx} ${my + ny} ${x2} ${y2}`;
}

/** Port anchor as % inside node box (toward hub = inward) */
export function portStylePercent(angle: number, inward: boolean): Record<string, string> {
  const a = inward ? angle + Math.PI : angle;
  const px = 50 + Math.cos(a) * 46;
  const py = 50 + Math.sin(a) * 46;
  return {
    left: `${px}%`,
    top: `${py}%`,
    transform: "translate(-50%, -50%)",
  };
}
