import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Civ + Venture Specs',
  description: 'Planning closure, technical specs, and governance docs',
  outDir: '../docs-dist',
  ignoreDeadLinks: true,
  srcExclude: ['fragemented/**', 'context/**', 'specs/**', 'models/civ-sim/fragemented/**'],
  themeConfig: {
    nav: [
      { text: 'Wiki', link: '/wiki/' },
      { text: 'Development Guide', link: '/development-guide/' },
      { text: 'Document Index', link: '/document-index/' },
      { text: 'API', link: '/api/' },
      { text: 'Roadmap', link: '/roadmap/' }
    ],
    sidebar: [{ text: 'Categories', items: [
      { text: 'Wiki', link: '/wiki/' },
      { text: 'Development Guide', link: '/development-guide/' },
      { text: 'Document Index', link: '/document-index/' },
      { text: 'API', link: '/api/' },
      { text: 'Roadmap', link: '/roadmap/' }
    ] }],
    search: { provider: 'local' }
  }
})
