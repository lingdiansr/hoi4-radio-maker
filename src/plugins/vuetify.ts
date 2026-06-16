import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import 'vuetify/styles'
import '@fontsource/oranienbaum/400.css'
import '@fontsource/source-serif-4/400.css'
import '@fontsource/source-serif-4/600.css'
import '@fontsource/jetbrains-mono/400.css'

const radioBureauTheme = {
  dark: true,
  colors: {
    background: '#12100e',
    surface: '#1a1714',
    'surface-variant': '#25211c',
    primary: '#ffb020',
    'primary-darken-1': '#cc7a00',
    'primary-lighten-1': '#ffd180',
    secondary: '#c4b5a0',
    'secondary-darken-1': '#9c8d78',
    tertiary: '#8f9e8a',
    error: '#ff8a80',
    info: '#90caf9',
    success: '#a5d6a7',
    warning: '#ffd54f',
    'on-background': '#efebe3',
    'on-surface': '#efebe3',
    'on-surface-variant': '#c4b5a0',
    'on-primary': '#1a1714',
    'on-secondary': '#1a1714',
    outline: '#4a4238',
    'outline-variant': '#2f2922',
  },
  variables: {
    'border-color': '#4a4238',
    'border-opacity': 0.24,
    'high-emphasis-opacity': 0.92,
    'medium-emphasis-opacity': 0.72,
    'disabled-opacity': 0.38,
    'idle-opacity': 0.08,
    'hover-opacity': 0.12,
    'focus-opacity': 0.16,
    'selected-opacity': 0.16,
    'activated-opacity': 0.2,
    'pressed-opacity': 0.24,
    'dragged-opacity': 0.16,
    'kbd-background-color': '#25211c',
    'kbd-color': '#efebe3',
  },
}

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'radioBureau',
    themes: {
      radioBureau: radioBureauTheme,
    },
  },
  defaults: {
    VBtn: {
      style: 'text-transform: none; letter-spacing: 0.02em;',
    },
    VCard: {
      style: 'border: 1px solid rgba(74, 66, 56, 0.4);',
    },
    VNavigationDrawer: {
      style: 'border-right: 1px solid rgba(74, 66, 56, 0.4);',
    },
    VTextField: {
      variant: 'outlined',
      density: 'comfortable',
    },
    VSelect: {
      variant: 'outlined',
      density: 'comfortable',
    },
  },
})

export default vuetify
