import { mockInvoke } from './fixtures'

/** Build-time stand-in for Tauri's IPC channel; mock-browser tests use plain sends. */
export class Channel<T> {
  onmessage: ((message: T) => void) | null = null
}

export async function invoke<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const hold = ({
    import_role_pack_command: 'Import',
    peek_role_pack_command: 'Peek',
    send_message: 'Send',
    get_llm_user_settings: 'Settings',
    get_global_ollama_model: 'Global',
    list_roles: 'Library',
  } as Record<string, string>)[command]
  if (hold && localStorage.getItem(`oclive.e2e.hold${hold}`) === '1') {
    await new Promise<void>((resolve) => {
      window.addEventListener(`oclive:e2e:release-${hold.toLowerCase()}`, () => resolve(), { once: true })
    })
  }
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
