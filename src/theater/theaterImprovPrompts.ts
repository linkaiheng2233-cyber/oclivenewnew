import enApp from '../i18n/locales/fragments/app.en'
import zhApp from '../i18n/locales/fragments/app.zh'

export type TheaterImprovLocale = 'zh' | 'en'

type TheaterPrompts = typeof zhApp.theater

function prompts(locale: TheaterImprovLocale): TheaterPrompts {
  return locale === 'zh' ? zhApp.theater : enApp.theater
}

function fill(template: string, vars: Record<string, string>): string {
  return Object.entries(vars).reduce(
    (out, [key, value]) => out.replaceAll(`{${key}}`, value),
    template,
  )
}

export function improvSpeakerLabel(
  locale: TheaterImprovLocale,
  speaker: 'user' | 'a' | 'b',
): string {
  const p = prompts(locale)
  if (speaker === 'user')
    return p.improvPromptSpeakerUser
  if (speaker === 'a')
    return p.improvPromptSpeakerRoleA
  return p.improvPromptSpeakerRoleB
}

export function buildOllamaImprovPrompts(
  locale: TheaterImprovLocale,
  sceneTitle: string,
  roleLabel: string,
  history: string,
): { system: string, user: string } {
  const p = prompts(locale)
  const system = fill(p.improvPromptOllamaSystem, { sceneTitle, roleLabel })
  const user = history.trim()
    ? fill(p.improvPromptOllamaUser, { history, roleLabel })
    : fill(p.improvPromptOllamaUserEmpty, { roleLabel })
  return { system, user }
}

export function buildKernelImprovUserMessage(
  locale: TheaterImprovLocale,
  sceneTitle: string,
  roleLabel: string,
  history: string,
): string {
  const p = prompts(locale)
  const historyBlock = history.trim()
    ? fill(p.improvPromptKernelHistoryBlock, { history })
    : ''
  return fill(p.improvPromptKernelUser, { sceneTitle, historyBlock, roleLabel })
}

export function buildImprovFallbackLine(
  locale: TheaterImprovLocale,
  roleLabel: string,
): string {
  return fill(prompts(locale).improvPromptFallback, { roleLabel })
}
