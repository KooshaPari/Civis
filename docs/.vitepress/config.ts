import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Civ + Venture Specs',
  description: 'Planning closure, technical specs, and governance docs',
  outDir: '../docs-dist',
  lastUpdated: true,
  ignoreDeadLinks: true,
  srcExclude: ['fragemented/**', 'context/**'],

  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Civ', link: '/civ/' },
      { text: 'Venture', link: '/venture/' },
      { text: 'Governance', link: '/governance/' },
      { text: 'Traceability', link: '/traceability/' }
    ],

    sidebar: {
      '/civ/': [
        {
          text: 'Civ Specs',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/civ/' },
            { text: 'Economy v1', link: '/specs/CIV-0100-economy-v1' },
            { text: 'Two-Zoom LOD v1', link: '/specs/CIV-0101-two-zoom-lod-v1' },
            { text: 'Climate Follow-up v1', link: '/specs/CIV-0102-climate-followup-v1' },
            { text: 'Institutions/Time-Series/Citizen', link: '/specs/CIV-0103-institutions-timeseries-citizen-lifecycle-v1' },
            { text: 'Minimal Constraint Theorem', link: '/specs/CIV-0104-minimal-constraint-set-theorem' },
            { text: 'War/Diplomacy/Shadow v1', link: '/specs/CIV-0105-war-diplomacy-shadow-v1' },
            { text: 'Social/Ideology/Health/Insurgency v1', link: '/specs/CIV-0106-social-ideology-health-insurgency-v1' }
          ]
        }
      ],
      '/venture/': [
        {
          text: 'Venture Specs',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/venture/' },
            { text: 'Technical Spec', link: '/venture/technical-spec' },
            { text: 'Data Model DB Spec', link: '/venture/data-model-db-spec' },
            { text: 'API Events Spec', link: '/venture/api-events-spec' },
            { text: 'Ops Compliance Spec', link: '/venture/ops-compliance-spec' },
            { text: 'Track A Artifact+Determinism', link: '/venture/track-a' },
            { text: 'Track B Treasury+Compliance', link: '/venture/track-b' },
            { text: 'Track C Control Plane', link: '/venture/track-c' },
            { text: 'Schema Pack', link: '/venture/schema-pack' },
            { text: 'Role Tool Matrix', link: '/venture/role-tool-allowlist-matrix' },
            { text: 'Implementation Roadmap', link: '/venture/implementation-roadmap' }
          ]
        }
      ],
      '/governance/': [
        {
          text: 'Governance',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/governance/' },
            { text: 'Track C Closure', link: '/governance/track-c-civ-sim-closure' },
            { text: 'Quality Gates', link: '/governance/QUALITY_GATES' }
          ]
        }
      ],
      '/traceability/': [
        {
          text: 'Traceability',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/traceability/' },
            { text: 'Matrix', link: '/traceability/TRACEABILITY_MATRIX' },
            { text: 'Event Taxonomy', link: '/traceability/EVENT_TAXONOMY' }
          ]
        }
      ]
    },

    search: { provider: 'local' },
    outline: 'deep'
  }
})
