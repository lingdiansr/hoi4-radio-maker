import { createVuetify } from 'vuetify'
import { useTheme } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import 'vuetify/styles'
import '@fontsource/oranienbaum/400.css'
import '@fontsource/source-serif-4/400.css'
import '@fontsource/source-serif-4/600.css'
import '@fontsource/jetbrains-mono/400.css'

/** Accent color family shared by both themes. */
const accent = {
  primary: '#ffb020',
  'primary-darken-1': '#b26a00',
  'primary-lighten-1': '#ffd180',
  error: '#ff8a80',
  info: '#90caf9',
  warning: '#ffd54f',
}

const radioBureauTheme = {
  dark: true,
  colors: {
    background: '#12100e',
    surface: '#1a1714',
    'surface-variant': '#25211c',
    ...accent,
    secondary: '#c4b5a0',
    'secondary-darken-1': '#9c8d78',
    tertiary: '#8f9e8a',
    success: '#a5d6a7',
    'on-background': '#efebe3',
    'on-surface': '#efebe3',
    'on-surface-variant': '#c4b5a0',
    'on-primary': '#201a12',
    'on-secondary': '#201a12',
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

/** Light variant of the radio-bureau palette (amber accent on warm paper). */
const radioBureauLightTheme = {
  dark: false,
  colors: {
    background: '#f4efe6',
    surface: '#fffdf7',
    'surface-variant': '#e9e2d3',
    ...accent,
    secondary: '#6f6352',
    'secondary-darken-1': '#564c3e',
    tertiary: '#5a6b5f',
    success: '#2e7d32',
    'on-background': '#211c15',
    'on-surface': '#211c15',
    'on-surface-variant': '#4a4238',
    'on-primary': '#201a12',
    'on-secondary': '#fffdf7',
    outline: '#8a7f6e',
    'outline-variant': '#c9bfac',
  },
  variables: {
    'border-color': '#8a7f6e',
    'border-opacity': 0.28,
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
    'kbd-background-color': '#e9e2d3',
    'kbd-color': '#211c15',
  },
}

/** Vuetify theme ids keyed by the `settings.theme` values users can pick. */
export const themeIdForSetting = {
  dark: 'radioBureau',
  light: 'radioBureauLight',
} as const

/** localStorage key caching the applied theme id so startup avoids a dark→light flash. */
export const THEME_STORAGE_KEY = 'hoi4-radio-maker:theme'

function cachedThemeId(): string {
  try {
    const cached = localStorage.getItem(THEME_STORAGE_KEY)
    return cached ?? 'radioBureau'
  } catch {
    return 'radioBureau'
  }
}

export const themeConfig = {
  defaultTheme: cachedThemeId(),
  themes: {
    radioBureau: radioBureauTheme,
    radioBureauLight: radioBureauLightTheme,
  },
}

const vuetify = createVuetify({
  components,
  directives,
  theme: themeConfig,
  defaults: {
    VBtn: {
      style: 'text-transform: none; letter-spacing: 0.02em;',
    },
    VCard: {
      style: 'border: 1px solid rgba(var(--v-theme-outline), 0.4);',
    },
    VNavigationDrawer: {
      style: 'border-right: 1px solid rgba(var(--v-theme-outline), 0.4);',
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

export { useTheme }
export default vuetify
