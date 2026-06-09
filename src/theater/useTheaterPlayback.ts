import { computed, onBeforeUnmount, ref, shallowRef, watch } from 'vue'
import type { TheaterBeat, TheaterSkeleton, TheaterVariableState } from './types'
import { defaultVariableState } from './useTheaterBeatPatch'

export function useTheaterPlayback(skeletonRef: () => TheaterSkeleton | null) {
  const displayedBeats = shallowRef<TheaterBeat[]>([])
  const visibleCount = ref(0)
  const playing = ref(false)
  const finished = ref(false)
  let timer: ReturnType<typeof setTimeout> | undefined

  const visibleBeats = computed(() =>
    displayedBeats.value.slice(0, visibleCount.value),
  )

  function clearTimer() {
    if (timer) {
      clearTimeout(timer)
      timer = undefined
    }
  }

  function resetPlayback(beats: TheaterBeat[]) {
    clearTimer()
    displayedBeats.value = beats.map(b => ({ ...b }))
    visibleCount.value = 0
    playing.value = false
    finished.value = false
  }

  function startPlayback() {
    const skeleton = skeletonRef()
    if (!skeleton || displayedBeats.value.length === 0) {
      return
    }
    clearTimer()
    playing.value = true
    finished.value = false
    visibleCount.value = 1
    scheduleNext(0)
  }

  function scheduleNext(index: number) {
    const beats = displayedBeats.value
    if (index >= beats.length - 1) {
      playing.value = false
      finished.value = true
      return
    }
    const delay = beats[index + 1]?.delay_ms ?? 2000
    timer = setTimeout(() => {
      visibleCount.value = index + 2
      scheduleNext(index + 1)
    }, delay)
  }

  function resumeAfterPatch() {
    if (finished.value) {
      return
    }
    if (visibleCount.value >= displayedBeats.value.length) {
      finished.value = true
      playing.value = false
      return
    }
    playing.value = true
    scheduleNext(visibleCount.value - 1)
  }

  watch(
    () => skeletonRef(),
    (sk) => {
      if (sk) {
        resetPlayback(sk.beats)
        startPlayback()
      }
    },
    { immediate: true },
  )

  onBeforeUnmount(clearTimer)

  return {
    displayedBeats,
    visibleBeats,
    visibleCount,
    playing,
    finished,
    resetPlayback,
    startPlayback,
    resumeAfterPatch,
  }
}

export function useTheaterVariables(skeletonRef: () => TheaterSkeleton | null) {
  const variables = ref<TheaterVariableState>({})

  watch(
    () => skeletonRef(),
    (sk) => {
      if (sk) {
        variables.value = defaultVariableState(sk)
      }
    },
    { immediate: true },
  )

  return { variables }
}
