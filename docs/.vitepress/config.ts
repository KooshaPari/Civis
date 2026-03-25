import { createPhenotypeConfig } from '@phenotype/docs/config'

export default createPhenotypeConfig({
  title: 'Civ + Venture Specs',
  description: 'Planning closure, technical specs, and governance docs',
  githubRepo: 'civ',
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
  overrides: {
    outDir: '../docs-dist',
    srcExclude: ['fragemented/**', 'context/**', 'specs/**', 'models/civ-sim/fragemented/**'],
  }
})
