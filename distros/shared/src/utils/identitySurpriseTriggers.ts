const IDENTITY_HINT_PATTERNS = [
  /其实我是/u,
  /当你的/u,
  /我是你/u,
  /扮演/u,
  /我的身份/u,
  /我是谁/u,
  /as your/i,
  /i am your/i,
  /my role is/i,
  /identity/i,
]

export function messageHintsUserIdentity(text: string): boolean {
  const t = text.trim()
  if (!t)
    return false
  return IDENTITY_HINT_PATTERNS.some(p => p.test(t))
}
