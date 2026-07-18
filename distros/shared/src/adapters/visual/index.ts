/**
 * Visual presentation adapter registry (Phase 4–6).
 * Host UI selects adapter by `performance_directive.kind`.
 */

export type VisualAdapterKind = 'image' | 'live2d' | 'rig3d' | 'procedural' | 'directory'

export interface VisualAdapterContext {
  /** social hero vs inner narrative shell (Phase 6) */
  mode: 'social' | 'inner'
}

export interface VisualAdapterHandle {
  kind: VisualAdapterKind
  mount: (el: HTMLElement) => void
  dispose: () => void
}

/** Image adapter: PNG/WebP hero (default Chat Pro path). */
export function createImageAdapter(): VisualAdapterHandle {
  return {
    kind: 'image',
    mount(_el: HTMLElement) {},
    dispose() {},
  }
}

/** Live2D adapter stub — Theater shell uses `Live2DStageAdapter.vue`. */
export function createLive2dAdapter(): VisualAdapterHandle {
  return {
    kind: 'live2d',
    mount(_el: HTMLElement) {},
    dispose() {},
  }
}

/** rig3d adapter stub (Phase 6). */
export function createRig3dAdapter(): VisualAdapterHandle {
  return {
    kind: 'rig3d',
    mount(_el: HTMLElement) {},
    dispose() {},
  }
}

/** Procedural adapter stub (Phase 6). */
export function createProceduralAdapter(): VisualAdapterHandle {
  return {
    kind: 'procedural',
    mount(_el: HTMLElement) {},
    dispose() {},
  }
}

/** Directory plugin adapter stub — `provides: ["visual_presentation.materialize"]`. */
export function createDirectoryAdapter(_pluginId: string): VisualAdapterHandle {
  return {
    kind: 'directory',
    mount(_el: HTMLElement) {},
    dispose() {},
  }
}

export function resolveVisualAdapter(
  kind: string,
  context: VisualAdapterContext = { mode: 'social' },
): VisualAdapterHandle {
  void context
  switch (kind) {
    case 'live2d':
      return createLive2dAdapter()
    case 'rig3d':
      return createRig3dAdapter()
    case 'procedural':
      return createProceduralAdapter()
    case 'directory':
      return createDirectoryAdapter('unknown')
    default:
      return createImageAdapter()
  }
}
