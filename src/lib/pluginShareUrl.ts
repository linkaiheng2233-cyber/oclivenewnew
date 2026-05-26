/** How a pasted plugin-market share URL should be handled. */
export type PluginShareUrlKind = 'index' | 'git' | 'invalid'

/**
 * Classify a creator-shared URL:
 * - `index` — `plugins.json` or any HTTP(S) catalog JSON (fetched via sync_plugin_index_command)
 * - `git` — repository clone URL (install via install_plugin_from_git)
 */
export function classifyPluginShareUrl(raw: string): PluginShareUrlKind {
  const s = raw.trim()
  if (!s) {
    return 'invalid'
  }
  const lower = s.toLowerCase()
  if (lower.endsWith('.json') || lower.includes('plugins.json')) {
    return 'index'
  }
  if (
    s.startsWith('git@')
    || /github\.com|gitlab\.com|gitee\.com|codeberg\.org|bitbucket\.org/i.test(s)
  ) {
    return 'git'
  }
  if (lower.startsWith('http://') || lower.startsWith('https://')) {
    return 'index'
  }
  return 'invalid'
}
