/** Safe UTF-8 import path migration: utils/tauri-api → api */
import fs from 'node:fs'
import path from 'node:path'

const root = path.resolve(import.meta.dirname, '..')
const srcRoot = path.join(root, 'src')

const replacements = [
  [/from '\.\/utils\/tauri-api'/g, "from './api'"],
  [/from "\.\/utils\/tauri-api"/g, 'from "./api"'],
  [/from '\.\.\/utils\/tauri-api'/g, "from '../api'"],
  [/from "\.\.\/utils\/tauri-api"/g, 'from "../api"'],
  [/from '\.\.\/\.\.\/utils\/tauri-api'/g, "from '../../api'"],
  [/from "\.\.\/\.\.\/utils\/tauri-api"/g, 'from "../../api"'],
  [/from '\.\/tauri-api'/g, "from '../api'"],
  [/from "\.\/tauri-api"/g, 'from "../api"'],
]

function walk(dir) {
  for (const name of fs.readdirSync(dir)) {
    const p = path.join(dir, name)
    const st = fs.statSync(p)
    if (st.isDirectory()) {
      if (name === 'api') continue
      walk(p)
      continue
    }
    if (!/\.(ts|vue|js|mjs)$/.test(name)) continue
    let text = fs.readFileSync(p, 'utf8')
    let next = text
    for (const [re, rep] of replacements) {
      next = next.replace(re, rep)
    }
    if (next !== text) {
      fs.writeFileSync(p, next, 'utf8')
      console.log('updated', path.relative(root, p))
    }
  }
}

walk(srcRoot)
