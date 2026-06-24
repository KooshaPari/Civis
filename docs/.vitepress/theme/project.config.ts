/**
 * Civis — per-project branding for the shared Phenotype docs theme.
 *
 * This is the ONLY file Civis owns in the theme. It overrides branding only;
 * the graphite glass chrome, Star-Wars holo layer, amber signal, and Geist type
 * are inherited unchanged from ./phenotype (the shared base).
 *
 * To rebrand for another Phenotype project: copy this file, change `name` and
 * the accent hexes. Leave everything in ./phenotype untouched.
 */
import type { PhenotypeProject } from './phenotype'

export const project: PhenotypeProject = {
  name: 'Civis',
  // Civis keeps the Phenotype electric-green signature (the Xbox-2001 hue).
  // Set these to rebrand, e.g. a future project might use cyan or amber.
  accent: '#3df07a',
  accentHi: '#8bffb4',
  accentDim: '#1e7a45',
}
