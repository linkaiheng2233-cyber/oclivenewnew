import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { createSSRApp } from 'vue'
import { renderToString } from 'vue/server-renderer'
import { compilePluginVueSource } from './compilePluginVueSfc'

const repoRoot = fileURLToPath(new URL('../../../../', import.meta.url))

function findVueFiles(root: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = join(root, entry.name)
    if (entry.isDirectory())
      return findVueFiles(path)
    return entry.isFile() && entry.name.endsWith('.vue') ? [path] : []
  })
}

describe('compilePluginVueSource', () => {
  it('compiles script-setup TypeScript, template, and scoped CSS', async () => {
    const styles: string[] = []
    const component = await compilePluginVueSource(
      'com.example.safe',
      'slots/Card.vue',
      `<script setup lang="ts">
import { ref } from 'vue'
const count = ref<number>(2)
</script>
<template><button class="count">{{ count }}</button></template>
<style scoped>.count { color: red; }</style>`,
      { addStyle: css => styles.push(css) },
    )

    const html = await renderToString(createSSRApp(component))
    expect(html).toContain('>2</button>')
    expect(html).toMatch(/data-v-[0-9a-f]{8}/)
    expect(styles).toHaveLength(1)
    expect(styles[0]).toMatch(/\.count\[data-v-[0-9a-f]{8}\]/)
  })

  it('rejects imports outside the explicitly supported Vue runtime', async () => {
    await expect(compilePluginVueSource(
      'com.example.unsafe',
      'slots/Card.vue',
      `<script setup>import value from './helper.js'</script><template>{{ value }}</template>`,
      { addStyle: () => {} },
    )).rejects.toThrow('may import only "vue"')
  })

  it('rejects external source blocks and preprocessors', async () => {
    await expect(compilePluginVueSource(
      'com.example.external',
      'slots/Card.vue',
      '<template src="./Card.html" />',
      { addStyle: () => {} },
    )).rejects.toThrow('External <script src> and <template src>')

    await expect(compilePluginVueSource(
      'com.example.scss',
      'slots/Card.vue',
      '<template><div /></template><style lang="scss">div { color: red; }</style>',
      { addStyle: () => {} },
    )).rejects.toThrow('style preprocessors are not supported')
  })

  it('compiles every checked-in directory-plugin Vue entry', async () => {
    const roots = [
      join(repoRoot, 'distros/chat-pro/plugins'),
      join(repoRoot, 'examples'),
    ]
    const files = roots.flatMap(findVueFiles)
    expect(files.length).toBeGreaterThan(0)

    for (const file of files) {
      const rel = relative(repoRoot, file).replace(/\\/g, '/')
      const component = await compilePluginVueSource(
        `fixture.${rel.split('/').slice(-3, -2)[0] ?? 'plugin'}`,
        rel,
        readFileSync(file, 'utf8'),
        { addStyle: () => {} },
      )
      expect(component, rel).toBeTruthy()
    }
  }, 20_000)
})
