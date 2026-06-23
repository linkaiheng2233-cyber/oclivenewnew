import { computed, ref } from 'vue'
import { getKernelDiagnostics } from '@oclive/shared/api/kernel'

const DEFAULT_UNLOCK_TURNS = 10

interface DistroUxProfile {
  immersiveUnlockHintAfterTurns: number
  allowModeSwitch: boolean
}

const profile = ref<DistroUxProfile>({
  immersiveUnlockHintAfterTurns: DEFAULT_UNLOCK_TURNS,
  allowModeSwitch: true,
})

let loaded = false

export async function ensureDistroUxProfileLoaded(): Promise<void> {
  if (loaded)
    return
  try {
    const diag = await getKernelDiagnostics()
    const summary = diag.healthJson?.active_profile_summary as {
      immersiveUnlockHintAfterTurns?: number
      allowModeSwitch?: boolean
    } | undefined
    if (summary) {
      if (typeof summary.immersiveUnlockHintAfterTurns === 'number' && summary.immersiveUnlockHintAfterTurns > 0) {
        profile.value.immersiveUnlockHintAfterTurns = summary.immersiveUnlockHintAfterTurns
      }
      if (typeof summary.allowModeSwitch === 'boolean') {
        profile.value.allowModeSwitch = summary.allowModeSwitch
      }
    }
  }
  catch {
    // keep defaults
  }
  loaded = true
}

export function useDistroUxProfile() {
  const immersiveUnlockHintAfterTurns = computed(
    () => profile.value.immersiveUnlockHintAfterTurns,
  )
  const allowModeSwitch = computed(() => profile.value.allowModeSwitch)

  return {
    immersiveUnlockHintAfterTurns,
    allowModeSwitch,
    ensureDistroUxProfileLoaded,
  }
}
