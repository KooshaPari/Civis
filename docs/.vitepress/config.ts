import { defineConfig } from 'vitepress'
import container from 'markdown-it-container'

export default defineConfig({
  title: 'Civ',
  description: 'Planning closure, technical specs, and governance docs',
  ignoreDeadLinks: true,
  cleanUrls: true,
  appearance: true,
  srcExclude: ['fragemented/**'],
  markdown: {
    lineNumbers: true,
    lazyLoading: true,
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
    // Star-Wars holo callout: `::: holo` → projection-styled block (see theme).
    config(md) {
      md.use(container, 'holo', {
        render(tokens: { nesting: number; info: string }[], idx: number) {
          const token = tokens[idx]
          if (token.nesting === 1) {
            const title = token.info.trim().slice('holo'.length).trim() || 'HOLO'
            return `<div class="custom-block holo"><p class="custom-block-title">${title}</p>\n`
          }
          return '</div>\n'
        },
      })
    },
  },
  themeConfig: {
    nav: [
      { text: 'Wiki', link: '/wiki/' },
      { text: 'Development Guide', link: '/development-guide/' },
      { text: 'Document Index', link: '/document-index/' },
      { text: 'API', link: '/api/' },
      { text: 'Roadmap', link: '/roadmap/' },
      {
        text: 'Dev UI',
        items: [
          { text: 'Dashboard', link: 'http://127.0.0.1:5173/' },
          { text: 'Status', link: 'http://127.0.0.1:5173/status.html' },
        ],
      },
    ],
    sidebar: [{ text: 'Categories', items: [
      { text: 'Wiki', link: '/wiki/' },
      { text: 'Development Guide', link: '/development-guide/' },
      { text: 'Document Index', link: '/document-index/' },
      { text: 'API', link: '/api/' },
      { text: 'Roadmap', link: '/roadmap/' }
    ] }],
    outline: {
      level: [2, 3],
      label: 'On this page',
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/KooshaPari/civ' }],
    search: {
      provider: 'local',
      options: {
        detailedView: true,
        miniSearch: {
          searchOptions: {
            fuzzy: 0.2,
            prefix: true,
            boost: { title: 4, text: 2, titles: 2 },
          },
        },
      },
    },
  },
})
