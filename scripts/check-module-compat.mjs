#!/usr/bin/env node
/**
 * Cross-layer compatibility ratchet for bundled Chat Pro directory plugins.
 *
 * Keeps the kernel slot registry, shared frontend constants, plugin manifests,
 * fallback entries, native Vue entries, and RPC declarations structurally aligned.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

function fail(message) {
  throw new Error(`module-compat: ${message}`)
}

function read(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8')
}

function parseBackendSlots() {
  const source = read('kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/bootstrap_dto.rs')
  const block = source.match(/pub const EMBEDDED_UI_SLOT_NAMES:[\s\S]*?=\s*&\[([\s\S]*?)\];/)
  if (!block)
    fail('cannot locate backend EMBEDDED_UI_SLOT_NAMES')
  return [...block[1].matchAll(/"([^"]+)"/g)].map(match => match[1])
}

function parseFrontendSlots() {
  const source = read('distros/shared/src/stores/plugin/constants.ts')
  const constants = new Map(
    [...source.matchAll(/export const (SLOT_[A-Z_]+)\s*=\s*'([^']+)'/g)]
      .map(match => [match[1], match[2]]),
  )
  const block = source.match(/export const ALL_EMBEDDED_SLOT_NAMES:[\s\S]*?=\s*\[([\s\S]*?)\]/)
  if (!block)
    fail('cannot locate frontend ALL_EMBEDDED_SLOT_NAMES')
  return [...block[1].matchAll(/\b(SLOT_[A-Z_]+)\b/g)].map((match) => {
    const value = constants.get(match[1])
    if (!value)
      fail(`frontend slot constant ${match[1]} has no string value`)
    return value
  })
}

function assertSameOrderedValues(label, expected, actual) {
  if (JSON.stringify(expected) !== JSON.stringify(actual)) {
    fail(`${label} drift\nbackend=${JSON.stringify(expected)}\nfrontend=${JSON.stringify(actual)}`)
  }
}

function validateBundledManifests(slotNames) {
  const pluginsRoot = path.join(root, 'distros', 'chat-pro', 'plugins')
  const knownSlots = new Set(slotNames)
  const pluginDirs = fs.readdirSync(pluginsRoot, { withFileTypes: true })
    .filter(entry => entry.isDirectory())
    .map(entry => path.join(pluginsRoot, entry.name))
  let manifestCount = 0
  let uiSlotCount = 0

  for (const pluginDir of pluginDirs) {
    const manifestPath = path.join(pluginDir, 'manifest.json')
    if (!fs.existsSync(manifestPath))
      continue
    manifestCount += 1
    const relativeManifest = path.relative(root, manifestPath).replaceAll('\\', '/')
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
    if (manifest.schema_version !== 1)
      fail(`${relativeManifest}: schema_version must be 1`)
    if (typeof manifest.id !== 'string' || !manifest.id.trim())
      fail(`${relativeManifest}: id is required`)
    if (typeof manifest.version !== 'string' || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(manifest.version))
      fail(`${relativeManifest}: version must be semver`)

    const rpcMethods = new Set(manifest.rpcMethods ?? [])
    if (rpcMethods.size > 0 && !manifest.process)
      fail(`${relativeManifest}: rpcMethods requires process`)
    for (const method of Object.keys(manifest.rpcTimeoutsMs ?? {})) {
      if (!rpcMethods.has(method))
        fail(`${relativeManifest}: rpcTimeoutsMs.${method} is absent from rpcMethods`)
    }

    const appearances = new Set()
    for (const slot of manifest.ui_slots ?? []) {
      uiSlotCount += 1
      if (!knownSlots.has(slot.slot))
        fail(`${relativeManifest}: unsupported ui slot ${JSON.stringify(slot.slot)}`)
      const appearanceKey = `${slot.slot}\u0000${String(slot.appearance_id ?? '').trim()}`
      if (appearances.has(appearanceKey))
        fail(`${relativeManifest}: duplicate slot appearance ${JSON.stringify(appearanceKey)}`)
      appearances.add(appearanceKey)
      for (const field of ['entry', 'vueComponent']) {
        if (!slot[field]) {
          if (field === 'entry')
            fail(`${relativeManifest}: ui_slots[].entry is required`)
          continue
        }
        const assetPath = path.resolve(pluginDir, slot[field])
        const relative = path.relative(pluginDir, assetPath)
        if (relative.startsWith('..') || path.isAbsolute(relative))
          fail(`${relativeManifest}: ${field} escapes plugin root: ${slot[field]}`)
        if (!fs.existsSync(assetPath))
          fail(`${relativeManifest}: missing ${field} asset ${slot[field]}`)
      }
    }
  }

  if (manifestCount === 0)
    fail('no bundled plugin manifests found')
  return { manifestCount, uiSlotCount }
}

const backendSlots = parseBackendSlots()
const frontendSlots = parseFrontendSlots()
assertSameOrderedValues('embedded UI slot registry', backendSlots, frontendSlots)
const result = validateBundledManifests(backendSlots)
console.log(`module-compat: OK (${backendSlots.length} slots, ${result.manifestCount} manifests, ${result.uiSlotCount} UI contributions)`)
