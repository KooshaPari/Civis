import type { Theme } from 'vitepress'
import PhenoDocsTheme from '@phenodocs-theme'
import './custom.css'

const theme: Theme = {
  ...PhenoDocsTheme,
  enhanceApp({ app }) {
    PhenoDocsTheme.enhanceApp?.({ app })
  },
}

export default theme
