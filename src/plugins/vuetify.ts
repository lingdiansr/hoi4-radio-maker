import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import 'vuetify/styles'

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        colors: {
          primary: '#D0BCFF',
          secondary: '#CCC2DC',
          surface: '#1C1B1F',
          background: '#121212',
          error: '#F2B8B5',
        },
      },
      light: {
        colors: {
          primary: '#6750A4',
          secondary: '#625B71',
          surface: '#FFFBFE',
          background: '#FFFFFF',
          error: '#B3261E',
        },
      },
    },
  },
})

export default vuetify
