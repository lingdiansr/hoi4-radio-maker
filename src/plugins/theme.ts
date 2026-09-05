import { onUnmounted } from 'vue'
import type { ThemeInstance } from 'vuetify'
import { resolveThemeId, THEME_STORAGE_KEY } from '@/plugins/vuetify'

const SYSTEM_QUERY = '(prefers-color-scheme: light)'

/**
 * Apply a persisted `settings.theme` value ('dark' | 'light' | 'system') to
 * the running Vuetify theme instance and cache the resolved id for startup.
 */
export function applyTheme(theme: ThemeInstance, setting: string | undefined | null): void {
 theme.global.name.value = resolveThemeId(setting)
 try {
  localStorage.setItem(THEME_STORAGE_KEY, theme.global.name.value)
 } catch {
  // Caching is best-effort; theme still applies for this session.
 }
}

/**
 * Follow the OS color scheme while the setting is 'system'. Call once in a
 * component setup (after the initial applyTheme); the listener auto-removes
 * on unmount.
 */
export function watchSystemTheme(theme: ThemeInstance, isSystem: () => boolean): void {
 const media = window.matchMedia(SYSTEM_QUERY)
 const onChange = () => {
  if (isSystem()) applyTheme(theme, 'system')
 }
 media.addEventListener('change', onChange)
 onUnmounted(() => media.removeEventListener('change', onChange))
}
