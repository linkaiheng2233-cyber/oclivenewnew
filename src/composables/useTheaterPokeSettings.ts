import type { TheaterCast } from './theater/theaterLogic'

export type TheaterPokeMode = 'patch' | 'ripple'

const POKE_MODE_KEY = 'oclive.theater.pokeMode.v1'
const VARIANT_SWIPE_KEY = 'oclive.theater.variantSwipe.v1'
const CUSTOM_LEAD_KEY = 'oclive.theater.customLeadCast.v1'

export function getTheaterPokeMode(): TheaterPokeMode {
  try {
    const v = localStorage.getItem(POKE_MODE_KEY)
    if (v === 'ripple')
      return 'ripple'
  }
  catch {
    /* ignore */
  }
  return 'patch'
}

export function setTheaterPokeMode(mode: TheaterPokeMode): void {
  try {
    localStorage.setItem(POKE_MODE_KEY, mode)
  }
  catch {
    /* ignore */
  }
}

export function getTheaterVariantSwipeEnabled(): boolean {
  try {
    const v = localStorage.getItem(VARIANT_SWIPE_KEY)
    if (v === '0' || v === 'false')
      return false
  }
  catch {
    /* ignore */
  }
  return true
}

export function setTheaterVariantSwipeEnabled(enabled: boolean): void {
  try {
    localStorage.setItem(VARIANT_SWIPE_KEY, enabled ? '1' : '0')
  }
  catch {
    /* ignore */
  }
}

export function getTheaterCustomLeadCast(): TheaterCast {
  try {
    const v = localStorage.getItem(CUSTOM_LEAD_KEY)
    if (v === 'b')
      return 'b'
  }
  catch {
    /* ignore */
  }
  return 'a'
}

export function setTheaterCustomLeadCast(cast: TheaterCast): void {
  try {
    localStorage.setItem(CUSTOM_LEAD_KEY, cast)
  }
  catch {
    /* ignore */
  }
}
