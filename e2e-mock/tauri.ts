import { mockInvoke } from './fixtures'

/** Build-time stand-in for Tauri's IPC channel; mock-browser tests use plain sends. */
export class Channel<T> {
  onmessage: ((message: T) => void) | null = null
}

export async function invoke<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const result = mockInvoke(command, args)
  if (result === null && command !== 'find_role_asset_path') {
    console.warn(`[e2e-mock] unhandled invoke: ${command}`, args)
  }
  return result as T
}

export function convertFileSrc(path: string): string {
  return path
}

export default invoke
