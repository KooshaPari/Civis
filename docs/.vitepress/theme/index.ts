/**
 * Civis docs theme entry — wires the shared Phenotype base to Civis branding.
 *
 * Keep this file thin: all design lives in ./phenotype (shared, locked), all
 * branding lives in ./project.config (Civis-owned). See ./phenotype/README.md.
 */
import { createPhenotypeTheme } from './phenotype'
import { project } from './project.config'

export default createPhenotypeTheme(project)
