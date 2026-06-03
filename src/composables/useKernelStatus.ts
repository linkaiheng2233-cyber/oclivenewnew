import type { DesktopKernelMode } from '../api/kernel'
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { fetchRoleSnapshot, getKernelConnectionStatus } from '../api/kernel'
import { useRoleStore } from '../stores/roleStore'
import { useChatStore } from '../stores/chatStore'

const POLL_MS = 8000

export function useRoleSnapshotPoll() {
  const roleStore = useRoleStore()
  const chatStore = useChatStore()
  let timer: ReturnType<typeof setInterval> | undefined

  async function tick() {
    if (chatStore.isLoading) {
      return
    }
    const roleId = roleStore.currentRoleId
    if (!roleId) {
      return
    }
    try {
      const conn = await getKernelConnectionStatus()
      if (!conn.healthy) {
        return
      }
    }
    catch {
      return
    }
    const snap = await fetchRoleSnapshot(
      roleId,
      roleStore.roleInfo.userPresenceScene ?? roleStore.roleInfo.currentScene ?? undefined,
    )
    if (!snap) {
      return
    }
    roleStore.roleInfo.favorability = snap.current_favorability
    roleStore.roleInfo.currentEmotion = snap.portrait_emotion || snap.current_emotion
    roleStore.roleInfo.relationState = snap.relation_state
    if (snap.current_scene != null) {
      roleStore.roleInfo.currentScene = snap.current_scene
    }
    if (snap.user_presence_scene != null) {
      roleStore.roleInfo.userPresenceScene = snap.user_presence_scene
    }
  }

  function start() {
    stop()
    timer = setInterval(() => { void tick() }, POLL_MS)
    window.addEventListener('focus', onFocus)
  }

  function stop() {
    if (timer) {
      clearInterval(timer)
      timer = undefined
    }
    window.removeEventListener('focus', onFocus)
  }

  function onFocus() {
    void tick()
  }

  onMounted(start)
  onBeforeUnmount(stop)

  return { tick, start, stop }
}

export function kernelModeLabel(mode: DesktopKernelMode, t: (k: string) => string): string {
  switch (mode) {
    case 'attached':
      return t('kernel.status.attached')
    case 'spawned':
      return t('kernel.status.spawned')
    case 'reconnecting':
      return t('kernel.status.reconnecting')
    default:
      return t('kernel.status.offline')
  }
}
