# Phenotype Docs Theme — "Console Holo"

A reusable VitePress theme for Phenotype-org documentation sites. Implements the
locked UI design language (`docs/design/ui-design-language.md`): a near-monochrome
**graphite glass console** (Xbox-2001 beveled chrome × Geist type × neo-glass),
**restrained electric-green neon** as edge/glow only, an **amber** semantic signal,
and a **Star-Wars hologram** layer (cyan scanline/glow callouts + hero accents).

## Architecture: shared base + per-project branding

```
.vitepress/theme/
├── index.ts            # Civis entry — wires base + project config (thin, ~5 lines)
├── project.config.ts   # Civis-OWNED branding (name + accent hexes) — the only project file
└── phenotype/          # SHARED BASE — identical across every Phenotype docs site
    ├── index.ts        # createPhenotypeTheme(project) factory
    ├── tokens.css      # locked design tokens (graphite ramp, neon, amber, holo, Geist)
    ├── base.css        # token → VitePress mapping + glass/bevel/holo styling
    ├── fonts.css       # Geist Sans + Geist Mono (Inter / JetBrains Mono fallback)
    └── README.md       # this file
```

The **base** (`phenotype/`) carries the entire design language. A **project**
overrides only its branding through a small config object — never by forking the
base. The accent color is wired through a `--ph-accent*` indirection layer, so
changing three hexes rebrands links / active states / focus glow / hover edges
while the graphite chrome, holo cyan, amber signal, and Geist type stay locked.

## Consuming this theme in another Phenotype project

1. Copy the `phenotype/` directory into your project's `.vitepress/theme/`.
2. Add a `project.config.ts`:

   ```ts
   import type { PhenotypeProject } from './phenotype'
   export const project: PhenotypeProject = {
     name: 'YourProject',
     accent: '#7fe9ff',     // optional — defaults to Phenotype green #3df07a
     accentHi: '#bdf4ff',   // optional
     accentDim: '#2a6f80',  // optional
   }
   ```

3. Point `.vitepress/theme/index.ts` at the factory:

   ```ts
   import { createPhenotypeTheme } from './phenotype'
   import { project } from './project.config'
   export default createPhenotypeTheme(project)
   ```

That's it — branding in one file, design inherited.

## Authoring with the holo layer

A `::: holo` custom container renders the Star-Wars projection treatment
(translucent cyan fill, 3px scanlines, animated scan-sweep, holo-glow border) for
special callouts. Register it once in `.vitepress/config.ts`:

```ts
markdown: {
  config(md) {
    md.use(containerPlugin, 'holo') // or any markdown-it container plugin
  },
}
```

Use sparingly — holo is "expensive attention" (≤8% of the page). Standard
`::: tip / warning / danger / info` blocks render as graphite glass slabs.

## Reuse-protocol note (cross-repo extraction)

Per the Phenotype Cross-Project Reuse Protocol, the natural next step is lifting
`phenotype/` into a real shared package — `@phenotype/docs-theme` (a workspace or
standalone repo) — so every Phenotype docs site depends on one published version
instead of a copied directory.

**That cross-repo move is a user-confirmation-gated follow-up** (ownership +
rollout impact across repos) and is intentionally NOT performed here. This in-repo
`phenotype/` directory is the staging form: self-contained, copy-paste reusable,
and already structured (factory + tokens + per-project config) so extraction is a
mechanical move with no API change.
