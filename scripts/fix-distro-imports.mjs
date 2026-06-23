#!/usr/bin/env node
/**
 * Rewrite frontend imports after kernel/distros split.
 * Shared modules: @oclive/shared/*
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const sharedComposable = new Set(
  fs.readdirSync(path.join(root, 'distros/shared/src/composables')).filter((f) => f.endsWith('.ts')),
)

const SHARED_SEGMENTS = ['api', 'stores', 'i18n', 'lib', 'utils', 'components', 'types', 'adapters', 'styles']

function sharedComposableImport(name) {
  return sharedComposable.has(name) ? `@oclive/shared/composables/${name.replace(/\.ts$/, '')}` : null
}

function rewriteContent(text, filePath) {
  let out = text
  for (const seg of SHARED_SEGMENTS) {
    out = out.replaceAll(`from '../${seg}`, `from '@oclive/shared/${seg}`)
    out = out.replaceAll(`from "../${seg}`, `from "@oclive/shared/${seg}`)
    out = out.replaceAll(`from '../../${seg}`, `from '@oclive/shared/${seg}`)
    out = out.replaceAll(`from "../../${seg}`, `from "@oclive/shared/${seg}`)
    out = out.replaceAll(`from '../../../${seg}`, `from '@oclive/shared/${seg}`)
    out = out.replaceAll(`import('../${seg}`, `import('@oclive/shared/${seg}`)
    out = out.replaceAll(`import("../../${seg}`, `import('@oclive/shared/${seg}`)
    out = out.replaceAll(`import('@oclive/shared/styles`, `import('@oclive/shared/styles`)
  }

  out = out.replace(/from ['"]\.\/([\w.-]+)['"]/g, (m, name) => {
    const hit = sharedComposableImport(`${name}.ts`)
    return hit ? `from '${hit}'` : m
  })
  out = out.replace(/from ['"]\.\.\/composables\/([\w.-]+)['"]/g, (m, name) => {
    const hit = sharedComposableImport(`${name}.ts`)
    return hit ? `from '@oclive/shared/composables/${name}'` : m
  })
  out = out.replace(/from ['"]\.\.\/\.\.\/composables\/([\w.-]+)['"]/g, (m, name) => {
    const hit = sharedComposableImport(`${name}.ts`)
    return hit ? `from '@oclive/shared/composables/${name}'` : m
  })

  if (filePath.includes(`${path.sep}chat-pro${path.sep}`)) {
    out = out.replace(
      /import\('\.\/shells\/theater\//g,
      "import('@oclive/theater/shells/theater/",
    )
  }

  return out
}

function walk(dir) {
  for (const name of fs.readdirSync(dir)) {
    const p = path.join(dir, name)
    const st = fs.statSync(p)
    if (st.isDirectory()) walk(p)
    else if (/\.(vue|ts|tsx)$/.test(name)) {
      const raw = fs.readFileSync(p, 'utf8')
      const next = rewriteContent(raw, p)
      if (next !== raw) fs.writeFileSync(p, next)
    }
  }
}

for (const d of ['distros/chat-pro', 'distros/theater', 'distros/shared']) {
  walk(path.join(root, d))
}
console.log('fix-distro-imports: done')
