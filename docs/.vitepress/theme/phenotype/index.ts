/**
 * Phenotype docs theme — SHARED BASE (the reusable library).
 *
 * `createPhenotypeTheme(project)` returns a VitePress theme that extends the
 * default theme with the locked "Console Holo" design language (graphite glass
 * chrome + restrained neon + Geist + Star-Wars holo). The BASE is identical for
 * every Phenotype docs site; a PROJECT supplies only its branding via a small
 * config object — no project should fork these files.
 *
 * Usage (per project, in theme/index.ts):
 *
 *   import { createPhenotypeTheme } from './phenotype'
 *   import { project } from './project.config'
 *   export default createPhenotypeTheme(project)
 *
 * Cross-repo extraction (lifting this `phenotype/` dir into a real shared
 * `@phenotype/docs-theme` package consumed by every repo) is a reuse-protocol
 * follow-up gated on user confirmation — see README.md in this directory.
 */
import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'

import './fonts.css'
import './tokens.css'
import './base.css'

/** Per-project branding surface. Projects override ONLY these fields. */
export interface PhenotypeProject {
  /** Project name (used by callers for the VitePress title, etc.). */
  name: string
  /**
   * Brand accent hex. Defaults to the Phenotype electric-green signature
   * (#3DF07A). Overriding this is how a project rebrands; the graphite chrome,
   * holo cyan, amber signal, and Geist type all stay shared/locked.
   */
  accent?: string
  /** Hottest core of the accent glow (1px inner line). */
  accentHi?: string
  /** Accent at rest / pre-glow trace lines. */
  accentDim?: string
}

/** Inject a project's brand accent as CSS-var overrides on :root. */
function brandCss(project: PhenotypeProject): string | null {
  const rules: string[] = []
  if (project.accent) rules.push(`--ph-accent:${project.accent}`)
  if (project.accentHi) rules.push(`--ph-accent-hi:${project.accentHi}`)
  if (project.accentDim) rules.push(`--ph-accent-dim:${project.accentDim}`)
  if (rules.length === 0) return null
  return `:root{${rules.join(';')}}`
}

/**
 * Build a VitePress theme for a Phenotype project. Shared base + thin branding.
 */
export function createPhenotypeTheme(project: PhenotypeProject): Theme {
  const css = brandCss(project)
  return {
    extends: DefaultTheme,
    enhanceApp(ctx) {
      DefaultTheme.enhanceApp?.(ctx)
      if (css && typeof document !== 'undefined') {
        const style = document.createElement('style')
        style.setAttribute('data-phenotype-brand', project.name)
        style.textContent = css
        document.head.appendChild(style)
      }
    },
  }
}

export default createPhenotypeTheme
