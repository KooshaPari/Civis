import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Civ',
  description: 'Planning closure, technical specs, and governance docs',
  ignoreDeadLinks: true,
  cleanUrls: true,
  srcExclude: ['fragemented/**'],
  markdown: {
    lineNumbers: true,
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
  },
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
    socialLinks: [{ icon: 'github', link: 'https://github.com/KooshaPari/civ' }],
    search: { provider: 'local' },
  },
})
