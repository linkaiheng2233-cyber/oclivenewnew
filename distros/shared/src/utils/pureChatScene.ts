import type { InteractionMode } from './interactionMode'

/** Co-present scene for daily chat (simple hook path). */
export const PURE_CHAT_DEFAULT_SCENE_ID = 'home'

/** Daily chat always uses the simple scene; immersive uses UI scene picker. */
export function effectiveChatSceneId(
  interactionMode: InteractionMode,
  uiSceneId: string,
): string {
  if (interactionMode === 'pure_chat')
    return PURE_CHAT_DEFAULT_SCENE_ID
  const trimmed = uiSceneId.trim()
  return trimmed || 'default'
}
