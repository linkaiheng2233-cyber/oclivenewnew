#!/usr/bin/env node
/**
 * Generate SHA256SUMS for release artifacts (kernel binary, manifest sidecar, etc.).
 *
 * Usage:
 *   node scripts/generate-sha256sums.mjs [--out PATH] file1 file2 ...
 *   node scripts/generate-sha256sums.mjs --out SHA256SUMS dist/path/to/binary
 */
import { createHash } from 'crypto'
import { createReadStream, writeFileSync } from 'fs'
import { basename, resolve } from 'path'
import { fileURLToPath } from 'url'

const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)))

function parseArgs(argv) {
  const files = []
  let out = null
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--out' && argv[i + 1]) {
      out = resolve(ROOT, argv[++i])
    } else if (argv[i] === '--help' || argv[i] === '-h') {
      console.log(
        'Usage: node scripts/generate-sha256sums.mjs [--out PATH] <file> [file...]',
      )
      process.exit(0)
    } else if (!argv[i].startsWith('-')) {
      files.push(resolve(ROOT, argv[i]))
    }
  }
  return { files, out }
}

function sha256File(filePath) {
  return new Promise((resolvePromise, reject) => {
    const hash = createHash('sha256')
    const stream = createReadStream(filePath)
    stream.on('error', reject)
    stream.on('data', (chunk) => hash.update(chunk))
    stream.on('end', () => resolvePromise(hash.digest('hex')))
  })
}

async function main() {
  const { files, out } = parseArgs(process.argv.slice(2))
  if (files.length === 0) {
    console.error('generate-sha256sums: at least one file path required')
    process.exit(1)
  }

  const lines = []
  for (const filePath of files) {
    const hex = await sha256File(filePath)
    lines.push(`${hex}  ${basename(filePath)}`)
  }
  const body = `${lines.join('\n')}\n`
  if (out) {
    writeFileSync(out, body, 'utf8')
    console.log(`[generate-sha256sums] wrote ${out.replace(/\\/g, '/')}`)
  } else {
    process.stdout.write(body)
  }
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : String(err))
  process.exit(1)
})
