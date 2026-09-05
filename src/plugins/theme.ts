import type { ThemeInstance } from 'vuetify'
import { themeIdForSetting, THEME_STORAGE_KEY } from '@/plugins/vuetify'

/**
 * Apply a persisted `settings.theme` value ('dark' | 'light') to the running
 * Vuetify theme instance and cache the resolved id for the next startup.
 */
export function applyTheme(theme: ThemeInstance, setting: string | undefined | null): void {
 const id = (themeIdForSetting as Record<string, string>)[setting ?? ''] ?? 'radioBureau'
 theme.global.name.value = id
 try {
  localStorage.setItem(THEME_STORAGE_KEY, id)
 } catch {
  // Caching is best-effort; theme still applies for this session.
 }
}
