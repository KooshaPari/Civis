import { defineLoader } from 'vitepress'
import { readFileSync } from 'fs'
import { resolve } from 'path'

export interface RegistryPackEntry {
  id: string
  name: string
  author: string
  version: string
  type: string
  description: string
  tags: string[]
  repo: string
  download_url: string
  pack_path: string
  framework_version: string
  verified: boolean
  featured: boolean
  conflicts_with?: string[]
  depends_on?: string[]
}

export interface RegistryDocument {
  version: string
  updated: string
  packs: RegistryPackEntry[]
}

declare const data: RegistryDocument
export { data }

export default defineLoader({
  load(): RegistryDocument {
    const registryPath = resolve(__dirname, '../public/registry.json')
    const raw = readFileSync(registryPath, 'utf-8')
    return JSON.parse(raw) as RegistryDocument
  },
})
