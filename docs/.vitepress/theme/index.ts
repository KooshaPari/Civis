import Theme from '@phenotype/docs/theme'
import CategorySwitcher from '../components/CategorySwitcher.vue'

export default {
  ...Theme,
  enhanceApp({ app }) {
    app.component('CategorySwitcher', CategorySwitcher)
  }
}
