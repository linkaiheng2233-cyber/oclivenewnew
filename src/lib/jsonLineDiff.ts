export type DiffLineKind = 'same' | 'add' | 'remove' | 'change'

export interface DiffLine {
  kind: DiffLineKind
  before?: string
  after?: string
}

export function formatJson(value: unknown): string {
  return JSON.stringify(value, null, 2)
}

/** 简易行级 diff（用于蓝图 JSON 预览）。 */
export function computeLineDiff(before: string, after: string): DiffLine[] {
  const a = before.split('\n')
  const b = after.split('\n')
  const out: DiffLine[] = []
  const max = Math.max(a.length, b.length)
  for (let i = 0; i < max; i++) {
    const left = a[i]
    const right = b[i]
    if (left === right) {
      if (left !== undefined) {
        out.push({ kind: 'same', before: left, after: right })
      }
    }
    else if (left === undefined) {
      out.push({ kind: 'add', after: right })
    }
    else if (right === undefined) {
      out.push({ kind: 'remove', before: left })
    }
    else {
      out.push({ kind: 'change', before: left, after: right })
    }
  }
  return out
}
