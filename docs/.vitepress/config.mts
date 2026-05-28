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
    ignoreDeadLinks: [
      // Local dev server URLs in dev-only deployment guides (not for production link checking)
      /^https?:\/\/localhost(:\d+)?\//,
      /^https?:\/\/localhost:\d+$/,
    ],
    srcExclude: ['**/archive/**', '**/research/**', '**/sessions/**', '**/worklog/**', 'game-launch-dashboard.md'],

    vue: {
      template: {
        compilerOptions: {
          isCustomElement: (tag) =>
            tag.startsWith('journey-') ||
            tag === 'journey-viewer'
        }
      }
    },

    vite: {
      build: {
        rollupOptions: {
          external: [/\.mp4$/, /\.mp3$/, /\.webm$/],
        },
      },
    },

    head: [
      ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
      ['link', { rel: 'stylesheet', href: '/.vitepress/theme/custom.css' }],
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
            { text: 'Your First Mod', link: '/guides/your-first-mod' },
            { text: 'For Mod Authors', link: '/guide/mod-author-guide' },
            { text: 'Troubleshooting', link: '/guide/troubleshooting' },
            { text: 'Quick Start', link: '/guide/quick-start' },
            { text: 'Creating Packs', link: '/guide/creating-packs' },
            { text: 'Pack Cookbook', link: '/guide/pack-cookbook' },
            { text: 'MCP Bridge & Automation', link: '/guide/mcp-bridge' },
            { text: 'Pack Registry', link: '/packs' },
          ],
        },
        {
          text: 'Core Concepts',
          items: [
            { text: 'Architecture Overview', link: '/concepts/architecture' },
            { text: 'Architecture Diagrams', link: '/architecture/diagrams' },
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
            { text: 'Interactive Examples', link: '/guide/api-reference-interactive' },
            { text: 'Registry API', link: '/api/registry' },
            { text: 'Content Loader API', link: '/api/content-loader' },
          ],
        },
        {
          text: 'Deployment',
          collapsed: false,
          items: [
            { text: 'Windows', link: '/deploy/windows-deployment' },
            { text: 'Linux', link: '/deploy/linux-deployment' },
            { text: 'macOS', link: '/deploy/macos-deployment' },
            { text: 'Docker', link: '/deploy/docker-deployment' },
            { text: 'Troubleshooting', link: '/deploy/troubleshooting' },
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
            { text: 'ADR-011: WinUI 3 Desktop Companion App', link: '/adr/ADR-011-desktop-companion' },
            { text: 'ADR-012: Fuzzing & Property-Based Testing Strategy', link: '/adr/ADR-012-fuzzing-strategy' },
            { text: 'ADR-013: Duplicate Instance Detection & Win32 Mutex Bypass', link: '/adr/ADR-013-duplicate-instance-detection-bypass' },
            { text: 'ADR-014: DINO Runtime Execution Model Discovery', link: '/adr/ADR-014-runtime-execution-model' },
            { text: 'ADR-015: Native Menu Injector Design', link: '/adr/ADR-015-native-menu-injector' },
            { text: 'ADR-016: No Harmony Patches on DINO Systems', link: '/adr/ADR-016-no-harmony-patches-on-dino-systems' },
            { text: 'ADR-017: Neural Text-to-Speech for Proof Videos', link: '/adr/ADR-017-neural-tts-for-proof-videos' },
            { text: 'ADR-018: Second Instance Detection and Bypass', link: '/adr/ADR-018-second-instance-bypass' },
            { text: 'ADR-019: Local Mod Manager Client (M12)', link: '/adr/ADR-019-mod-manager-client' },
          ],
        },
        {
          text: 'Operational Docs',
          collapsed: true,
          items: [
            { text: 'Asset CLI Implementation', link: '/ASSET_CLI_IMPLEMENTATION' },
            { text: 'CLI Tools Distribution', link: '/CLI_TOOLS_DISTRIBUTION' },
            { text: 'Concurrent Instances', link: '/CONCURRENT_INSTANCES' },
            { text: 'Concurrent Instances Quick Start', link: '/CONCURRENT_INSTANCES_QUICK_START' },
            { text: 'Deployment', link: '/DEPLOYMENT' },
            { text: 'Developer Guide', link: '/DEVELOPER_GUIDE' },
            { text: 'Extended Feature Eval', link: '/EXTENDED_FEATURE_EVAL' },
            { text: 'Libification Roadmap', link: '/LIBIFICATION_ROADMAP' },
            { text: 'NuGet Publishing Guide', link: '/NUGET_PUBLISHING_GUIDE' },
            { text: 'Parallel Automation', link: '/PARALLEL_AUTOMATION' },
            { text: 'Parallel Automation Setup', link: '/PARALLEL_AUTOMATION_SETUP' },
            { text: 'Performance Baseline', link: '/PERFORMANCE_BASELINE' },
            { text: 'Performance Optimizations', link: '/PERFORMANCE_OPTIMIZATIONS' },
            { text: 'Platform Support Matrix', link: '/PLATFORM_SUPPORT_MATRIX' },
            { text: 'Platform Validation Results', link: '/PLATFORM_VALIDATION_RESULTS' },
            { text: 'Polyglot Build', link: '/POLYGLOT_BUILD' },
            { text: 'Polyglot Build Workflow', link: '/POLYGLOT_BUILD_WORKFLOW' },
            { text: 'Polyglot Integration', link: '/POLYGLOT_INTEGRATION' },
            { text: 'Project Status', link: '/PROJECT_STATUS' },
            { text: 'QA Matrix', link: '/QA_MATRIX' },
            { text: 'QA Validation Matrix', link: '/QA_VALIDATION_MATRIX' },
            { text: 'Release Automation Setup', link: '/RELEASE_AUTOMATION_SETUP' },
            { text: 'Release Status', link: '/RELEASE_STATUS' },
            { text: 'Roadmap v0.24.0', link: '/ROADMAP_v0.24.0' },
            { text: 'Single Instance Bypass Guide', link: '/SINGLE_INSTANCE_BYPASS_GUIDE' },
            { text: 'Structured Logging', link: '/STRUCTURED_LOGGING' },
            { text: 'Task Runner', link: '/TASK_RUNNER' },
            { text: 'Test Environment Audit', link: '/TEST_ENVIRONMENT_AUDIT' },
            { text: 'Test Strategy', link: '/TEST_STRATEGY' },
            { text: 'Truth Table', link: '/TRUTH_TABLE' },
            { text: 'WBS', link: '/WBS' },
            { text: 'v0.25.0 Readiness Status', link: '/v0.25.0-readiness-status' },
            { text: 'Product Requirements Document', link: '/product-requirements-document' },
            { text: 'Test Quality Metrics', link: '/test-quality-metrics' },
            { text: 'Test Validation Matrix', link: '/test-validation-matrix' },
            { text: 'User Journeys', link: '/user-journeys' },
            { text: 'User Journeys (Expanded)', link: '/user-journeys-expanded' },
            { text: 'User Journeys (Tier 2)', link: '/user-journeys-tier2' },
          ],
        },
        {
          text: 'QA & Observability',
          items: [
            { text: 'Pattern Catalog', link: '/quality/pattern-catalog' },
            { text: 'Roslyn Analyzers', link: '/quality/roslyn-analyzers' },
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
        copyright: '© 2026 Phenotype',
      },
    },

    mermaid: {},
  })
)
