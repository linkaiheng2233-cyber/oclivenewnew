/** Radial layout + ComfyUI-style port anchors for architecture graph. */

import { comfyLinkPathDirected } from './graphEditorTheme'

export interface RadialNodeLayout {
  key: string
  x: number
  y: number
  cx: number
  cy: number
  /** Radians from center; 0 = east, −π/2 = top */
  angle: number
}

export interface RectBox { x: number, y: number, w: number, h: number }

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
  const angle = startAngle + (2 * Math.PI * index) / count
  const cx = centerX + radius * Math.cos(angle)
  const cy = centerY + radius * Math.sin(angle)
  return {
    key: '',
    x: cx - nodeWidth / 2,
    y: cy - nodeHeight / 2,
    cx,
    cy,
    angle,
  }
}

export function pointOnRay(cx: number, cy: number, angle: number, distance: number): { x: number, y: number } {
  return {
    x: cx + Math.cos(angle) * distance,
    y: cy + Math.sin(angle) * distance,
  }
}

/** Port on rectangle edge facing (toward) a target point — ComfyUI slot anchor. */
export function rectPortToward(
  box: RectBox,
  targetX: number,
  targetY: number,
): { x: number, y: number } {
  const cx = box.x + box.w / 2
  const cy = box.y + box.h / 2
  const ang = Math.atan2(targetY - cy, targetX - cx)
  const hw = box.w / 2
  const hh = box.h / 2
  const absCos = Math.abs(Math.cos(ang))
  const absSin = Math.abs(Math.sin(ang))
  let px: number
  let py: number
  if (absCos * hh > absSin * hw) {
    px = cx + Math.sign(Math.cos(ang)) * hw
    py = cy + Math.tan(ang) * Math.sign(Math.cos(ang)) * hw
    py = Math.max(box.y + 4, Math.min(box.y + box.h - 4, py))
  }
  else {
    py = cy + Math.sign(Math.sin(ang)) * hh
    px = cx + (Math.tan(ang) * Math.sign(Math.sin(ang)) * hh)
    px = Math.max(box.x + 4, Math.min(box.x + box.w - 4, px))
  }
  return { x: px, y: py }
}

/** Port on circle/hex kernel rim toward target. */
export function circlePortToward(
  cx: number,
  cy: number,
  radius: number,
  targetX: number,
  targetY: number,
): { x: number, y: number } {
  const ang = Math.atan2(targetY - cy, targetX - cx)
  return {
    x: cx + Math.cos(ang) * radius * 0.94,
    y: cy + Math.sin(ang) * radius * 0.94,
  }
}

/** Left-side input slot (ComfyUI) at fractional height. */
export function rectPortLeft(box: RectBox, fracY: number): { x: number, y: number } {
  return { x: box.x, y: box.y + box.h * fracY }
}

/** Right-side output slot at fractional height. */
export function rectPortRight(box: RectBox, fracY: number): { x: number, y: number } {
  return { x: box.x + box.w, y: box.y + box.h * fracY }
}

/** Wire between two boxes: out anchor on A toward B, in anchor on B toward A. */
export function linkBetweenRects(from: RectBox, to: RectBox): string {
  const p1 = rectPortToward(from, to.x + to.w / 2, to.y + to.h / 2)
  const p2 = rectPortToward(to, from.x + from.w / 2, from.y + from.h / 2)
  return comfyLinkPathDirected(p1.x, p1.y, p2.x, p2.y)
}

export function linkKernelToRect(
  kcx: number,
  kcy: number,
  kr: number,
  to: RectBox,
): string {
  const tc = { x: to.x + to.w / 2, y: to.y + to.h / 2 }
  const p1 = circlePortToward(kcx, kcy, kr, tc.x, tc.y)
  const p2 = rectPortToward(to, kcx, kcy)
  return comfyLinkPathDirected(p1.x, p1.y, p2.x, p2.y)
}

/** Bus right slot i of n → module (staged fan-out). */
export function linkBusSlotToModule(
  bus: RectBox,
  module: RectBox,
  slotIndex: number,
  slotCount: number,
): string {
  const frac = (slotIndex + 1) / (slotCount + 1)
  const p1 = rectPortRight(bus, frac)
  const p2 = rectPortToward(module, p1.x, p1.y)
  return comfyLinkPathDirected(p1.x, p1.y, p2.x, p2.y)
}

export function linkModuleToPlugin(module: RectBox, plugin: RectBox): string {
  const mc = { x: module.x + module.w / 2, y: module.y + module.h / 2 }
  const pc = { x: plugin.x + plugin.w / 2, y: plugin.y + plugin.h / 2 }
  const p1 = rectPortToward(module, pc.x, pc.y)
  const p2 = rectPortToward(plugin, mc.x, mc.y)
  return comfyLinkPathDirected(p1.x, p1.y, p2.x, p2.y)
}
