let nextOpenPath = 'C:\\mock\\plugin.zip'

export function setNextOpenPath(path: string): void {
  nextOpenPath = path
}

export async function open(options?: {
  multiple?: boolean
}): Promise<string | string[] | null> {
  if (options?.multiple)
    return [nextOpenPath]
  return nextOpenPath
}

export async function save(_options?: unknown): Promise<string | null> {
  return 'C:\\mock\\export.ocpak'
}

export async function confirm(_message: string): Promise<boolean> {
  return true
}
