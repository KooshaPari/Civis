import { h } from 'vue'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import './custom.css'

export default {
  extends: DefaultTheme,
  Layout: () => {
    return h(DefaultTheme.Layout, null, {
      // Customize as needed
    })
  },
  enhanceApp({ app, router, siteData }) {
    // Enhance as needed
  }
} satisfies Theme
