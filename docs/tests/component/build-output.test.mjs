import assert from 'node:assert/strict'
import { readFileSync, existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { test } from 'node:test'

const docsRoot = resolve(import.meta.dirname, '../..')

test('vitepress config exists', () => {
  const configPath = resolve(docsRoot, '.vitepress/config.ts')
  assert.ok(existsSync(configPath), 'config.ts should exist')
  const content = readFileSync(configPath, 'utf8')
  assert.ok(content.includes('defineConfig'), 'should have defineConfig')
})

test('site-meta.mjs exports createSiteMeta', () => {
  const metaPath = resolve(docsRoot, '.vitepress/site-meta.mjs')
  assert.ok(existsSync(metaPath), 'site-meta.mjs should exist')
  const content = readFileSync(metaPath, 'utf8')
  assert.ok(content.includes('createSiteMeta'), 'should export createSiteMeta')
})
