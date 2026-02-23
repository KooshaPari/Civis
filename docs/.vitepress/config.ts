import { defineConfig } from 'vitepress'

// Supported locales: en, zh-CN, zh-TW, fa, fa-Latn
const locales = {
  root: {
    label: "English",
    lang: "en",
    title: 'Civ + Venture Specs',
    description: 'Planning closure, technical specs, and governance docs'
  },
  "zh-CN": {
    label: "简体中文",
    lang: "zh-CN",
    title: 'Civ + 风险投资规范',
    description: '规划终结、技术规范和治理文档'
  },
  "zh-TW": {
    label: "繁體中文",
    lang: "zh-TW",
    title: 'Civ + 風險投資規範',
    description: '規劃終結、技術規範和治理文檔'
  },
  fa: {
    label: "فارسی",
    lang: "fa",
    title: 'مشخصات Civ + Venture',
    description: 'بستن برنامه ریزی، مشخصات فنی و مستندات حکمرانی'
  },
  "fa-Latn": {
    label: "Pinglish",
    lang: "fa-Latn",
    title: 'Civ + Venture Specs',
    description: 'Planning closure, technical specs (Latin)'
  }
};

export default defineConfig({
  title: 'Civ + Venture Specs',
  description: 'Planning closure, technical specs, and governance docs',
  outDir: '../docs-dist',
  lastUpdated: true,
  ignoreDeadLinks: true,
  srcExclude: ['fragemented/**', 'context/**', 'specs/**', 'models/civ-sim/fragemented/**'],
  locales,
  markdown: {
    config: (md) => md.set({ html: false })
  },

  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Civ', link: '/civ/' },
      { text: 'Venture', link: '/venture/' },
      { text: 'Governance', link: '/governance/' },
      { text: 'Traceability', link: '/traceability/' },
      {
        text: "🌐 Language",
        items: [
          { text: "English", link: "/" },
          { text: "简体中文", link: "/zh-CN/" },
          { text: "繁體中文", link: "/zh-TW/" },
          { text: "فارسی", link: "/fa/" },
          { text: "Pinglish", link: "/fa-Latn/" }
        ]
      }
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
