import { invokeWithFriendlyError } from './helpers'

export async function writeUserTextFile(path: string, contents: string): Promise<void> {
  await invokeWithFriendlyError('write_user_text_file', { path, contents })
}
