import { h } from 'vue'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import JourneyViewer from './components/JourneyViewer.vue'
import PackCard from './components/PackCard.vue'
import PackFilter from './components/PackFilter.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  Layout: () => {
    return h(DefaultTheme.Layout, null, {
      // Customize as needed
    })
  },
  enhanceApp({ app, router, siteData }) {
    // Register global components
    app.component('JourneyViewer', JourneyViewer)
    app.component('PackCard', PackCard)
    app.component('PackFilter', PackFilter)
  }
} satisfies Theme
