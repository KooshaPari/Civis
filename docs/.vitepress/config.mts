import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

const isPagesBuild = process.env.GITHUB_ACTIONS === 'true' || process.env.GITHUB_PAGES === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'Dino'
const docsBase = isPagesBuild ? `/${repoName}/` : '/'

export default withMermaid(
  defineConfig({
    title: 'DINOForge',
    description: 'General-purpose mod platform for Diplomacy is Not an Option',
    lang: 'en-US',
    base: docsBase,
    lastUpdated: true,
    cleanUrls: true,
    appearance: 'dark',
    ignoreDeadLinks: true,
    srcExclude: ['**/archive/**', '**/research/**', '**/sessions/**', '**/worklog/**'],

    vite: {
      build: {
        rollupOptions: {
          external: [/\.mp4$/, /\.mp3$/, /\.webm$/],
        },
      },
    },

    head: [
      ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ],

    themeConfig: {
      logo: '/favicon.svg',
      siteTitle: 'DINOForge',

      nav: [
        { text: 'Guide', link: '/guide/getting-started' },
        { text: 'Packs', link: '/packs' },
        { text: 'Concepts', link: '/concepts/architecture' },
        { text: 'Warfare', link: '/warfare/overview' },
        { text: 'Reference', link: '/reference/schemas' },
        { text: 'Roadmap', link: '/roadmap/' },
        { text: 'Community', link: '/community' },
      ],

      sidebar: [
        {
          text: 'Getting Started',
          items: [
            { text: 'Home', link: '/' },
            { text: 'Installation', link: '/guide/getting-started' },
            { text: 'Quick Start', link: '/guide/quick-start' },
            { text: 'Creating Packs', link: '/guide/creating-packs' },
            { text: 'MCP Bridge & Automation', link: '/guide/mcp-bridge' },
            { text: 'Pack Registry', link: '/packs' },
          ],
        },
        {
          text: 'Core Concepts',
          items: [
            { text: 'Architecture Overview', link: '/concepts/architecture' },
            { text: 'ECS Bridge Layer', link: '/concepts/ecs-bridge' },
            { text: 'Registry System', link: '/concepts/registry-system' },
            { text: 'Aviation System', link: '/concepts/aviation' },
          ],
        },
        {
          text: 'Specifications',
          items: [
            { text: 'Overview', link: '/specs/' },
            { text: 'User Specification', link: '/specs/user-spec' },
            { text: 'Technical Specification', link: '/specs/technical-spec' },
          ],
        },
        {
          text: 'API Reference',
          items: [
            { text: 'Registry API', link: '/api/registry' },
            { text: 'Content Loader API', link: '/api/content-loader' },
          ],
        },
        {
          text: 'Schema Reference',
          items: [
            { text: 'Unit Schema', link: '/reference/unit-schema' },
            { text: 'Building Schema', link: '/reference/building-schema' },
            { text: 'Asset Pipeline', link: '/reference/asset-pipeline' },
            { text: 'All Schemas', link: '/reference/schemas' },
          ],
        },
        {
          text: 'Asset Pipeline',
          items: [
            { text: 'Overview', link: '/asset-intake/README' },
            { text: 'Asset Control PRD', link: '/asset-intake/assetctl-prd' },
            { text: 'Quick Reference', link: '/asset-intake/quick-reference' },
            { text: 'Implementation Roadmap', link: '/asset-intake/implementation-roadmap' },
            { text: 'Sketchfab CLI Commands', link: '/asset-intake/sketchfab-cli-commands' },
          ],
        },
        {
          text: 'Warfare Domain',
          items: [
            { text: 'Overview', link: '/warfare/overview' },
            { text: 'Factions & Archetypes', link: '/warfare/factions' },
            { text: 'Unit Roles & Composition', link: '/warfare/unit-roles' },
            { text: 'Domain Specification', link: '/warfare/domain-spec' },
          ],
        },
        {
          text: 'Reference',
          items: [
            { text: 'CLI Reference', link: '/reference/cli' },
            { text: 'Game Data Reference', link: '/reference/dino-game-notes' },
            { text: 'Modding DX Reference', link: '/reference/modding-dx-reference' },
            { text: 'Modding Research', link: '/reference/modding-research' },
            { text: 'Project Semantics', link: '/reference/kooshapari-project-semantics' },
            {
              text: 'Asset Intake Specs',
              items: [
                { text: 'Blender Normalization', link: '/reference/asset-intake/blender-normalization-worker' },
                { text: 'Unity Import Contract', link: '/reference/asset-intake/unity-import-contract' },
                { text: 'Faction Taxonomy', link: '/reference/asset-intake/faction-taxonomy' },
              ],
            },
          ],
        },
        {
          text: 'Architecture Decisions',
          items: [
            { text: 'ADR Index', link: '/adr/' },
            { text: 'ADR-001: Agent-Driven Development', link: '/adr/ADR-001-agent-driven-development' },
            { text: 'ADR-002: Declarative-First Architecture', link: '/adr/ADR-002-declarative-first-architecture' },
            { text: 'ADR-003: Pack System Design', link: '/adr/ADR-003-pack-system-design' },
            { text: 'ADR-004: Registry Model', link: '/adr/ADR-004-registry-model' },
            { text: 'ADR-005: ECS Integration Strategy', link: '/adr/ADR-005-ecs-integration-strategy' },
            { text: 'ADR-006: Domain Plugin Architecture', link: '/adr/ADR-006-domain-plugin-architecture' },
            { text: 'ADR-007: Observability First', link: '/adr/ADR-007-observability-first' },
            { text: 'ADR-008: Wrap, Don\'t Handroll', link: '/adr/ADR-008-wrap-dont-handroll' },
            { text: 'ADR-009: Runtime Orchestration', link: '/adr/ADR-009-runtime-orchestration' },
            { text: 'ADR-010: Asset Intake Pipeline', link: '/adr/ADR-010-asset-intake-pipeline' },
          ],
        },
        {
          text: 'QA & Observability',
          items: [
            { text: 'Performance Benchmarks', link: '/benchmarks/' },
            { text: 'Mutation Score', link: '/mutation-score/' },
          ],
        },
        {
          text: 'Project',
          items: [
            { text: 'Roadmap', link: '/roadmap/' },
            { text: 'Community', link: '/community' },
            { text: 'Test Results', link: '/test-results/' },
            { text: 'Feature Proof', link: '/proof/' },
            { text: 'Compatibility', link: '/compat' },
            { text: 'Worklog & Research', link: '/worklog/' },
          ],
        },
      ],

      editLink: {
        pattern: 'https://github.com/KooshaPari/Dino/edit/main/docs/:path',
        text: 'Edit this page on GitHub',
      },

      search: {
        provider: 'local',
      },

      socialLinks: [
        { icon: 'github', link: 'https://github.com/KooshaPari/Dino' },
      ],

      footer: {
        message: 'Released under the MIT License.',
        copyright: '© 2025 Phenotype',
      },
    },

    mermaid: {},
  })
)
