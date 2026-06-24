export function createSiteMeta({ base = '/' } = {}) {
  return {
    base,
    title: 'civ',
    description: 'Documentation',
    themeConfig: {
      nav: [
        { text: 'Home', link: base || '/' },
      ],
    },
  }
}
