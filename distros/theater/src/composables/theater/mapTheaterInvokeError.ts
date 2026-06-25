import { ApiInvokeError } from '@oclive/shared/api/helpers'
import { SceneGenTimeoutError } from './theaterLogic'

export type TheaterInvokeErrorKind =
  | 'client_timeout'
  | 'kernel_offline'
  | 'kernel_stale_cast_rewrite'
  | 'invoke_error'

export interface TheaterInvokeErrorMapping {
  kind: TheaterInvokeErrorKind
  userMessageKey: string
  recoverable: boolean
}

export function isKernelOfflineError(err: unknown): boolean {
  if (!(err instanceof ApiInvokeError))
    return false
  const code = err.code ?? ''
  const raw = (err.raw ?? '').toLowerCase()
  return code === 'KERNEL_HTTP_UNAVAILABLE'
    || code === 'KERNEL_ATTACH_FAILED'
    || code === 'REMOTE_SERVICE_UNAVAILABLE'
    || raw.includes('connection refused')
    || raw.includes('failed to fetch')
    || raw.includes('内核')
}

export function mapTheaterInvokeError(err: unknown): TheaterInvokeErrorMapping {
  if (err instanceof SceneGenTimeoutError) {
    return {
      kind: 'client_timeout',
      userMessageKey: 'theater.poke.sceneTimeout',
      recoverable: true,
    }
  }
  if (isKernelOfflineError(err)) {
    return {
      kind: 'kernel_offline',
      userMessageKey: 'theater.poke.kernelOffline',
      recoverable: true,
    }
  }
  if (err instanceof ApiInvokeError) {
    const blob = `${err.message}\n${err.raw ?? ''}`.toLowerCase()
    if (blob.includes('base_beats must not be empty') || blob.includes('cast_rewrite')) {
      return {
        kind: 'kernel_stale_cast_rewrite',
        userMessageKey: 'theater.poke.sceneFailed',
        recoverable: true,
      }
    }
  }
  return {
    kind: 'invoke_error',
    userMessageKey: 'theater.poke.sceneFailed',
    recoverable: true,
  }
}

/** Outline sheet: timeout keeps poke key; other failures use outline.failed. */
export function mapTheaterOutlineInvokeError(err: unknown): TheaterInvokeErrorMapping {
  const mapped = mapTheaterInvokeError(err)
  if (mapped.kind === 'client_timeout')
    return mapped
  return {
    ...mapped,
    userMessageKey: 'theater.outline.failed',
    recoverable: true,
  }
}
